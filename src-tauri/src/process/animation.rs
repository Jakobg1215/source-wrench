use std::collections::HashMap;

use crate::{
    import::ImportedFileData,
    input::CompilationDataInput,
    utilities::{
        logging::{log, LogLevel},
        mathematics::{Quaternion, Vector3},
    },
};

use super::{
    bones::BoneTable,
    structures::{ProcessedAnimatedBoneData, ProcessedAnimationData, ProcessedAnimationPosition, ProcessedAnimationRotation, ProcessedSequenceData},
    ProcessingDataError, FLOAT_TOLERANCE,
};

/// Takes all animations and converts the local bones to the bone table bones.
pub fn map_animations_to_table(
    input: &CompilationDataInput,
    import: &HashMap<String, ImportedFileData>,
    bone_table: &BoneTable,
) -> Result<Vec<MappedAnimation>, ProcessingDataError> {
    let mut mapped_animations = Vec::new();

    for input_animation in &input.animations {
        let mut mapped_animation = MappedAnimation::new(input_animation.name.clone());

        // UNWRAP: The source file should exist from import stage.
        let source_animation = import.get(&input_animation.source_file).unwrap();

        let mut mapped_bones: HashMap<usize, MappedBoneAnimation> = HashMap::new();

        for source_frame in &source_animation.animation {
            for bone_frame in &source_frame.bones {
                let bone = match mapped_bones.get_mut(&bone_frame.bone) {
                    Some(mapped) => mapped,
                    None => {
                        let source_bone = source_animation.get_bone_by_index(bone_frame.bone);
                        mapped_bones.insert(bone_frame.bone, MappedBoneAnimation::new(*bone_table.get_bone_index(&source_bone.name)));
                        let mapped = mapped_bones.get_mut(&bone_frame.bone).unwrap(); // UNWRAP: It inserted from above.
                        mapped.frames.reserve_exact(source_animation.animation.len());
                        mapped
                    }
                };

                bone.frames.push((bone_frame.position, bone_frame.orientation));
            }
        }

        mapped_animation.frame_count = source_animation.animation.len();
        mapped_animation.animation.reserve_exact(mapped_bones.len());

        for (_, bone) in mapped_bones.drain() {
            mapped_animation.animation.push(bone);
        }

        mapped_animations.push(mapped_animation);
    }

    Ok(mapped_animations)
}

/// This compresses all animations.
/// If the animation is not used then its ignored.
pub fn compress_animations(
    input: &CompilationDataInput,
    bone_table: &mut BoneTable,
    animations: Vec<MappedAnimation>,
) -> Result<Vec<ProcessedAnimationData>, ProcessingDataError> {
    let mut processed_animations = Vec::new();

    for mapped_animation in animations {
        let is_used = input.sequences.iter().any(|sequence| sequence.animation == mapped_animation.name);

        if !is_used {
            log(format!("Animation \"{}\" Not Used!", mapped_animation.name), LogLevel::Warn);
            continue;
        }

        let mut processed_animation = ProcessedAnimationData::new(mapped_animation.name);
        processed_animation.frame_count = mapped_animation.frame_count;

        // Get Bone Scale
        for mapped_bone in &mapped_animation.animation {
            let bone = bone_table.get_mut(mapped_bone.bone_index);
            // TODO: Make this get the best scale values for best quality.
            bone.position_scale = Vector3::new(1.0 / 32.0, 1.0 / 32.0, 1.0 / 32.0);
            bone.rotation_scale = Vector3::new(1.0 / 32.0, 1.0 / 32.0, 1.0 / 32.0);
        }

        for mut mapped_bone in mapped_animation.animation {
            let mut processed_bone = ProcessedAnimatedBoneData::new(mapped_bone.bone_index);
            let bone = bone_table.get_mut(mapped_bone.bone_index);

            if mapped_bone.frames.len() == 1 {
                // UNWRAP: We know it exist as length is one.
                let (mapped_position, mapped_rotation) = mapped_bone.frames.pop().unwrap();

                if (mapped_position - bone.position).sum() > FLOAT_TOLERANCE {
                    processed_bone.position = Some(ProcessedAnimationPosition::Raw(mapped_position - bone.position));
                }

                if (mapped_rotation.to_angles() - bone.orientation.to_angles()).sum() > FLOAT_TOLERANCE {
                    processed_bone.rotation = Some(ProcessedAnimationRotation::Raw(mapped_rotation));
                }

                processed_animation.bones.push(processed_bone);

                continue;
            }

            for (mapped_position, mapped_rotation) in mapped_bone.frames {
                todo!()
            }
        }

        processed_animations.push(processed_animation);
    }

    Ok(processed_animations)
}

pub fn process_sequences(input: &CompilationDataInput, animations: &Vec<ProcessedAnimationData>) -> Result<Vec<ProcessedSequenceData>, ProcessingDataError> {
    let mut processed_sequences = Vec::new();

    for sequence in &input.sequences {
        let mut processed_sequence = ProcessedSequenceData::new(sequence.name.clone());

        let animation_index = animations.iter().position(|animation| animation.name == sequence.name);

        match animation_index {
            Some(index) => processed_sequence.animations.push(index),
            None => return Err(ProcessingDataError::SequenceAnimationNotFound),
        };

        processed_sequences.push(processed_sequence);
    }

    Ok(processed_sequences)
}

pub struct MappedAnimation {
    name: String,
    frame_count: usize,
    animation: Vec<MappedBoneAnimation>,
}

impl MappedAnimation {
    fn new(name: String) -> Self {
        Self {
            name,
            frame_count: 0,
            animation: Vec::new(),
        }
    }
}

pub struct MappedBoneAnimation {
    bone_index: usize,
    frames: Vec<(Vector3, Quaternion)>,
}

impl MappedBoneAnimation {
    fn new(bone_index: usize) -> Self {
        Self {
            bone_index,
            frames: Vec::new(),
        }
    }
}
