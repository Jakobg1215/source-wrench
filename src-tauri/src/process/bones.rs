use std::sync::Arc;

use tauri::State;
use thiserror::Error as ThisError;

use crate::{
    import::{FileManager, ImportFileData},
    input::ImputedCompilationData,
    utilities::mathematics::Matrix4,
};

use super::{ProcessedBone, ProcessedBoneData};

#[derive(Debug, ThisError)]
pub enum ProcessingBoneError {}

pub fn process_bones(input: &ImputedCompilationData, import: &State<FileManager>) -> Result<ProcessedBoneData, ProcessingBoneError> {
    let mut bone_table = ProcessedBoneData::default();

    for (imported_file, imported_data) in import.files.lock().unwrap().iter() {
        let mut remapped_bones = Vec::with_capacity(imported_data.skeleton.len());

        for (import_bone_index, import_bone) in imported_data.skeleton.iter().enumerate() {
            if let Some(global_bone_index) = find_global_bone_index(&bone_table, &import_bone.name) {
                remapped_bones.push(global_bone_index);
                continue;
            }

            if bone_should_be_collapsed(input, imported_data, import_bone_index) {
                remapped_bones.push(match import_bone.parent {
                    Some(parent_index) => remapped_bones[parent_index],
                    None => 0,
                });
                continue;
            }

            let processed_parent = import_bone.parent.map(|parent_index| remapped_bones[parent_index]);

            let pose_bone = match processed_parent {
                Some(parent_index) => {
                    let parent_matrix = &bone_table.processed_bones[parent_index].pose;
                    *parent_matrix * Matrix4::new(import_bone.position, import_bone.orientation.to_matrix())
                }
                None => Matrix4::new(import_bone.position, import_bone.orientation.to_matrix()),
            };

            let processed_bone = ProcessedBone {
                name: import_bone.name.clone(),
                parent: processed_parent,
                position: import_bone.position,
                rotation: import_bone.orientation.to_angles(),
                pose: pose_bone,
                ..Default::default()
            };

            remapped_bones.push(bone_table.processed_bones.len());
            bone_table.processed_bones.push(processed_bone);
        }

        bone_table.remapped_bones.insert(imported_file.clone(), remapped_bones);
    }

    let mut processed_bone_names = bone_table
        .processed_bones
        .iter()
        .enumerate()
        .map(|(index, bone)| (index, bone.name.clone()))
        .collect::<Vec<(usize, String)>>();
    processed_bone_names.sort_by(|(_, a), (_, b)| a.to_lowercase().cmp(&b.to_lowercase()));

    bone_table.sorted_bones_by_name = processed_bone_names.iter().map(|(index, _)| *index).collect();

    Ok(bone_table)
}

fn find_global_bone_index(bone_table: &ProcessedBoneData, bone_name: &str) -> Option<usize> {
    bone_table.processed_bones.iter().position(|processed_bone| processed_bone.name == bone_name)
}

fn bone_should_be_collapsed(_input: &ImputedCompilationData, imported_data: &Arc<ImportFileData>, bone_index: usize) -> bool {
    for import_animation in &imported_data.animations {
        debug_assert!(!import_animation.channels.is_empty(), "Import Animation Channels Are Empty!");
        if import_animation.frame_count == 1 {
            continue;
        }

        for import_channel in &import_animation.channels {
            if import_channel.bone != bone_index {
                continue;
            }

            if import_channel.position.len() <= 1 && import_channel.orientation.len() <= 1 {
                continue;
            }

            return false;
        }
    }

    for import_part in &imported_data.parts {
        for import_vertex in &import_part.vertices {
            for import_link in &import_vertex.links {
                if import_link.bone == bone_index {
                    return false;
                }
            }
        }
    }

    true
}
