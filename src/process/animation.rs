use indexmap::IndexMap;
use thiserror::Error as ThisError;

use crate::{
    import::FileManager,
    input::ImputedCompilationData,
    utilities::{
        logging::{log, LogLevel},
        mathematics::{Matrix3, Matrix4, Quaternion, Vector3},
    },
};

use super::{ProcessedAnimatedBoneData, ProcessedAnimation, ProcessedAnimationData, ProcessedBoneData};

#[derive(Debug, ThisError)]
pub enum ProcessingAnimationError {
    #[error("No Animation File Selected")]
    NoFileSource,
    #[error("Animation File Source Not Loaded")]
    FileSourceNotLoaded,
    #[error("Model Has Too Many Animations")]
    TooManyAnimations,
}

pub fn process_animations(
    input: &ImputedCompilationData,
    import: &FileManager,
    processed_bone_data: &ProcessedBoneData,
) -> Result<ProcessedAnimationData, ProcessingAnimationError> {
    struct ChannelData {
        position: Vec<Vector3>,
        rotation: Vec<Quaternion>,
    }

    let mut remapped_animations = Vec::with_capacity(input.animations.len());
    let mut processed_animations = Vec::new();
    let mut model_frame_count = 0;
    for (imputed_animation_index, (_, imputed_animation)) in input.animations.iter().enumerate() {
        remapped_animations.push(processed_animations.len());

        // Check if the animation is used in any sequence.
        if !input.sequences.iter().any(|(_, sequence)| {
            sequence
                .animations
                .iter()
                .any(|row| row.iter().any(|&used_animation| used_animation == imputed_animation_index))
        }) {
            log(format!("Animation \"{}\" Not Used!", imputed_animation.name), LogLevel::Warn);
            continue;
        }

        // Gather imported animation data.
        let imported_file = import
            .get_file_data(imputed_animation.source_file_path.as_ref().ok_or(ProcessingAnimationError::NoFileSource)?)
            .ok_or(ProcessingAnimationError::FileSourceNotLoaded)?;
        let (_, imported_animation) = imported_file.animations.get_index(imputed_animation.source_animation).unwrap();

        let frame_count = imported_animation.frame_count.get();
        model_frame_count += frame_count;

        let mut animation_channels = IndexMap::new();
        for (bone, channel) in &imported_animation.channels {
            let (import_bone_name, import_bone_data) = imported_file.skeleton.get_index(*bone).unwrap();

            let (mapped_index, _) = match processed_bone_data.processed_bones.get_full(import_bone_name) {
                Some((index, _, data)) => (index, data),
                None => continue,
            };

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

            let mut position_channel = bake_channel_keyframes(&channel.position, frame_count, import_bone_data.position);
            let mut rotation_channel = bake_channel_keyframes(&channel.rotation, frame_count, import_bone_data.orientation.normalize());

            if import_bone_data.parent.is_none() {
                let source_transform = Matrix4::new(Matrix3::from_up_forward(imported_file.up, imported_file.forward), Vector3::default());

                for frame in 0..frame_count {
                    let key_matrix = Matrix4::new(rotation_channel[frame].to_matrix(), position_channel[frame]);
                    let key_transform = source_transform.inverse() * key_matrix;
                    position_channel[frame] = key_transform.translation();
                    rotation_channel[frame] = key_transform.rotation().to_quaternion();
                }
            }

            // TODO: Translate channel data to bone table.

            animation_channels.insert(
                mapped_index,
                ChannelData {
                    position: position_channel,
                    rotation: rotation_channel,
                },
            );
        }

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

            let mut section_data = Vec::with_capacity(animation_channels.len());
            for (index_bone, channel_data) in &animation_channels {
                let bone = &processed_bone_data.processed_bones[*index_bone];
                let mut delta_position = Vec::with_capacity(section_frame_count);
                let mut delta_rotation = Vec::with_capacity(section_frame_count);

                // TODO: If animation is delta then skip subtracting from bone
                for frame in section_frame_start..=section_frame_end {
                    delta_position.push(channel_data.position[frame] - bone.position);
                    delta_rotation.push(channel_data.rotation[frame].to_angles() - bone.rotation);
                }

                section_data.push(ProcessedAnimatedBoneData {
                    bone: (*index_bone).try_into().unwrap(),
                    raw_position: channel_data.position[section_frame_start..=section_frame_end].to_vec(),
                    raw_rotation: channel_data.rotation[section_frame_start..=section_frame_end].to_vec(),
                    delta_position,
                    delta_rotation,
                });
            }

            processed_animation.sections.push(section_data);
        }

        processed_animations.push(processed_animation);
    }

    log(format!("Model uses {} frames.", model_frame_count), LogLevel::Debug);

    if processed_animations.len() > (i16::MAX as usize + 1) {
        return Err(ProcessingAnimationError::TooManyAnimations);
    }

    let mut animation_scales = vec![(Vector3::default(), Vector3::default()); processed_bone_data.processed_bones.len()];
    for processed_animation in &processed_animations {
        for sections in &processed_animation.sections {
            for section in sections {
                for position in &section.delta_position {
                    for axis in 0..3 {
                        let value = position[axis].abs();
                        if value > animation_scales[section.bone as usize].0[axis] {
                            animation_scales[section.bone as usize].0[axis] = value;
                        }
                    }
                }

                for rotation in &section.delta_rotation {
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
        remapped_animations,
    })
}
