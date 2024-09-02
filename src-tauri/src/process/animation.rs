use tauri::State;

use crate::{
    import::FileManager,
    input::ImputedCompilationData,
    utilities::{
        logging::{log, LogLevel},
        mathematics::Vector3,
    },
};

use super::{
    bones::BoneTable, ProcessedAnimatedBoneData, ProcessedAnimation, ProcessedAnimationPosition, ProcessedAnimationRotation, ProcessedSequence,
    ProcessingDataError, FLOAT_TOLERANCE,
};

pub fn process_animations(
    input: &ImputedCompilationData,
    import: &State<FileManager>,
    bone_table: &mut BoneTable,
) -> Result<Vec<ProcessedAnimation>, ProcessingDataError> {
    let mut processed_animations = Vec::new();

    for input_animation in &input.animations {
        let imported_file = import.get_file(&input_animation.file_source).expect("Source File Not Found!");
        let imported_animation = if imported_file.animations.len() == 1 {
            imported_file.animations.first().unwrap()
        } else {
            imported_file
                .animations
                .iter()
                .find(|anim| anim.name == input_animation.name)
                .expect("Animation Not Found!")
        };

        let is_used = input
            .sequences
            .iter()
            .any(|sequence| sequence.animations.iter().any(|animation| animation == &input_animation.name));

        if !is_used {
            log(format!("Animation \"{}\" Not Used!", input_animation.name), LogLevel::Warn);
            continue;
        }

        let mut processed_animation = ProcessedAnimation {
            name: input_animation.name.clone(),
            frame_count: imported_animation.frame_count,
            ..Default::default()
        };

        for (_, bone_data) in &mut bone_table.bones {
            // TODO: Make this get the best scale values for best quality.
            bone_data.position_scale = Vector3::new(1.0 / 32.0, 1.0 / 32.0, 1.0 / 32.0);
            bone_data.rotation_scale = Vector3::new(1.0 / 32.0, 1.0 / 32.0, 1.0 / 32.0);
        }

        let mapped_bone = bone_table.remapped_bones.get(&input_animation.file_source).expect("Source File Not Remapped!");

        for channel in &imported_animation.channels {
            let mut processed_bone_data = ProcessedAnimatedBoneData {
                bone: *mapped_bone.get(&channel.bone).expect("Mapped Bone Not Found!"),
                ..Default::default()
            };

            let (_, bone_data) = bone_table.bones.get_index(processed_bone_data.bone).expect("Bone Not Found!");

            if imported_animation.frame_count == 1 {
                let position_frame = channel.position.first().expect("No Position First Frame!");
                if (position_frame.value - bone_data.position).sum() > FLOAT_TOLERANCE {
                    // TODO: If the animation is delta then it should just pass the raw animated position.
                    processed_bone_data.position = Some(ProcessedAnimationPosition::Raw(position_frame.value - bone_data.position));
                }

                let rotation_frame = channel.orientation.first().expect("No Rotation First Frame!");
                if (rotation_frame.value.to_angles() - bone_data.orientation.to_angles()).sum() > FLOAT_TOLERANCE {
                    processed_bone_data.rotation = Some(ProcessedAnimationRotation::Raw(
                        (rotation_frame.value.to_angles() - bone_data.orientation.to_angles()).to_quaternion(),
                    ));
                }

                processed_animation.bones.push(processed_bone_data);
                continue;
            }

            for _frame in 0..imported_animation.frame_count {
                todo!("Write Compression Of Animations")
            }
        }

        processed_animations.push(processed_animation);
    }

    Ok(processed_animations)
}

pub fn process_sequences(input: &ImputedCompilationData, animations: &[ProcessedAnimation]) -> Result<Vec<ProcessedSequence>, ProcessingDataError> {
    let mut processed_sequences = Vec::new();

    for sequence in &input.sequences {
        let mut processed_sequence = ProcessedSequence {
            name: sequence.name.clone(),
            ..Default::default()
        };

        for sequence_animation in &sequence.animations {
            let animation_index = animations.iter().position(|animation| &animation.name == sequence_animation);

            match animation_index {
                Some(index) => processed_sequence.animations.push(index),
                None => return Err(ProcessingDataError::SequenceAnimationNotFound),
            };
        }

        processed_sequences.push(processed_sequence);
    }

    Ok(processed_sequences)
}
