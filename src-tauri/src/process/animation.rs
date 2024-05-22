use std::collections::HashMap;

use crate::{
    import::{ImportedBoneAnimation, ImportedFile},
    input::ImputedCompilationData,
    utilities::{
        logging::{log, LogLevel},
        mathematics::Vector3,
    },
};

use super::{
    bones::BoneTable,
    structures::{ProcessedAnimatedBoneData, ProcessedAnimation, ProcessedAnimationPosition, ProcessedAnimationRotation, ProcessedSequence},
    ProcessingDataError, FLOAT_TOLERANCE,
};

struct MappedAnimation {
    name: String,
    frame_count: usize,
    bones: Vec<MappedBoneAnimation>,
}

impl MappedAnimation {
    fn new(name: String) -> Self {
        Self {
            name,
            frame_count: 0,
            bones: Vec::new(),
        }
    }
}

struct MappedBoneAnimation {
    bone_index: usize,
    frames: Vec<ImportedBoneAnimation>,
}

impl MappedBoneAnimation {
    fn new(bone_index: usize) -> Self {
        Self {
            bone_index,
            frames: Vec::new(),
        }
    }
}

pub fn process_animations(
    input: &ImputedCompilationData,
    import: &HashMap<String, ImportedFile>,
    bone_table: &mut BoneTable,
) -> Result<Vec<ProcessedAnimation>, ProcessingDataError> {
    let mapped_animations = map_animations_to_table(&input, &import)?;

    compress_animations(input, bone_table, mapped_animations)
}

/// Takes all animations and converts the local bones to the bone table bones.
fn map_animations_to_table(input: &ImputedCompilationData, import: &HashMap<String, ImportedFile>) -> Result<Vec<MappedAnimation>, ProcessingDataError> {
    let mut mapped_animations = Vec::with_capacity(input.animations.len());

    for input_animation in &input.animations {
        let mut mapped_animation = MappedAnimation::new(input_animation.name.clone());
        let imported_file = import.get(&input_animation.source_file).expect("Source File Not Found!");
        mapped_animation.frame_count = imported_file.get_frame_count();

        for (bone_index, animation_data) in imported_file.animation.iter().enumerate() {
            let mapped_bone = imported_file.remapped_bones.get(&bone_index).expect("Mapped Bone Not Found!");
            let mut mapped_bone_animation = MappedBoneAnimation::new(*mapped_bone);

            for (frame, animation_key) in animation_data {
                // TODO: This is a waste of memory and should be changed.
                while *frame != mapped_bone_animation.frames.len() {
                    // TODO: This should get the bone data from the table.
                    let pervious_frame = mapped_bone_animation.frames.last().expect("No First Frame!");
                    mapped_bone_animation.frames.push(*pervious_frame);
                }

                mapped_bone_animation.frames.push(*animation_key);
            }

            mapped_animation.bones.push(mapped_bone_animation);
        }
        mapped_animations.push(mapped_animation);
    }
    Ok(mapped_animations)
}

/// This compresses all animations.
/// If the animation is not used then its ignored.
fn compress_animations(
    input: &ImputedCompilationData,
    bone_table: &mut BoneTable,
    animations: Vec<MappedAnimation>,
) -> Result<Vec<ProcessedAnimation>, ProcessingDataError> {
    let mut processed_animations = Vec::new();

    for mapped_animation in animations {
        let is_used = input.sequences.iter().any(|sequence| sequence.animation == mapped_animation.name);

        if !is_used {
            log(format!("Animation \"{}\" Not Used!", mapped_animation.name), LogLevel::Warn);
            continue;
        }

        let mut processed_animation = ProcessedAnimation::new(mapped_animation.name);
        processed_animation.frame_count = mapped_animation.frame_count;

        // Get Bone Scale
        for mapped_bone in &mapped_animation.bones {
            let bone = bone_table.get_mut(mapped_bone.bone_index).expect("Mapped Bone Not Found!");
            // TODO: Make this get the best scale values for best quality.
            bone.position_scale = Vector3::new(1.0 / 32.0, 1.0 / 32.0, 1.0 / 32.0);
            bone.rotation_scale = Vector3::new(1.0 / 32.0, 1.0 / 32.0, 1.0 / 32.0);
        }

        for mut mapped_bone in mapped_animation.bones {
            let mut processed_bone = ProcessedAnimatedBoneData::new(mapped_bone.bone_index);
            let bone = bone_table.get_mut(mapped_bone.bone_index).expect("Mapped Bone Not Found!");

            if mapped_bone.frames.len() == 1 {
                // UNWRAP: We know it exist as length is one.
                let animation_data = mapped_bone.frames.pop().unwrap();

                if (animation_data.position - bone.position).sum() > FLOAT_TOLERANCE {
                    // TODO: If the animation is delta then it should just pass the raw animated position.
                    processed_bone.position = Some(ProcessedAnimationPosition::Raw(animation_data.position - bone.position));
                }

                if (animation_data.orientation.to_angles() - bone.orientation.to_angles()).sum() > FLOAT_TOLERANCE {
                    // TODO: If the animation is delta then it should just pass the raw animated position.
                    processed_bone.rotation = Some(ProcessedAnimationRotation::Raw(
                        (animation_data.orientation.to_angles() - bone.orientation.to_angles()).to_quaternion(),
                    ));
                }

                processed_animation.bones.push(processed_bone);
                continue;
            }

            for _animation_data in mapped_bone.frames {
                todo!("Write Compression Of Animations")
            }
        }

        processed_animations.push(processed_animation);
    }

    Ok(processed_animations)
}

pub fn process_sequences(input: &ImputedCompilationData, animations: &Vec<ProcessedAnimation>) -> Result<Vec<ProcessedSequence>, ProcessingDataError> {
    let mut processed_sequences = Vec::new();

    for sequence in &input.sequences {
        let mut processed_sequence = ProcessedSequence::new(sequence.name.clone());

        let animation_index = animations.iter().position(|animation| animation.name == sequence.animation);

        match animation_index {
            Some(index) => processed_sequence.animations.push(index),
            None => return Err(ProcessingDataError::SequenceAnimationNotFound),
        };

        processed_sequences.push(processed_sequence);
    }

    Ok(processed_sequences)
}
