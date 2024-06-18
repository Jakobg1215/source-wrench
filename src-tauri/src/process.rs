use animation::{process_animations, process_sequences};
use bones::process_bone_table;
use mesh::process_mesh_data;
use tauri::State;
use thiserror::Error as ThisError;

use crate::{
    import::FileManager,
    input::ImputedCompilationData,
    process::bones::create_bone_table,
    utilities::{
        logging::{log, LogLevel},
        mathematics::{Angles, Quaternion, Vector2, Vector3, Vector4},
    },
};

mod animation;
mod bones;
mod mesh;

#[derive(Default, Debug)]
pub struct ProcessedData {
    pub bone_data: ProcessedBoneData,
    pub animation_data: Vec<ProcessedAnimation>,
    pub sequence_data: Vec<ProcessedSequence>,
    pub model_data: ProcessedModelData,
}

#[derive(Default, Debug)]
pub struct ProcessedBoneData {
    pub processed_bones: Vec<ProcessedBone>,
    pub sorted_bones_by_name: Vec<usize>,
}

#[derive(Default, Debug)]
pub struct ProcessedBone {
    pub name: String,
    pub parent: Option<usize>,
    pub position: Vector3,
    pub rotation: Angles,
    pub animation_position_scale: Vector3,
    pub animation_rotation_scale: Vector3,
}

#[derive(Default, Debug)]
pub struct ProcessedAnimation {
    pub name: String,
    pub frame_count: usize,
    pub bones: Vec<ProcessedAnimatedBoneData>,
}

#[derive(Default, Debug)]
pub struct ProcessedAnimatedBoneData {
    pub bone: usize,
    pub position: Option<ProcessedAnimationPosition>,
    pub rotation: Option<ProcessedAnimationRotation>,
}

#[derive(Debug)]
pub enum ProcessedAnimationPosition {
    Raw(Vector3),
    Compressed, // TODO: Implement compression
}

#[derive(Debug)]
pub enum ProcessedAnimationRotation {
    Raw(Quaternion),
    Compressed, // TODO: Implement compression
}

#[derive(Default, Debug)]
pub struct ProcessedSequence {
    pub name: String,
    pub animations: Vec<usize>,
}

#[derive(Default, Debug)]
pub struct ProcessedModelData {
    pub body_parts: Vec<ProcessedBodyPart>,
    pub materials: Vec<String>,
}

impl ProcessedModelData {
    pub fn add_material(&mut self, new_material: String) -> usize {
        match self.materials.iter().position(|material| material == &new_material) {
            Some(index) => return index,
            None => self.materials.push(new_material),
        };

        self.materials.len() - 1
    }
}

#[derive(Default, Debug)]
pub struct ProcessedBodyPart {
    pub name: String,
    pub parts: Vec<ProcessedModel>,
}

#[derive(Default, Debug)]
pub struct ProcessedModel {
    pub name: String,
    pub meshes: Vec<ProcessedMesh>,
}

#[derive(Default, Debug)]
pub struct ProcessedMesh {
    pub material: usize,
    pub vertex_data: Vec<ProcessedVertex>,
    pub strip_groups: Vec<ProcessedStripGroup>,
}

#[derive(Default, Debug, Clone)]
pub struct ProcessedVertex {
    pub weights: [f64; 3],
    pub bones: [usize; 3],
    pub bone_count: usize,
    pub position: Vector3,
    pub normal: Vector3,
    pub uv: Vector2,
    pub tangent: Vector4,
}

#[derive(Default, Debug)]
pub struct ProcessedStripGroup {
    pub vertices: Vec<ProcessedMeshVertex>,
    pub indices: Vec<usize>,
    pub strips: Vec<ProcessedStrip>,
    pub is_flexed: bool,
}

#[derive(Default, Debug)]
pub struct ProcessedMeshVertex {
    pub bone_count: usize,
    pub vertex_index: usize,
    pub bones: [usize; 3],
}

#[derive(Default, Debug)]
pub struct ProcessedStrip {
    pub indices_count: usize,
    pub vertex_count: usize,
    pub bone_count: usize,
    pub hardware_bones: Vec<ProcessedHardwareBone>,
}

#[derive(Default, Debug)]
pub struct ProcessedHardwareBone {
    pub hardware_bone: usize,
    pub bone_table_bone: usize,
}

#[derive(Debug, ThisError)]
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
    #[error("Model Has Too Many Materials")]
    TooManyMaterials,
    #[error("Model Has Too Many Body Parts")]
    TooManyBodyParts,
}

/// The tolerance for floating point numbers until they are considered equal.
// TODO: Make this an imputed value.
const FLOAT_TOLERANCE: f64 = 0.000001;

pub fn process(input: &ImputedCompilationData, file_manager: &State<FileManager>) -> Result<ProcessedData, ProcessingDataError> {
    log("Creating Bone Table", LogLevel::Debug);
    let mut bone_table = create_bone_table(&file_manager)?;
    log(format!("Model uses {} source bones", bone_table.bones.len()), LogLevel::Verbose);

    // TODO: Mark bones as collapsed if they are not used.

    log("Processing Animations", LogLevel::Debug);
    let processed_animations = process_animations(&input, &file_manager, &mut bone_table)?;
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
    let processed_mesh = process_mesh_data(&input, &file_manager, &bone_table)?;
    log(format!("Model has {} materials", processed_mesh.materials.len()), LogLevel::Verbose);
    log(format!("Model has {} body parts", processed_mesh.body_parts.len()), LogLevel::Verbose);

    if processed_mesh.body_parts.len() > i32::MAX as usize {
        return Err(ProcessingDataError::TooManyBodyParts);
    }

    if processed_mesh.materials.len() > i16::MAX as usize {
        return Err(ProcessingDataError::TooManyMaterials);
    }

    let processed_bone_data = process_bone_table(&bone_table);
    if processed_bone_data.processed_bones.len() > i8::MAX as usize {
        return Err(ProcessingDataError::TooManyBones);
    }

    let mut processed_data = ProcessedData::default();
    processed_data.bone_data = processed_bone_data;
    processed_data.animation_data = processed_animations;
    processed_data.sequence_data = processed_sequences;
    processed_data.model_data = processed_mesh;
    Ok(processed_data)
}
