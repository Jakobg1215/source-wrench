use std::collections::HashMap;

use tauri::State;

use crate::{
    import::FileManager,
    utilities::mathematics::{Quaternion, Vector3},
};

use super::{ProcessedBone, ProcessedBoneData, ProcessingDataError};

#[derive(Default, Debug)]
pub struct BoneTable {
    pub bones: Vec<GlobalBone>,
    pub remapped_bones: HashMap<String, HashMap<usize, usize>>,
}

impl BoneTable {
    fn add_bone(&mut self, mut bone: GlobalBone, parent: Option<&str>) -> Result<usize, ProcessingDataError> {
        let root = match parent {
            Some(parent) => match self.bones.iter().position(|bone| bone.name == parent) {
                Some(index) => Some(index),
                None => return Err(ProcessingDataError::BoneHierarchyError),
            },
            None => None,
        };

        bone.parent = root;

        self.bones.push(bone);

        Ok(self.bones.len() - 1)
    }

    fn is_bone_in_table(&self, name: &str) -> bool {
        self.bones.iter().any(|bone| bone.name == name)
    }

    fn is_same_hierarchy(&self, target: &str, root: &str) -> bool {
        let bone = match self.bones.iter().find(|bone| bone.name == target) {
            Some(bone) => bone,
            None => return false,
        };

        let root = match self.bones.iter().find(|bone| bone.name == root) {
            Some(bone) => bone,
            None => return false,
        };

        return bone.parent == root.parent;
    }

    fn has_parent(&self, name: &str) -> bool {
        let bone = match self.bones.iter().find(|bone| bone.name == name) {
            Some(bone) => bone,
            None => return false,
        };

        bone.parent.is_some()
    }
}

#[derive(Default, Debug)]
pub struct GlobalBone {
    pub name: String,
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
        for (index, bone) in file_data.skeleton.iter().enumerate() {
            let parent = match bone.parent {
                Some(parent) => match file_data.skeleton.get(parent) {
                    Some(parent) => Some(parent),
                    None => None,
                },
                None => None,
            };

            // Check if the bone is already in the table.
            if bone_table.is_bone_in_table(&bone.name) {
                if bone_table.has_parent(&bone.name) && parent.is_none() {
                    return Err(ProcessingDataError::BoneHierarchyError);
                }

                if parent.is_none() {
                    continue;
                }

                if bone_table.is_same_hierarchy(&bone.name, &parent.expect("Parent Should Exist!").name) {
                    continue;
                }

                return Err(ProcessingDataError::BoneHierarchyError);
            }

            // Check if the parent is in the table.
            if parent.is_some() && !bone_table.is_bone_in_table(&parent.expect("Parent Should Exist!").name) {
                return Err(ProcessingDataError::BoneHierarchyError);
            }

            // Add the bone to the table.
            let mut new_bone = GlobalBone::default();
            new_bone.name = bone.name.clone();
            new_bone.position = bone.position;
            new_bone.orientation = bone.orientation;

            let bone_table_index = bone_table.add_bone(new_bone, parent.map(|parent| parent.name.as_str()))?;

            // Add bone to remap table.

            let remap = match bone_table.remapped_bones.get_mut(file_name) {
                Some(remap) => remap,
                None => {
                    bone_table.remapped_bones.insert(file_name.clone(), HashMap::new());
                    bone_table.remapped_bones.get_mut(file_name).unwrap()
                }
            };

            remap.insert(index, bone_table_index);
        }
    }

    Ok(bone_table)
}

pub fn process_bone_table(bone_table: &BoneTable) -> ProcessedBoneData {
    let mut processed_bones = ProcessedBoneData::default();

    for table_bone in bone_table.bones.iter() {
        if table_bone.collapsible {
            continue;
        }

        let mut processed_bone = ProcessedBone::default();
        processed_bone.name = table_bone.name.clone();
        processed_bone.parent = table_bone.parent;
        processed_bone.position = table_bone.position;
        processed_bone.rotation = table_bone.orientation.to_angles();
        processed_bone.animation_position_scale = table_bone.position_scale;
        processed_bone.animation_rotation_scale = table_bone.rotation_scale;

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
