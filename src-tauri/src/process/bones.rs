use std::collections::HashMap;

use indexmap::IndexMap;
use tauri::State;

use crate::{
    import::FileManager,
    utilities::mathematics::{Quaternion, Vector3},
};

use super::{ProcessedBone, ProcessedBoneData, ProcessingDataError};

#[derive(Debug, Default)]
pub struct BoneTable {
    pub bones: IndexMap<String, GlobalBone>,
    pub remapped_bones: HashMap<String, HashMap<usize, usize>>,
}

#[derive(Debug, Default)]
pub struct GlobalBone {
    pub parent: Option<usize>,
    pub collapsible: bool,
    pub position: Vector3,
    pub orientation: Quaternion,
    pub position_scale: Vector3,
    pub rotation_scale: Vector3,
}

pub fn create_bone_table(import: &State<FileManager>) -> Result<BoneTable, ProcessingDataError> {
    let mut bone_table = BoneTable::default();

    for (file_name, file_data) in import.files.lock().unwrap().iter() {
        let mut remapped_bones = HashMap::new();

        for (bone_index, import_bone) in file_data.skeleton.iter().enumerate() {
            if let Some(mapped_index) = bone_table.bones.get_index_of(&import_bone.name) {
                if let Some(parent_mapped_index) = import_bone.parent {
                    if !remapped_bones.contains_key(&parent_mapped_index) {
                        return Err(ProcessingDataError::BoneHierarchyError);
                    }
                }
                remapped_bones.insert(bone_index, mapped_index);
                continue;
            }

            // Check if the parent is in the table.
            if let Some(parent_index) = import_bone.parent {
                if !remapped_bones.contains_key(&parent_index) {
                    return Err(ProcessingDataError::BoneHierarchyError);
                }
            }

            let new_bone = GlobalBone {
                position: import_bone.position,
                orientation: import_bone.orientation,
                parent: import_bone.parent.and_then(|parent_index| remapped_bones.get(&parent_index).copied()),
                ..Default::default()
            };

            bone_table.bones.insert(import_bone.name.clone(), new_bone);
            remapped_bones.insert(bone_index, bone_table.bones.len() - 1);
        }

        bone_table.remapped_bones.insert(file_name.clone(), remapped_bones);
    }

    Ok(bone_table)
}

pub fn process_bone_table(bone_table: &BoneTable) -> ProcessedBoneData {
    let mut processed_bones = ProcessedBoneData::default();

    for (bone_name, bone_data) in &bone_table.bones {
        // TODO: Remove collapsed bones.

        let pose = match bone_data.parent {
            Some(index) => {
                let parent = &processed_bones.processed_bones[index];

                let raw_rotation = parent.pose.0.concatenate(bone_data.orientation.to_matrix());
                // TODO: This should be moved to the mathematics library.
                let raw_position = Vector3::new(
                    parent.pose.0[0][0] * bone_data.position.x
                        + parent.pose.0[0][1] * bone_data.position.y
                        + parent.pose.0[0][2] * bone_data.position.z
                        + parent.pose.1.x,
                    parent.pose.0[1][0] * bone_data.position.x
                        + parent.pose.0[1][1] * bone_data.position.y
                        + parent.pose.0[1][2] * bone_data.position.z
                        + parent.pose.1.y,
                    parent.pose.0[2][0] * bone_data.position.x
                        + parent.pose.0[2][1] * bone_data.position.y
                        + parent.pose.0[2][2] * bone_data.position.z
                        + parent.pose.1.z,
                );

                (raw_rotation, raw_position)
            }
            None => (bone_data.orientation.to_matrix(), bone_data.position),
        };

        let processed_bone = ProcessedBone {
            name: bone_name.clone(),
            parent: bone_data.parent,
            position: bone_data.position,
            rotation: bone_data.orientation.to_angles(),
            pose,
            animation_position_scale: bone_data.position_scale,
            animation_rotation_scale: bone_data.rotation_scale,
        };

        processed_bones.processed_bones.push(processed_bone);
    }

    let mut processed_bone_names = processed_bones
        .processed_bones
        .iter()
        .enumerate()
        .map(|(index, bone)| (index, bone.name.clone()))
        .collect::<Vec<(usize, String)>>();
    processed_bone_names.sort_by(|(_, a), (_, b)| a.to_lowercase().cmp(&b.to_lowercase()));

    processed_bones.sorted_bones_by_name = processed_bone_names.iter().map(|(index, _)| *index).collect();

    processed_bones
}
