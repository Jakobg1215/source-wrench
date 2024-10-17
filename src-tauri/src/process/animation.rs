use indexmap::{map::Entry, IndexMap};
use tauri::State;
use thiserror::Error as ThisError;

use crate::{
    import::{FileManager, ImportKeyFrame},
    input::ImputedCompilationData,
    utilities::{
        logging::{log, LogLevel},
        mathematics::{Angles, Vector3},
    },
};

use super::{
    ProcessedAnimatedBoneData, ProcessedAnimation, ProcessedAnimationData, ProcessedAnimationEncoding, ProcessedAnimationEncodingHeader, ProcessedBoneData,
    ProcessedSequence, FLOAT_TOLERANCE,
};

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
    #[derive(Debug, Default)]
    struct ProcessingChannels {
        position: Vec<Vector3>,
        rotation: Vec<Angles>,
    }

    #[derive(Debug, Default)]
    struct RawProcessedAnimation {
        name: String,
        frame_count: usize,
        animation_data: IndexMap<usize, ProcessingChannels>,
    }

    let mut processed_animations = Vec::with_capacity(input.animations.len());

    for imputed_animation in &input.animations {
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
        let imported_animation = match imported_file.animations.iter().find(|anim| anim.name == imputed_animation.animation_name) {
            Some(imported_animation) => imported_animation,
            None => {
                return Err(ProcessingAnimationError::AnimationNotFound(imputed_animation.animation_name.clone()));
            }
        };

        let mut processing_bones = IndexMap::with_capacity(bone_table.processed_bones.len());

        for channel in &imported_animation.channels {
            let mapped_bone = &remapped_bones[channel.bone];
            if mapped_bone.was_collapsed {
                continue;
            }
            let bone = &bone_table.processed_bones[mapped_bone.bone_index];

            let processing_channel = match processing_bones.entry(mapped_bone.bone_index) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(vacant_entry) => vacant_entry.insert(ProcessingChannels::default()),
            };

            processing_channel.position = bake_channel_keyframes(&channel.position, imported_animation.frame_count, bone.position);

            processing_channel.rotation = bake_channel_keyframes(&channel.rotation, imported_animation.frame_count, bone.rotation);
        }

        for bone_index in 0..bone_table.processed_bones.len() {
            if processing_bones.contains_key(&bone_index) {
                continue;
            }

            let bone = &bone_table.processed_bones[bone_index];

            processing_bones.insert(
                bone_index,
                ProcessingChannels {
                    position: vec![bone.position; imported_animation.frame_count],
                    rotation: vec![bone.rotation; imported_animation.frame_count],
                },
            );
        }

        let frame_count = imported_animation.frame_count;

        // TODO: Implement animation processing.
        // TODO: Add a check if the position data is going to be out of bounds.

        processed_animations.push(RawProcessedAnimation {
            name: imputed_animation.name.clone(),
            frame_count,
            animation_data: processing_bones,
        });
    }

    let mut compressed_animations = Vec::new();

    let mut animation_scales = vec![(Vector3::new(f64::MIN, f64::MIN, f64::MIN), Vector3::new(f64::MIN, f64::MIN, f64::MIN)); bone_table.processed_bones.len()];
    for animation in &processed_animations {
        for (bone, channel) in &animation.animation_data {
            for frame in 0..animation.frame_count {
                let bone_data = &bone_table.processed_bones[*bone];

                for axis in 0..3 {
                    // TODO: If the animation is delta then it should not be subtracted from the bone data.
                    let value = channel.position[frame][axis] - bone_data.position[axis];
                    if value > animation_scales[*bone].0[axis] {
                        animation_scales[*bone].0[axis] = value.abs();
                    }
                }

                for axis in 0..3 {
                    // TODO: If the animation is delta then it should not be subtracted from the bone data.
                    let value = channel.rotation[frame][axis] - bone_data.rotation[axis];
                    if value > animation_scales[*bone].0[axis] {
                        animation_scales[*bone].0[axis] = value.abs();
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

    for processed_animation in processed_animations {
        // Check if the animation is used in any sequence.
        if !input.sequences.iter().any(|sequence| {
            sequence
                .animations
                .iter()
                .any(|row| row.iter().any(|animation| animation == &processed_animation.name))
        }) {
            log(format!("Animation \"{}\" Not Used!", processed_animation.name), LogLevel::Warn);
            continue;
        }

        let frames_per_sections = 30; // TODO: Make this configurable.
        let animation_section_split_threshold = 120; // TODO: Make this configurable.

        let section_count = if processed_animation.frame_count >= animation_section_split_threshold {
            (processed_animation.frame_count / frames_per_sections) + 2
        } else {
            1
        };
        let section_frame_count = if processed_animation.frame_count >= animation_section_split_threshold {
            frames_per_sections
        } else {
            processed_animation.frame_count
        };

        let mut compressed_animation = ProcessedAnimation {
            name: processed_animation.name,
            frame_count: processed_animation.frame_count,
            sections: Vec::with_capacity(section_count),
        };

        let mut sorted_bones = processed_animation.animation_data.keys().collect::<Vec<_>>();
        sorted_bones.sort();

        for section in 0..section_count {
            let section_frame_start = (section * section_frame_count).min(processed_animation.frame_count);
            let section_frame_end = ((section + 1) * section_frame_count).min(processed_animation.frame_count);

            type AnimationValues = [Option<Vec<ProcessedAnimationEncoding>>; 3];

            let mut section_bones: IndexMap<usize, (AnimationValues, AnimationValues)> = IndexMap::new();

            fn add_value_to_encoding(axis: &mut Option<Vec<ProcessedAnimationEncoding>>, local_frame_count: usize, value: f64) {
                if let Some(encodings) = axis {
                    let last_encoding = encodings.last_mut().unwrap();

                    if last_encoding.header.total == u8::MAX {
                        encodings.push(ProcessedAnimationEncoding {
                            header: ProcessedAnimationEncodingHeader { valid: 1, total: 1 },
                            values: vec![value],
                        });
                        return;
                    }

                    let last_value = *last_encoding.values.last().unwrap();

                    if (value - last_value).abs() < FLOAT_TOLERANCE {
                        let last_encoding = encodings.last_mut().unwrap();
                        last_encoding.header.total += 1;
                        return;
                    }

                    if last_encoding.header.valid == last_encoding.header.total {
                        last_encoding.header.valid += 1;
                        last_encoding.header.total += 1;
                        last_encoding.values.push(value);
                        return;
                    }

                    encodings.push(ProcessedAnimationEncoding {
                        header: ProcessedAnimationEncodingHeader { valid: 1, total: 1 },
                        values: vec![value],
                    });
                    return;
                }

                if local_frame_count == 0 {
                    *axis = Some(vec![
                        (ProcessedAnimationEncoding {
                            header: ProcessedAnimationEncodingHeader { valid: 1, total: 1 },
                            values: vec![value],
                        }),
                    ]);
                    return;
                }

                if local_frame_count == 1 {
                    *axis = Some(vec![
                        (ProcessedAnimationEncoding {
                            header: ProcessedAnimationEncodingHeader { valid: 2, total: 2 },
                            values: vec![0.0, value],
                        }),
                    ]);
                    return;
                }

                let mut total_empty_frames = local_frame_count;

                let mut empty_frames = Vec::new();

                while total_empty_frames > u8::MAX as usize {
                    empty_frames.push(ProcessedAnimationEncoding {
                        header: ProcessedAnimationEncodingHeader { valid: 1, total: u8::MAX },
                        values: vec![0.0],
                    });

                    total_empty_frames -= u8::MAX as usize;
                }

                if total_empty_frames > 0 {
                    empty_frames.push(ProcessedAnimationEncoding {
                        header: ProcessedAnimationEncodingHeader {
                            valid: 1,
                            total: total_empty_frames as u8,
                        },
                        values: vec![0.0],
                    });
                }

                empty_frames.push(ProcessedAnimationEncoding {
                    header: ProcessedAnimationEncodingHeader { valid: 1, total: 1 },
                    values: vec![value],
                });

                *axis = Some(empty_frames);
            }

            let mut local_frame_count = 0;
            for frame_index in section_frame_start..section_frame_end {
                for bone in &sorted_bones {
                    let channel_data = processed_animation.animation_data.get(*bone).unwrap();
                    let bone_data = &bone_table.processed_bones[**bone];

                    for axis in 0..3 {
                        // TODO: If the animation is delta then it should not be subtracted from the bone data.
                        let value = channel_data.position[frame_index][axis] - bone_data.position[axis];

                        let section_bone = section_bones.entry(**bone).or_default();
                        let section_axis = &mut section_bone.0[axis];

                        let always_add = match section_axis {
                            Some(encodings) => {
                                let last_encoding = encodings.last().unwrap();
                                encodings.len() > 1 || last_encoding.header.total > 1
                            }
                            None => false,
                        };

                        if !always_add && value.abs() < FLOAT_TOLERANCE {
                            continue;
                        }

                        add_value_to_encoding(section_axis, local_frame_count, value);
                    }

                    for axis in 0..3 {
                        // TODO: If the animation is delta then it should not be subtracted from the bone data.
                        let value = channel_data.rotation[frame_index][axis] - bone_data.rotation[axis];

                        let section_bone = section_bones.entry(**bone).or_default();
                        let section_axis = &mut section_bone.1[axis];

                        let always_add = match section_axis {
                            Some(encodings) => {
                                let last_encoding = encodings.last().unwrap();
                                encodings.len() > 1 || last_encoding.header.total > 1
                            }
                            None => false,
                        };

                        if !always_add && value.abs() < FLOAT_TOLERANCE {
                            continue;
                        }

                        add_value_to_encoding(section_axis, local_frame_count, value);
                    }
                }

                local_frame_count += 1;
            }

            let mut section_data = Vec::new();

            for (bone, (position, rotation)) in section_bones {
                if position.iter().all(Option::is_none) && rotation.iter().all(Option::is_none) {
                    continue;
                }

                section_data.push(ProcessedAnimatedBoneData {
                    bone: bone.try_into().unwrap(),
                    position,
                    rotation,
                });
            }

            compressed_animation.sections.push(section_data);
        }

        compressed_animations.push(compressed_animation);
    }

    if compressed_animations.len() > (i16::MAX as usize) + 1 {
        return Err(ProcessingAnimationError::TooManyAnimations);
    }

    Ok(ProcessedAnimationData {
        processed_animations: compressed_animations,
        animation_scales,
    })
}

/// Convert channel keyframes to a continuous set of values.
fn bake_channel_keyframes<T: Copy>(channel: &[ImportKeyFrame<T>], frame_count: usize, default: T) -> Vec<T> {
    let mut baked_channel = Vec::with_capacity(frame_count);

    for frame in 0..frame_count {
        if let Some(keyframe) = channel.iter().find(|keyframe| keyframe.frame == frame) {
            baked_channel.push(keyframe.value);
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
