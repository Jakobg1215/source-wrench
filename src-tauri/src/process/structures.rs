use serde_json::value::Index;

use crate::utilities::mathematics::{Angles, Quaternion, Vector2, Vector3, Vector4};

/// The main structure of where all processed data is contained.
/// This processed data is set up to be easily written out at the write stage.
pub struct ProcessedData {
    bone_data: ProcessedBoneData,
    animation_data: Vec<ProcessedAnimationData>,
    sequence_data: Vec<ProcessedSequenceData>,
    model_data: ProcessedModelData,
}

impl ProcessedData {
    pub fn new() -> Self {
        Self {
            bone_data: ProcessedBoneData::new(),
            animation_data: Vec::new(),
            sequence_data: Vec::new(),
            model_data: ProcessedModelData::new(),
        }
    }
}

/// The main structure that store everything relevant to bone data.
pub struct ProcessedBoneData {
    /// The bone table were any data that uses bones indexes to.
    processed_bones: Vec<ProcessedBone>,
    /// The bone table sorted by name with quick sort.
    sorted_bones_by_name: Vec<usize>,
}

impl ProcessedBoneData {
    pub fn new() -> Self {
        Self {
            processed_bones: Vec::new(),
            sorted_bones_by_name: Vec::new(),
        }
    }
}

/// The structure that contains the data for a bone.
pub struct ProcessedBone {
    /// The name of the bone.
    name: String,
    /// If the bone has no parent then its -1 else its the index to the parent bone.
    parent: Option<usize>,
    /// The location of the bone relative to its parent or in world space if not parented.
    position: Vector3,
    /// The orientation of the bone in world space.
    rotation: Angles,
    /// The scale factor of each axis for position animation data as they are stored as fix-point shorts.
    animation_position_scale: Vector3,
    /// The scale factor of each axis for rotation animation data as they are stored as fix-point shorts.
    animation_rotation_scale: Vector3,
}

impl ProcessedBone {
    pub fn new(name: String) -> Self {
        Self {
            name,
            parent: None,
            position: Vector3::zero(),
            rotation: Angles::zero(),
            animation_position_scale: Vector3::zero(),
            animation_rotation_scale: Vector3::zero(),
        }
    }
}

/// The structure that contains data for an animation.
pub struct ProcessedAnimationData {
    /// The name of the animation.
    pub name: String,
    /// If the animation is delta for additive animations.
    is_delta: bool,
    /// The amount of frames that the animation has.
    pub frame_count: usize,
    /// The animation data of the bones in the animation.
    pub bones: Vec<ProcessedAnimatedBoneData>,
}

impl ProcessedAnimationData {
    pub fn new(name: String) -> Self {
        Self {
            name,
            is_delta: false,
            frame_count: 0,
            bones: Vec::new(),
        }
    }
}

/// The structure for the an animated bone.
pub struct ProcessedAnimatedBoneData {
    /// The index of the bone in the bone table.
    bone: usize,
    /// The position data of the animated bone.
    pub position: Option<ProcessedAnimationPosition>,
    /// The rotation data of the animated bone.
    pub rotation: Option<ProcessedAnimationRotation>,
}

impl ProcessedAnimatedBoneData {
    pub fn new(bone: usize) -> Self {
        Self {
            bone,
            position: None,
            rotation: None,
        }
    }
}

pub enum ProcessedAnimationPosition {
    /// If the bone is changed from its bind pose but only for the first frame.
    Raw(Vector3),
    /// The animation data in run length encoding.
    Compressed, // TODO: Implement compression
}

pub enum ProcessedAnimationRotation {
    /// If the bone is changed from its bind pose but only for the first frame.
    Raw(Quaternion),
    /// The animation data in run length encoding.
    Compressed, // TODO: Implement compression
}

/// The structure that contains data for sequences.
pub struct ProcessedSequenceData {
    /// The name of the sequence.
    name: String,
    /// The array of animation indexes used for blending.
    pub animations: Vec<usize>,
}

impl ProcessedSequenceData {
    pub fn new(name: String) -> Self {
        Self { name, animations: Vec::new() }
    }
}

/// The main structure that stores everything relevant to mesh data.
pub struct ProcessedModelData {
    pub body_groups: Vec<ProcessedBodyGroupData>,
    pub materials: Vec<String>,
}

impl ProcessedModelData {
    pub fn new() -> Self {
        Self {
            body_groups: Vec::new(),
            materials: Vec::new(),
        }
    }
}

impl ProcessedModelData {
    pub fn add_material(&mut self, new_material: String) -> usize {
        match self.materials.iter().position(|material| material == &new_material) {
            Some(index) => return index,
            None => self.materials.push(new_material),
        };

        self.materials.len() - 1
    }

    pub fn get_material_index(&self, material: &str) -> usize {
        // UNWRAP: The material should exist from adding it.
        self.materials.iter().position(|mat| mat == material).unwrap()
    }
}

pub struct ProcessedBodyGroupData {
    name: String,
    parts: Vec<ProcessedBodyPartData>,
}

impl ProcessedBodyGroupData {
    pub fn new(name: String) -> Self {
        Self { name, parts: Vec::new() }
    }
}

pub struct ProcessedBodyPartData {
    name: String,
    meshes: Vec<ProcessedMeshData>,
}

pub struct ProcessedMeshData {
    material: usize,
    vertex_data: Vec<ProcessedVertexData>,
    strip_groups: Vec<ProcessedStripGroup>,
}

pub struct ProcessedVertexData {
    weights: [f64; 3],
    bones: [u8; 3],
    bone_count: u8,
    position: Vector3,
    normal: Vector3,
    uv: Vector2,
    tangent: Vector4,
}

pub struct ProcessedStripGroup {
    vertices: Vec<ProcessedMeshVertex>,
    indices: Vec<u16>,
    strips: Vec<ProcessedStrip>,
    is_flexed: bool,
}

pub struct ProcessedMeshVertex {
    bone_count: u8,
    vertex_index: u16,
    bones: [u8; 3],
}

pub struct ProcessedStrip {
    indices_count: i32,
    vertex_count: i32,
    bone_count: i16,
    hardware_bones: Vec<ProcessedHardwareBone>,
}

pub struct ProcessedHardwareBone {
    hardware_bone: i32,
    bone_table_bone: i32,
}
