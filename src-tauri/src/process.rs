use std::collections::HashMap;

use thiserror::Error;

use crate::{
    import::ImportedFileData,
    input::CompilationDataInput,
    process::{
        animation::{compress_animations, map_animations_to_table, process_sequences},
        bones::create_bone_table,
        mesh::process_mesh_data,
    },
    utilities::logging::{log, LogLevel},
};

use self::structures::ProcessedData;

mod animation;
mod bones;
mod mesh;
mod structures;

#[derive(Error, Debug)]
pub enum ProcessingDataError {
    #[error("Bone Had Different Hierarchy Than Pose")]
    BoneHierarchyError,
    #[error("Model Has Too Many Bone")]
    TooManyBones,
    #[error("Model Has Too Many Animations")]
    TooManyAnimations,
    #[error("Model Has Too Many Sequences")]
    TooManySequences,
    #[error("Sequence Could Not Find Animation")]
    SequenceAnimationNotFound,
    #[error("Model Has No Sequences")]
    NoSequences,
    #[error("Body Part Name Is Too Long")]
    BodyPartNameTooLong,
    #[error("Vertex Has More That 3 Weight Links")]
    VertHasTooManyLinks,
    #[error("Face Did Not Have 3 Indices")]
    IncompleteFace,
    #[error("Failed To Generated Tangents")]
    FailedTangentGeneration,
}

const FLOAT_TOLERANCE: f64 = 0.000001;

pub fn process(input: CompilationDataInput, import: HashMap<String, ImportedFileData>) -> Result<ProcessedData, ProcessingDataError> {
    log("Creating Bone Table", LogLevel::Debug);
    let mut bone_table = create_bone_table(&import)?;
    log(format!("Model uses {} source bones", bone_table.size()), LogLevel::Verbose);

    if bone_table.size() > u8::MAX as usize {
        // FIXME: This does not take into account collapsed bones!
        // return Err(ProcessingDataError::TooManyBones);
    }

    // TODO: The animations should be put in one function called process animations.
    log("Mapping Animations", LogLevel::Debug);
    let mapped_animations = map_animations_to_table(&input, &import, &bone_table)?;
    log(format!("Mapped {} Animations", mapped_animations.len()), LogLevel::Debug);

    log("Compressing Animations", LogLevel::Debug);
    let processed_animations = compress_animations(&input, &mut bone_table, mapped_animations)?;
    log(format!("Model has {} animations", processed_animations.len()), LogLevel::Verbose);

    if processed_animations.len() > i16::MAX as usize {
        return Err(ProcessingDataError::TooManyAnimations);
    }

    log("Processing Sequences", LogLevel::Debug);
    let processed_sequences = process_sequences(&input, &processed_animations)?;
    log(format!("Model has {} sequences", processed_sequences.len()), LogLevel::Verbose);

    if processed_sequences.len() == 0 {
        return Err(ProcessingDataError::NoSequences);
    }

    if processed_sequences.len() > i32::MAX as usize {
        return Err(ProcessingDataError::TooManySequences);
    }

    log("Processing Mesh Data", LogLevel::Debug);
    let _processed_mesh = process_mesh_data(&input, &import, &bone_table)?;

    todo!()
}
