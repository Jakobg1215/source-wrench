use bitflags::bitflags;
use indexmap::{IndexMap, IndexSet};
use thiserror::Error as ThisError;

use crate::{
    import::FileManager,
    input::ImputedCompilationData,
    utilities::{
        logging::{log, LogLevel},
        mathematics::{Angles, BoundingBox, Matrix4, Quaternion, Vector2, Vector3, Vector4},
    },
};

mod animation;
mod bones;
mod mesh;
mod sequences;

use animation::{process_animations, ProcessingAnimationError};
use bones::{process_bones, ProcessingBoneError};
use mesh::{process_meshes, ProcessingMeshError};
use sequences::{process_sequences, ProcessingSequenceError};

#[derive(Debug, Default)]
pub struct ProcessedData {
    pub bone_data: ProcessedBoneData,
    pub animation_data: ProcessedAnimationData,
    pub sequence_data: Vec<ProcessedSequence>,
    pub model_data: ProcessedModelData,
}

#[derive(Debug, Default)]
pub struct ProcessedBoneData {
    pub processed_bones: IndexMap<String, ProcessedBone>,
    pub sorted_bones_by_name: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct ProcessedBone {
    pub parent: Option<usize>,
    /// The position of the bone relative to the parent bone.
    pub position: Vector3,
    /// The rotation of the bone relative to the parent bone.
    pub rotation: Angles,
    pub flags: ProcessedBoneFlags,
    /// The transforms in world space.
    pub pose: Matrix4,
}

bitflags! {
    #[derive(Debug, Default)]
    pub struct ProcessedBoneFlags: i32 {
        const USED_BY_VERTEX = 0x00000400;
    }
}

#[derive(Debug, Default)]
pub struct ProcessedAnimationData {
    pub processed_animations: Vec<ProcessedAnimation>,
    pub animation_scales: Vec<(Vector3, Vector3)>,
    pub remapped_animations: Vec<usize>,
}

#[derive(Debug, Default)]
pub struct ProcessedAnimation {
    pub name: String,
    pub frame_count: usize,
    pub sections: Vec<Vec<ProcessedAnimatedBoneData>>,
}

#[derive(Debug, Default)]
pub struct ProcessedAnimatedBoneData {
    pub bone: u8,
    pub raw_position: Vec<Vector3>,
    pub raw_rotation: Vec<Quaternion>,
    pub delta_position: Vec<Vector3>,
    pub delta_rotation: Vec<Angles>,
}

#[derive(Debug, Default)]
pub struct ProcessedSequence {
    pub name: String,
    pub animations: Vec<Vec<i16>>,
}

#[derive(Debug, Default)]
pub struct ProcessedModelData {
    pub body_parts: Vec<ProcessedBodyPart>,
    pub bounding_box: BoundingBox,
    pub hitboxes: IndexMap<u8, BoundingBox>,
    pub materials: IndexSet<String>,
}

#[derive(Debug, Default)]
pub struct ProcessedBodyPart {
    pub name: String,
    pub models: Vec<ProcessedModel>,
}

#[derive(Debug, Default)]
pub struct ProcessedModel {
    pub name: String,
    pub meshes: Vec<ProcessedMesh>,
}

#[derive(Debug, Default)]
pub struct ProcessedMesh {
    pub material: i32,
    pub vertex_data: Vec<ProcessedVertex>,
    pub strip_groups: Vec<ProcessedStripGroup>,
}

#[derive(Debug, Default)]
pub struct ProcessedVertex {
    pub weights: [f32; 3],
    pub bones: [u8; 3],
    pub bone_count: u8,
    pub position: Vector3,
    pub normal: Vector3,
    pub texture_coordinate: Vector2,
    pub tangent: Vector4,
}

#[derive(Debug, Default)]
pub struct ProcessedStripGroup {
    pub vertices: Vec<ProcessedMeshVertex>,
    pub indices: Vec<u16>,
    pub strips: Vec<ProcessedStrip>,
}

#[derive(Debug, Default)]
pub struct ProcessedMeshVertex {
    pub bone_count: u8,
    pub vertex_index: u16,
    pub bones: [u8; 3],
}

#[derive(Debug, Default)]
pub struct ProcessedStrip {
    pub indices_count: i32,
    pub indices_offset: i32,
    pub vertex_count: i32,
    pub vertex_offset: i32,
    pub bone_count: i16,
    pub hardware_bones: Vec<ProcessedHardwareBone>,
}

#[derive(Debug, Default)]
pub struct ProcessedHardwareBone {
    pub hardware_bone: i32,
    pub bone_table_bone: i32,
}

#[derive(Debug, ThisError)]
pub enum ProcessingDataError {
    #[error("Model Has No Bones")]
    NoBones,
    #[error("Model Has No Sequences")]
    NoSequences,
    #[error("Model Has No Animations")]
    NoAnimations,
    #[error("Failed To Process Bone Data: {0}")]
    ProcessingBoneError(#[from] ProcessingBoneError),
    #[error("Failed To Process Animation Data: {0}")]
    ProcessingAnimationError(#[from] ProcessingAnimationError),
    #[error("Failed To Process Sequence Data: {0}")]
    ProcessingSequenceError(#[from] ProcessingSequenceError),
    #[error("Failed To Process Mesh Data: {0}")]
    ProcessingMeshError(#[from] ProcessingMeshError),
}

pub const MAX_HARDWARE_BONES_PER_STRIP: usize = 53;
pub const VERTEX_CACHE_SIZE: usize = 16;

/// The tolerance for floating point numbers until they are considered equal.
pub const FLOAT_TOLERANCE: f64 = f32::EPSILON as f64;

pub fn process(input: &ImputedCompilationData, file_manager: &FileManager) -> Result<ProcessedData, ProcessingDataError> {
    log("Processing Bones.", LogLevel::Debug);
    let processed_bone_data = process_bones(input, file_manager)?;
    log(format!("Model uses {} bones.", processed_bone_data.processed_bones.len()), LogLevel::Verbose);

    if processed_bone_data.processed_bones.is_empty() {
        return Err(ProcessingDataError::NoBones);
    }

    log("Processing Animations.", LogLevel::Debug);
    let processed_animation_data = process_animations(input, file_manager, &processed_bone_data)?;
    log(
        format!("Model has {} animations.", processed_animation_data.processed_animations.len()),
        LogLevel::Verbose,
    );

    if processed_animation_data.processed_animations.is_empty() {
        return Err(ProcessingDataError::NoAnimations);
    }

    log("Processing Sequences.", LogLevel::Debug);
    let processed_sequences = process_sequences(input, &processed_animation_data.remapped_animations)?;
    log(format!("Model has {} sequences.", processed_sequences.len()), LogLevel::Verbose);

    if processed_sequences.is_empty() {
        return Err(ProcessingDataError::NoSequences);
    }

    log("Processing Mesh Data.", LogLevel::Debug);
    let processed_mesh = process_meshes(input, file_manager, &processed_bone_data)?;
    log(format!("Model has {} materials.", processed_mesh.materials.len()), LogLevel::Verbose);
    log(format!("Model has {} body parts.", processed_mesh.body_parts.len()), LogLevel::Verbose);

    Ok(ProcessedData {
        bone_data: processed_bone_data,
        animation_data: processed_animation_data,
        sequence_data: processed_sequences,
        model_data: processed_mesh,
    })
}
