use indexmap::IndexMap;
use tauri::State;
use thiserror::Error as ThisError;

use crate::{
    import::{FileManager, ImportPart},
    input::ImputedCompilationData,
    process::ProcessedRemappedBone,
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
}

pub fn process_bones(input: &ImputedCompilationData, import: &State<FileManager>) -> Result<ProcessedBoneData, ProcessingBoneError> {
    let mut source_bone_table: IndexMap<String, ProcessedBone> = IndexMap::new();
    let mut remapped_files = IndexMap::new();

    for imputed_body_part in &input.body_parts {
        for imputed_model in &imputed_body_part.models {
            if remapped_files.contains_key(&imputed_model.file_source) {
                continue;
            }

            let imported_file = import.get_file(&imputed_model.file_source).ok_or(ProcessingBoneError::FileSourceNotLoaded)?;

            let mut remapped_bones = Vec::with_capacity(imported_file.skeleton.len());

            for (import_bone_index, import_bone) in imported_file.skeleton.iter().enumerate() {
                let bone_flags = create_bone_flags(import_bone_index, &imported_file.parts);

                if let Some((global_bone_index, _, global_bone)) = source_bone_table.get_full_mut(&import_bone.name) {
                    global_bone.flags.insert(bone_flags);
                    remapped_bones.push(ProcessedRemappedBone { index: global_bone_index });
                    continue;
                }

                let processed_parent = import_bone.parent.map(|parent_index| remapped_bones[parent_index].index);

                remapped_bones.push(ProcessedRemappedBone {
                    index: source_bone_table.len(),
                });
                source_bone_table.insert(
                    import_bone.name.clone(),
                    ProcessedBone {
                        parent: processed_parent,
                        position: import_bone.position,
                        rotation: import_bone.orientation.to_angles().normalize(),
                        flags: bone_flags,
                        ..Default::default()
                    },
                );
            }

            remapped_files.insert(imputed_model.file_source.clone(), remapped_bones);
        }
    }

    for imputed_animation in &input.animations {
        if remapped_files.contains_key(&imputed_animation.file_source) {
            continue;
        }

        let imported_file = import
            .get_file(&imputed_animation.file_source)
            .ok_or(ProcessingBoneError::FileSourceNotLoaded)?;

        let mut remapped_bones = Vec::with_capacity(imported_file.skeleton.len());

        for import_bone in &imported_file.skeleton {
            if let Some(global_bone_index) = source_bone_table.get_index_of(&import_bone.name) {
                remapped_bones.push(ProcessedRemappedBone { index: global_bone_index });
                continue;
            }

            let processed_parent = import_bone.parent.map(|parent_index| remapped_bones[parent_index].index);

            remapped_bones.push(ProcessedRemappedBone {
                index: source_bone_table.len(),
            });
            source_bone_table.insert(
                import_bone.name.clone(),
                ProcessedBone {
                    parent: processed_parent,
                    position: import_bone.position,
                    rotation: import_bone.orientation.to_angles().normalize(),
                    ..Default::default()
                },
            );
        }

        remapped_files.insert(imputed_animation.file_source.clone(), remapped_bones);
    }

    log(format!("Model uses {} source bones.", source_bone_table.len()), LogLevel::Debug);

    // TODO: Tag bones from input data

    // TODO: Enforce skeleton hierarchy

    // TODO: Collapse bones

    if source_bone_table.len() > (i8::MAX as usize) + 1 {
        return Err(ProcessingBoneError::TooManyBones);
    }

    // Build bone pose matrices
    for bone_index in 0..source_bone_table.len() {
        let bone = &source_bone_table[bone_index];

        source_bone_table[bone_index].pose = match bone.parent {
            Some(parent_index) => source_bone_table[parent_index].pose * Matrix4::new(bone.position, bone.rotation.to_matrix()),
            None => Matrix4::new(bone.position, bone.rotation.to_matrix()),
        };
    }

    let mut sorted_bones_by_name: Vec<u8> = (0..source_bone_table.len() as u8).collect();
    sorted_bones_by_name.sort_by(|from, to| {
        let bone_from = source_bone_table.get_index(*from as usize).unwrap().0;
        let bone_to = source_bone_table.get_index(*to as usize).unwrap().0;
        bone_from.cmp(bone_to)
    });

    Ok(ProcessedBoneData {
        processed_bones: source_bone_table,
        remapped_bones: remapped_files,
        sorted_bones_by_name,
    })
}

fn create_bone_flags(bone_index: usize, import_parts: &[ImportPart]) -> ProcessedBoneFlags {
    let mut flags = ProcessedBoneFlags::default();

    for import_part in import_parts {
        for vertex in &import_part.vertices {
            for link in &vertex.links {
                if link.bone == bone_index {
                    flags.insert(ProcessedBoneFlags::USED_BY_VERTEX);
                }
            }
        }
    }

    flags
}
