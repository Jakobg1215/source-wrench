use indexmap::{IndexMap, IndexSet};
use thiserror::Error as ThisError;

use crate::{
    import::FileManager,
    input::ImputedCompilationData,
    utilities::{
        logging::{log, LogLevel},
        mathematics::{Matrix3, Matrix4, Vector3},
    },
};

use super::{ProcessedBone, ProcessedBoneData, ProcessedBoneFlags};

#[derive(Debug, ThisError)]
pub enum ProcessingBoneError {
    #[error("No Animation File Selected")]
    NoFileSource,
    #[error("Animation File Source Not Loaded")]
    FileSourceNotLoaded,
    #[error("Model Has Too Many Bone")]
    TooManyBones,
}

pub fn process_bones(input: &ImputedCompilationData, import: &FileManager) -> Result<ProcessedBoneData, ProcessingBoneError> {
    // Load all source files to a bone table.
    let mut remapped_files = IndexSet::with_capacity(import.loaded_file_count());
    let mut source_bone_table: IndexMap<String, ProcessedBone> = IndexMap::new();

    for imputed_body_group in &input.body_groups {
        for imputed_model in &imputed_body_group.models {
            let source_file_path = imputed_model.source_file_path.as_ref().ok_or(ProcessingBoneError::NoFileSource)?;

            if remapped_files.contains(source_file_path) {
                continue;
            }

            let imported_file = import.get_file_data(source_file_path).ok_or(ProcessingBoneError::FileSourceNotLoaded)?;

            for (import_bone_index, (import_bone_name, import_bone)) in imported_file.skeleton.iter().enumerate() {
                let mut bone_flags = ProcessedBoneFlags::default();
                for (enabled_part_index, enabled_part) in imputed_model.enabled_source_parts.iter().enumerate() {
                    if !enabled_part {
                        continue;
                    }

                    let (_, import_part) = imported_file.parts.get_index(enabled_part_index).unwrap();

                    for vertex in &import_part.vertices {
                        if vertex.links.contains_key(&import_bone_index) {
                            bone_flags.insert(ProcessedBoneFlags::USED_BY_VERTEX);
                        }
                    }
                }

                if let Some(global_bone) = source_bone_table.get_mut(import_bone_name) {
                    global_bone.flags.insert(bone_flags);
                    continue;
                }

                // TODO: Validate the data and not unwrap
                let import_bone_parent = import_bone.parent.map(|parent_index| {
                    source_bone_table
                        .get_index_of(imported_file.skeleton.get_index(parent_index).map(|(parent_name, _)| parent_name).unwrap())
                        .unwrap()
                });

                let source_transform = Matrix4::new(Matrix3::from_up_forward(imported_file.up, imported_file.forward), Vector3::default());
                let bone_matrix = Matrix4::new(import_bone.orientation.to_matrix(), import_bone.position);
                let bone_transform = if import_bone_parent.is_none() {
                    source_transform.inverse() * bone_matrix
                } else {
                    bone_matrix
                };

                source_bone_table.insert(
                    import_bone_name.clone(),
                    ProcessedBone {
                        parent: import_bone_parent,
                        position: bone_transform.translation(),
                        rotation: bone_transform.rotation().to_angles(),
                        flags: bone_flags,
                        ..Default::default()
                    },
                );
            }

            remapped_files.insert(source_file_path);
        }
    }

    for imputed_animation in &input.animations {
        let source_file_path = imputed_animation.source_file_path.as_ref().ok_or(ProcessingBoneError::NoFileSource)?;

        if remapped_files.contains(source_file_path) {
            continue;
        }

        let imported_file = import.get_file_data(source_file_path).ok_or(ProcessingBoneError::FileSourceNotLoaded)?;

        for (import_bone_name, import_bone) in &imported_file.skeleton {
            if source_bone_table.contains_key(import_bone_name) {
                continue;
            }

            // TODO: Validate the data and not unwrap
            let import_bone_parent = import_bone.parent.map(|parent_index: usize| {
                source_bone_table
                    .get_index_of(imported_file.skeleton.get_index(parent_index).map(|(parent_name, _)| parent_name).unwrap())
                    .unwrap()
            });

            let source_transform = Matrix4::new(Matrix3::from_up_forward(imported_file.up, imported_file.forward), Vector3::default());
            let bone_matrix = Matrix4::new(import_bone.orientation.to_matrix(), import_bone.position);
            let bone_transform = if import_bone_parent.is_none() {
                source_transform.inverse() * bone_matrix
            } else {
                bone_matrix
            };

            source_bone_table.insert(
                import_bone_name.clone(),
                ProcessedBone {
                    parent: import_bone_parent,
                    position: bone_transform.translation(),
                    rotation: bone_transform.rotation().to_angles(),
                    ..Default::default()
                },
            );
        }

        remapped_files.insert(source_file_path);
    }

    log(format!("Model uses {} source bones.", source_bone_table.len()), LogLevel::Debug);

    // Generate all pose matrixes for source bones.
    for bone_index in 0..source_bone_table.len() {
        if let Some(parent_pose) = source_bone_table[bone_index].parent.map(|index| source_bone_table[index].pose) {
            let bone = &mut source_bone_table[bone_index];
            let pose_matrix = parent_pose * Matrix4::new(bone.rotation, bone.position);
            bone.pose = pose_matrix;
            continue;
        }

        let bone = &mut source_bone_table[bone_index];
        bone.pose = Matrix4::new(bone.rotation, bone.position);
    }

    // TODO: Tag bones from input data

    // TODO: Enforce skeleton hierarchy

    let mut collapsed_bone_table = IndexMap::with_capacity(source_bone_table.len());
    let mut collapsed_remap = Vec::with_capacity(source_bone_table.len());
    let mut collapsed_count = 0;
    for (bone_name, bone_data) in source_bone_table {
        if !bone_data.flags.is_empty() {
            collapsed_bone_table.insert(bone_name, bone_data);
            collapsed_remap.push((false, Some(collapsed_bone_table.len() - 1)));
            continue;
        }

        collapsed_remap.push((true, bone_data.parent));
        collapsed_count += 1;
        log(format!("Collapsed \"{}\"!", bone_name), LogLevel::Verbose);
    }
    log(format!("Collapsed {} bones.", collapsed_count), LogLevel::Debug);

    if collapsed_bone_table.len() > (i8::MAX as usize) + 1 {
        return Err(ProcessingBoneError::TooManyBones);
    }

    // Remap bones parents.
    for (_, bone_data) in &mut collapsed_bone_table {
        let old_parent = match bone_data.parent {
            Some(parent) => parent,
            None => continue,
        };

        let mut new_parent = Some(old_parent);

        loop {
            match collapsed_remap[new_parent.unwrap()] {
                (true, None) => {
                    new_parent = None;
                    break;
                }
                (false, None) => {
                    new_parent = None;
                    break;
                }
                (false, Some(parent)) => {
                    new_parent = Some(parent);
                    break;
                }
                (true, Some(parent)) => new_parent = Some(parent),
            };
        }

        bone_data.parent = new_parent;
    }

    // Update bones local rotation and location
    for bone_index in 0..collapsed_bone_table.len() {
        if let Some(parent_pose) = collapsed_bone_table[bone_index].parent.map(|index| collapsed_bone_table[index].pose) {
            let bone = &mut collapsed_bone_table[bone_index];
            let local_pose = parent_pose.inverse() * bone.pose;
            bone.rotation = local_pose.rotation().to_angles();
            bone.position = local_pose.translation();
            continue;
        }

        let bone = &mut collapsed_bone_table[bone_index];
        bone.rotation = bone.pose.rotation().to_angles();
        bone.position = bone.pose.translation();
    }

    let mut sorted_bones_by_name: Vec<u8> = (0..collapsed_bone_table.len() as u8).collect();
    sorted_bones_by_name.sort_by(|from, to| {
        let bone_from = collapsed_bone_table.get_index(*from as usize).unwrap().0;
        let bone_to = collapsed_bone_table.get_index(*to as usize).unwrap().0;
        bone_from.cmp(bone_to)
    });

    Ok(ProcessedBoneData {
        processed_bones: collapsed_bone_table,
        sorted_bones_by_name,
    })
}
