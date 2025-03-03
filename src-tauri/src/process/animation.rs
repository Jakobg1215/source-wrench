use indexmap::IndexMap;
use tauri::State;
use thiserror::Error as ThisError;

use crate::{
    import::FileManager,
    input::ImputedCompilationData,
    utilities::{
        logging::{log, LogLevel},
        mathematics::{Quaternion, Vector3},
    },
};

use super::{ProcessedAnimatedBoneData, ProcessedAnimation, ProcessedAnimationData, ProcessedBoneData, ProcessedSequence};

#[derive(Debug, ThisError)]
pub enum ProcessingAnimationError {
    #[error("Animation File Source Not Loaded")]
    FileSourceNotLoaded,
    #[error("Animation Not Found: {0}")]
    AnimationNotFound(String),
    #[error("Model Has Too Many Animations")]
    TooManyAnimations,
    #[error("Sequence Could Not Find Animation")]
    SequenceAnimationNotFound,
}

pub fn process_animations(
    input: &ImputedCompilationData,
    import: &State<FileManager>,
    bone_table: &ProcessedBoneData,
) -> Result<ProcessedAnimationData, ProcessingAnimationError> {
    struct ChannelData {
        position: Vec<Vector3>,
        rotation: Vec<Quaternion>,
    }

    let mut processed_animations = Vec::new();
    for imputed_animation in &input.animations {
        // Check if the animation is used in any sequence.
        if !input.sequences.iter().any(|sequence| {
            sequence
                .animations
                .iter()
                .any(|row| row.iter().any(|animation| animation == &imputed_animation.name))
        }) {
            log(format!("Animation \"{}\" Not Used!", imputed_animation.name), LogLevel::Warn);
            continue;
        }

        // Gather imported animation data.
        let imported_file = match import.get_file(&imputed_animation.file_source) {
            Some(file) => file,
            None => {
                return Err(ProcessingAnimationError::FileSourceNotLoaded);
            }
        };
        let remapped_bones = match bone_table.remapped_bones.get(&imputed_animation.file_source) {
            Some(remapped_bones) => remapped_bones,
            None => {
                return Err(ProcessingAnimationError::FileSourceNotLoaded);
            }
        };
        let imported_animation = match imported_file.animations.get(&imputed_animation.animation_name) {
            Some(imported_animation) => imported_animation,
            None => {
                return Err(ProcessingAnimationError::AnimationNotFound(imputed_animation.animation_name.clone()));
            }
        };

        let mut animation_channels = IndexMap::new();

        for (bone, channel) in &imported_animation.channels {
            let mapped_bone = &remapped_bones[*bone];
            if animation_channels.contains_key(&mapped_bone.index) {
                continue;
            }

            let bone = &bone_table.processed_bones[mapped_bone.index];

            animation_channels.insert(
                mapped_bone.index,
                ChannelData {
                    position: bake_channel_keyframes(&channel.position, imported_animation.frame_count.get(), bone.position),
                    rotation: bake_channel_keyframes(&channel.rotation, imported_animation.frame_count.get(), bone.rotation.to_quaternion()),
                },
            );
        }

        let frame_count = imported_animation.frame_count.get();

        // TODO: Implement animation processing.
        // TODO: Add a check if the position data is going to be out of bounds.

        // Split animation into sections
        let frames_per_sections = 30; // TODO: Make this configurable.
        let animation_section_split_threshold = 120; // TODO: Make this configurable.

        let section_count = if frame_count >= animation_section_split_threshold {
            (frame_count / frames_per_sections) + 2
        } else {
            1
        };
        let section_frame_count = if frame_count >= animation_section_split_threshold {
            frames_per_sections
        } else {
            frame_count
        };

        let mut processed_animation = ProcessedAnimation {
            name: imputed_animation.name.clone(),
            frame_count,
            sections: Vec::with_capacity(section_count),
        };

        for section in 0..section_count {
            let section_frame_start = (section * section_frame_count).min(frame_count - 1);
            let section_frame_end = ((section + 1) * section_frame_count).min(frame_count - 1);

            let mut section_data = Vec::new();
            for (index_bone, channel_data) in &animation_channels {
                let bone = &bone_table.processed_bones[*index_bone];
                let mut position = Vec::new();
                let mut rotation = Vec::new();

                // TODO: If animation is delta then skip subtracting from bone
                for frame in section_frame_start..=section_frame_end {
                    position.push(channel_data.position[frame] - bone.position);
                    rotation.push(channel_data.rotation[frame].to_angles() - bone.rotation);
                }

                section_data.push(ProcessedAnimatedBoneData {
                    bone: (*index_bone).try_into().unwrap(),
                    position,
                    rotation,
                });
            }

            processed_animation.sections.push(section_data);
        }

        processed_animations.push(processed_animation);
    }

    let mut animation_scales = vec![(Vector3::default(), Vector3::default()); bone_table.processed_bones.len()];
    for processed_animation in &processed_animations {
        for sections in &processed_animation.sections {
            for section in sections {
                for position in &section.position {
                    for axis in 0..3 {
                        let value = position[axis].abs();
                        if value > animation_scales[section.bone as usize].0[axis] {
                            animation_scales[section.bone as usize].0[axis] = value;
                        }
                    }
                }

                for rotation in &section.rotation {
                    for axis in 0..3 {
                        let value = rotation[axis].abs();
                        if value > animation_scales[section.bone as usize].1[axis] {
                            animation_scales[section.bone as usize].1[axis] = value;
                        }
                    }
                }
            }
        }
    }

    for (position, rotation) in &mut animation_scales {
        for axis in 0..3 {
            position[axis] /= (i16::MAX as f64) + 1.0;
            rotation[axis] /= (i16::MAX as f64) + 1.0;
        }
    }

    Ok(ProcessedAnimationData {
        processed_animations,
        animation_scales,
    })
}

/// Convert channel keyframes to a continuous set of values.
fn bake_channel_keyframes<T: Copy>(channel: &IndexMap<usize, T>, frame_count: usize, default: T) -> Vec<T> {
    let mut baked_channel = Vec::with_capacity(frame_count);

    for frame in 0..frame_count {
        if let Some(keyframe) = channel.get(&frame) {
            baked_channel.push(*keyframe);
            continue;
        }

        if let Some(last_value) = baked_channel.last() {
            baked_channel.push(*last_value);
            continue;
        }

        baked_channel.push(default);
    }

    baked_channel
}

pub fn process_sequences(input: &ImputedCompilationData, animations: &[ProcessedAnimation]) -> Result<Vec<ProcessedSequence>, ProcessingAnimationError> {
    let mut processed_sequences = Vec::with_capacity(input.sequences.len());

    for input_sequence in &input.sequences {
        let mut processed_sequence = ProcessedSequence {
            name: input_sequence.name.clone(),
            animations: vec![vec![0; input_sequence.animations[0].len()]; input_sequence.animations.len()],
        };

        for (row_index, row_value) in input_sequence.animations.iter().enumerate() {
            for (column_index, column_value) in row_value.iter().enumerate() {
                let animation = animations.iter().position(|animation| animation.name == *column_value);

                let animation_index = match animation {
                    Some(index) => index,
                    None => {
                        return Err(ProcessingAnimationError::SequenceAnimationNotFound);
                    }
                };

                processed_sequence.animations[row_index][column_index] = animation_index.try_into().unwrap();
            }
        }

        processed_sequences.push(processed_sequence);
    }

    Ok(processed_sequences)
}
