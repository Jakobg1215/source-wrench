use std::collections::HashMap;

use crate::{
    import::ImportedFileData,
    utilities::mathematics::{Matrix, Quaternion, Vector3},
};

use super::ProcessingDataError;

pub fn create_bone_table(import: &HashMap<String, ImportedFileData>) -> Result<BoneTable, ProcessingDataError> {
    let mut bone_table = BoneTable::new();

    for file_data in import.values() {
        for bone in &file_data.skeleton {
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

                if bone_table.is_same_hierarchy(&bone.name, &parent.unwrap().name) {
                    continue;
                }

                return Err(ProcessingDataError::BoneHierarchyError);
            }

            // Check if the parent is in the table.
            if parent.is_some() && !bone_table.is_bone_in_table(&parent.unwrap().name) {
                return Err(ProcessingDataError::BoneHierarchyError);
            }

            // Add the bone to the table.
            let mut new_bone = GlobalBone::new(bone.name.clone(), bone.position, bone.orientation);

            if parent.is_none() {
                new_bone.bone_to_pose = Matrix::new(bone.orientation, bone.position);
            } else {
                let parent = parent.unwrap();
                let parent_matrix = Matrix::new(parent.orientation, parent.position);
                let parent_matrix_transpose = parent_matrix.transpose();

                new_bone.bone_to_pose = parent_matrix_transpose.concatenate(&Matrix::new(bone.orientation, bone.position));
            }

            let bone_index = bone_table.add_bone(new_bone, parent.map(|parent| parent.name.as_str()))?;
            bone_table.mapped_bones.insert(bone.name.clone(), bone_index);
        }
    }

    Ok(bone_table)
}

pub struct BoneTable {
    bones: Vec<GlobalBone>,
    mapped_bones: HashMap<String, usize>,
}

impl BoneTable {
    fn new() -> Self {
        Self {
            bones: Vec::new(),
            mapped_bones: HashMap::new(),
        }
    }
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

    pub fn get_bone_index(&self, bone_name: &str) -> &usize {
        // UNWRAP: Bone should exist from bone table creation.
        self.mapped_bones.get(bone_name).unwrap()
    }

    pub fn get_mut(&mut self, index: usize) -> &mut GlobalBone {
        // UNWRAP: Bone should exist from bone table creation.
        self.bones.get_mut(index).unwrap()
    }
}

pub struct GlobalBone {
    name: String,
    parent: Option<usize>,
    collapsible: bool,
    pub position: Vector3,
    pub orientation: Quaternion,
    bone_to_pose: Matrix,
    pub position_scale: Vector3,
    pub rotation_scale: Vector3,
}

impl GlobalBone {
    pub fn new(name: String, position: Vector3, orientation: Quaternion) -> Self {
        Self {
            name,
            parent: None,
            collapsible: true,
            position,
            orientation,
            bone_to_pose: Matrix::identity(),
            position_scale: Vector3::one(),
            rotation_scale: Vector3::one(),
        }
    }
}
