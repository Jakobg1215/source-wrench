use std::collections::HashMap;

use crate::{
    import::ImportedFileData,
    process::structures::ProcessedBone,
    utilities::mathematics::{Quaternion, Vector3},
};

use super::{structures::ProcessedBoneData, ProcessingDataError};

#[derive(Default)]
pub struct BoneTable {
    bones: Vec<GlobalBone>,
}

impl BoneTable {
    /// Add a bone to the table.
    /// Returns the index of the bone in the table.
    /// If the bone is a root bone, the parent is None.
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

    /// Check if the bone is already in the table.
    fn is_bone_in_table(&mut self, name: &str) -> bool {
        self.bones.iter().any(|bone| bone.name == name)
    }

    /// Check if the bone has the same parent as the table.
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

    /// Check if the bone has a parent.
    fn has_parent(&self, name: &str) -> bool {
        let bone = match self.bones.iter().find(|bone| bone.name == name) {
            Some(bone) => bone,
            None => return false,
        };

        bone.parent.is_some()
    }

    pub fn size(&self) -> usize {
        self.bones.len()
    }

    pub fn get(&self, index: usize) -> Option<&GlobalBone> {
        self.bones.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut GlobalBone> {
        self.bones.get_mut(index)
    }
}

pub struct GlobalBone {
    pub name: String,
    pub parent: Option<usize>,
    pub collapsible: bool,
    pub position: Vector3,
    pub orientation: Quaternion,
    pub position_scale: Vector3,
    pub rotation_scale: Vector3,
}

impl GlobalBone {
    pub fn new(name: String, position: Vector3, orientation: Quaternion) -> Self {
        Self {
            name,
            parent: None,
            collapsible: false,
            position,
            orientation,
            position_scale: Vector3::one(),
            rotation_scale: Vector3::one(),
        }
    }
}

pub fn process_bone_table(bone_table: &BoneTable) -> ProcessedBoneData {
    let mut processed_bones = ProcessedBoneData::default();

    for table_bone in bone_table.bones.iter() {
        if table_bone.collapsible {
            continue;
        }

        let mut processed_bone = ProcessedBone::new(table_bone.name.clone());
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

pub fn create_bone_table(import: &mut HashMap<String, ImportedFileData>) -> Result<BoneTable, ProcessingDataError> {
    let mut bone_table = BoneTable::default();

    for file_data in import.values_mut() {
        file_data.remapped_bones.reserve(file_data.skeleton.len());

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
            let new_bone = GlobalBone::new(bone.name.clone(), bone.position, bone.orientation);

            let bone_table_index = bone_table.add_bone(new_bone, parent.map(|parent| parent.name.as_str()))?;

            // Add bone to remap table.
            file_data.remapped_bones.insert(index, bone_table_index);
        }
    }

    Ok(bone_table)
}
