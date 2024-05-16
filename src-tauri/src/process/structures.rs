use crate::utilities::mathematics::{Angles, Quaternion, Vector2, Vector3, Vector4};

/// The main structure of where all processed data is contained.
/// This processed data is set up to be easily written out at the write stage.
pub struct ProcessedData {
    pub bone_data: ProcessedBoneData,
    pub animation_data: Vec<ProcessedAnimationData>,
    pub sequence_data: Vec<ProcessedSequenceData>,
    pub model_data: ProcessedModelData,
}

/// The main structure that store everything relevant to bone data.
#[derive(Default)]
pub struct ProcessedBoneData {
    /// The bone table were any data that uses bones indexes to.
    pub processed_bones: Vec<ProcessedBone>,
    /// The bone table sorted by name with quick sort.
    pub sorted_bones_by_name: Vec<usize>,
}

/// The structure that contains the data for a bone.
pub struct ProcessedBone {
    /// The name of the bone.
    pub name: String,
    /// If the bone has no parent then its -1 else its the index to the parent bone.
    pub parent: Option<usize>,
    /// The location of the bone relative to its parent or in world space if not parented.
    pub position: Vector3,
    /// The orientation of the bone in world space.
    pub rotation: Angles,
    /// The scale factor of each axis for position animation data as they are stored as fix-point shorts.
    pub animation_position_scale: Vector3,
    /// The scale factor of each axis for rotation animation data as they are stored as fix-point shorts.
    pub animation_rotation_scale: Vector3,
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
    pub is_delta: bool,
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
    pub bone: usize,
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
    pub name: String,
    /// The array of animation indexes used for blending.
    pub animations: Vec<usize>,
}

impl ProcessedSequenceData {
    pub fn new(name: String) -> Self {
        Self { name, animations: Vec::new() }
    }
}

/// The main structure that stores everything relevant to mesh data.
#[derive(Default)]
pub struct ProcessedModelData {
    pub body_groups: Vec<ProcessedBodyGroupData>,
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

pub struct ProcessedBodyGroupData {
    pub name: String,
    pub parts: Vec<ProcessedBodyPartData>,
}

impl ProcessedBodyGroupData {
    pub fn new(name: String) -> Self {
        Self { name, parts: Vec::new() }
    }
}

impl ProcessedBodyGroupData {
    pub fn add_part(&mut self, part: ProcessedBodyPartData) {
        self.parts.push(part);
    }
}

pub struct ProcessedBodyPartData {
    pub name: String,
    pub meshes: Vec<ProcessedMeshData>,
}

impl ProcessedBodyPartData {
    pub fn new(name: String) -> Self {
        Self { name, meshes: Vec::new() }
    }
}

pub struct ProcessedMeshData {
    pub material: usize,
    pub vertex_data: Vec<ProcessedVertexData>,
    pub strip_groups: Vec<ProcessedStripGroup>,
}

impl ProcessedMeshData {
    pub fn new(material: usize) -> Self {
        Self {
            material,
            vertex_data: Vec::new(),
            strip_groups: Vec::new(),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct ProcessedVertexData {
    pub weights: [f64; 3],
    pub bones: [usize; 3],
    pub bone_count: usize,
    pub position: Vector3,
    pub normal: Vector3,
    pub uv: Vector2,
    pub tangent: Vector4,
}

#[derive(Default)]
pub struct ProcessedStripGroup {
    pub vertices: Vec<ProcessedMeshVertex>,
    pub indices: Vec<usize>,
    pub strips: Vec<ProcessedStrip>,
    pub is_flexed: bool,
}

#[derive(Default)]
pub struct ProcessedMeshVertex {
    pub bone_count: usize,
    pub vertex_index: usize,
    pub bones: [usize; 3],
}

#[derive(Default)]
pub struct ProcessedStrip {
    pub indices_count: usize,
    pub vertex_count: usize,
    pub bone_count: usize,
    pub hardware_bones: Vec<ProcessedHardwareBone>,
}

pub struct ProcessedHardwareBone {
    pub hardware_bone: usize,
    pub bone_table_bone: usize,
}
