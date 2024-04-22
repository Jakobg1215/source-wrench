use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display, Formatter},
};

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

const FLOAT_TOLERANCE: f64 = 0.000001;

pub fn process(input: CompilationDataInput, import: HashMap<String, ImportedFileData>) -> Result<ProcessedData, ProcessingDataError> {
    log("Creating Bone Table", LogLevel::Debug);
    let mut bone_table = create_bone_table(&import)?;
    log(format!("Model uses {} source bones", bone_table.size()), LogLevel::Verbose);

    // TODO: The animations should be put in one function called process animations.
    log("Mapping Animations", LogLevel::Debug);
    let mapped_animations = map_animations_to_table(&input, &import, &bone_table)?;
    log(format!("Mapped {} Animations", mapped_animations.len()), LogLevel::Debug);

    log("Compressing Animations", LogLevel::Debug);
    let processed_animations = compress_animations(&input, &mut bone_table, mapped_animations)?;
    log(format!("Model has {} animations", processed_animations.len()), LogLevel::Verbose);

    if processed_animations.len() > i16::MAX as usize {
        return Err(ProcessingDataError::TooManyAnimations(i16::MAX as usize));
    }

    log("Processing Sequences", LogLevel::Debug);
    let processed_sequences = process_sequences(&input, &processed_animations)?;
    log(format!("Model has {} sequences", processed_sequences.len()), LogLevel::Verbose);

    if processed_sequences.len() == 0 {
        return Err(ProcessingDataError::NoSequences);
    }

    if processed_sequences.len() > i32::MAX as usize {
        return Err(ProcessingDataError::TooManySequences(i32::MAX as usize));
    }

    log("Processing Mesh Data", LogLevel::Debug);
    let processed_mesh = process_mesh_data(&input, &import, &bone_table)?;

    todo!()
}

#[derive(Debug)]
pub enum ProcessingDataError {
    BoneHierarchyError,
    TooManyBones(usize),
    TooManyAnimations(usize),
    TooManySequences(usize),
    SequenceAnimationNotFound,
    NoSequences,
}

impl Display for ProcessingDataError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        // let error_message: &str = match self {};
        // TODO: Switch to thiserror!
        fmt.write_str("")
    }
}

impl Error for ProcessingDataError {}
