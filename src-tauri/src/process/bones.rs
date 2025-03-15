use indexmap::{IndexMap, IndexSet};
use tauri::State;
use thiserror::Error as ThisError;

use crate::{
    import::FileManager,
    input::ImputedCompilationData,
    utilities::{
        logging::{log, LogLevel},
        mathematics::Matrix4,
    },
};

use super::{ProcessedBone, ProcessedBoneData, ProcessedBoneFlags};

#[derive(Debug, ThisError)]
pub enum ProcessingBoneError {
    #[error("Animation File Source Not Loaded")]
    FileSourceNotLoaded,
    #[error("Model Has Too Many Bone")]
    TooManyBones,
    #[error("Part Not Found: {0}")]
    PartNotFound(String),
}

pub fn process_bones(input: &ImputedCompilationData, import: &State<FileManager>) -> Result<ProcessedBoneData, ProcessingBoneError> {
    // Load all source files to a bone table.
    let mut remapped_files = IndexSet::with_capacity(import.loaded_file_count());
    let mut source_bone_table: IndexMap<String, ProcessedBone> = IndexMap::new();

    for (_, imputed_body_part) in &input.body_parts {
        for (_, imputed_model) in &imputed_body_part.models {
            if remapped_files.contains(&imputed_model.file_source) {
                continue;
            }

            let imported_file = import.get_file(&imputed_model.file_source).ok_or(ProcessingBoneError::FileSourceNotLoaded)?;

            for (import_bone_index, (import_bone_name, import_bone)) in imported_file.skeleton.iter().enumerate() {
                let mut bone_flags = ProcessedBoneFlags::default();
                for imputed_part_name in &imputed_model.part_names {
                    let import_part = match imported_file.parts.get(imputed_part_name) {
                        Some(part) => part,
                        None => return Err(ProcessingBoneError::PartNotFound(imputed_part_name.clone())),
                    };

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

                source_bone_table.insert(
                    import_bone_name.clone(),
                    ProcessedBone {
                        parent: import_bone_parent,
                        position: import_bone.position,
                        rotation: import_bone.orientation.to_angles(),
                        flags: bone_flags,
                        ..Default::default()
                    },
                );
            }

            remapped_files.insert(imputed_model.file_source.clone());
        }
    }

    for (_, imputed_animation) in &input.animations {
        if remapped_files.contains(&imputed_animation.file_source) {
            continue;
        }

        let imported_file = import
            .get_file(&imputed_animation.file_source)
            .ok_or(ProcessingBoneError::FileSourceNotLoaded)?;

        for (import_bone_name, import_bone) in &imported_file.skeleton {
            if source_bone_table.contains_key(import_bone_name) {
                continue;
            }

            // TODO: Validate the data and not unwrap
            let import_bone_parent = import_bone.parent.map(|parent_index| {
                source_bone_table
                    .get_index_of(imported_file.skeleton.get_index(parent_index).map(|(parent_name, _)| parent_name).unwrap())
                    .unwrap()
            });

            source_bone_table.insert(
                import_bone_name.clone(),
                ProcessedBone {
                    parent: import_bone_parent,
                    position: import_bone.position,
                    rotation: import_bone.orientation.to_angles(),
                    ..Default::default()
                },
            );
        }

        remapped_files.insert(imputed_animation.file_source.clone());
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
    for (bone_name, bone_data) in source_bone_table {
        if !bone_data.flags.is_empty() {
            collapsed_bone_table.insert(bone_name, bone_data);
            collapsed_remap.push((false, Some(collapsed_bone_table.len() - 1)));
            continue;
        }

        collapsed_remap.push((true, bone_data.parent));
        log(format!("Collapsed {}", bone_name), LogLevel::Verbose);
    }

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
