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

    for (file_name, (_, file_data)) in import.files.lock().unwrap().iter() {
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

            let mut new_bone = GlobalBone::default();
            new_bone.position = import_bone.position;
            new_bone.orientation = import_bone.orientation;
            new_bone.parent = import_bone.parent.and_then(|parent_index| remapped_bones.get(&parent_index).copied());

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
        if bone_data.collapsible {
            continue;
        }

        let mut processed_bone = ProcessedBone::default();
        processed_bone.name = bone_name.clone();
        processed_bone.parent = bone_data.parent;
        processed_bone.position = bone_data.position;
        processed_bone.rotation = bone_data.orientation.to_angles();
        processed_bone.animation_position_scale = bone_data.position_scale;
        processed_bone.animation_rotation_scale = bone_data.rotation_scale;

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
