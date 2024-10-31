use crate::utilities::mathematics::{Angles, BoundingBox, Matrix4, Quaternion, Vector3};

use bitflags::bitflags;

use super::{FileWriteError, FileWriter, WriteToWriter};

#[derive(Debug)]
pub struct ModelFileHeader {
    pub identifier: ModelFileHeaderIdentifier,
    pub version: i32,
    pub checksum: i32,
    pub file_length_index: usize,
    pub eye_position: Vector3,
    pub illumination_position: Vector3,
    pub bounding_box: BoundingBox,
    pub clipping_box: BoundingBox,
    pub flags: ModelFileHeaderFlags,
    pub bones: Vec<ModelFileBone>,
    pub bone_offset: usize,
    pub bone_controllers: Vec<()>,
    pub bone_controller_offset: usize,
    pub hitbox_sets: Vec<ModelFileHitboxSet>,
    pub hitbox_set_offset: usize,
    pub local_animation_descriptions: Vec<ModelFileAnimationDescription>,
    pub local_animation_description_offset: usize,
    pub local_sequence_descriptions: Vec<ModelFileSequenceDescription>,
    pub local_sequence_description_offset: usize,
    pub materials: Vec<ModelFileMaterial>,
    pub material_offset: usize,
    pub material_paths: Vec<String>,
    pub material_path_offset: usize,
    pub material_replacements: Vec<Vec<i16>>,
    pub material_replacement_offset: usize,
    pub body_parts: Vec<ModelFileBodyPart>,
    pub body_part_offset: usize,
    pub local_attachments: Vec<()>,
    pub local_attachment_offset: usize,
    pub local_nodes: Vec<()>,
    pub local_node_offset: usize,
    pub local_node_names_offset: usize,
    pub flex_descriptions: Vec<()>,
    pub flex_description_offset: usize,
    pub flex_controllers: Vec<()>,
    pub flex_controller_offset: usize,
    pub flex_rules: Vec<()>,
    pub flex_rule_offset: usize,
    pub inverse_kinematic_chains: Vec<()>,
    pub inverse_kinematic_chain_offset: usize,
    pub mouths: Vec<()>,
    pub mouth_offset: usize,
    pub local_pose_parameters: Vec<()>,
    pub local_pose_parameters_offset: usize,
    pub surface_properties: String,
    pub keyvalues: String,
    pub local_inverse_kinematics_auto_play_locks: Vec<()>,
    pub local_inverse_kinematics_auto_play_lock_offset: usize,
    pub mass: f32,
    pub contents: ModelFileHeaderContents,
    pub include_models: Vec<()>,
    pub include_model_offset: usize,
    pub animation_block_file_name: String,
    pub animation_blocks: Vec<()>,
    pub animation_block_offset: usize,
    pub sorted_bone_table_by_name: Vec<u8>,
    pub sorted_bone_table_by_name_index: usize,
    pub constant_directional_light_dot: u8,
    pub root_lod: u8,
    pub max_allowed_root_lod: u8,
    pub flex_flex_controller_remaps: Vec<()>,
    pub flex_flex_controller_remap_offset: usize,
    pub vertex_animation_scale: f32,
    pub second_header: ModelFileSecondHeader,
    pub second_header_offset: usize,
}

impl Default for ModelFileHeader {
    fn default() -> Self {
        Self {
            identifier: Default::default(),
            version: Default::default(),
            checksum: Default::default(),
            file_length_index: Default::default(),
            eye_position: Default::default(),
            illumination_position: Default::default(),
            bounding_box: Default::default(),
            clipping_box: Default::default(),
            flags: ModelFileHeaderFlags::FORCE_OPAQUE,
            bones: Default::default(),
            bone_offset: Default::default(),
            bone_controllers: Default::default(),
            bone_controller_offset: Default::default(),
            hitbox_sets: Default::default(),
            hitbox_set_offset: Default::default(),
            local_animation_descriptions: Default::default(),
            local_animation_description_offset: Default::default(),
            local_sequence_descriptions: Default::default(),
            local_sequence_description_offset: Default::default(),
            materials: Default::default(),
            material_offset: Default::default(),
            material_paths: Default::default(),
            material_path_offset: Default::default(),
            material_replacements: Default::default(),
            material_replacement_offset: Default::default(),
            body_parts: Default::default(),
            body_part_offset: Default::default(),
            local_attachments: Default::default(),
            local_attachment_offset: Default::default(),
            local_nodes: Default::default(),
            local_node_offset: Default::default(),
            local_node_names_offset: Default::default(),
            flex_descriptions: Default::default(),
            flex_description_offset: Default::default(),
            flex_controllers: Default::default(),
            flex_controller_offset: Default::default(),
            flex_rules: Default::default(),
            flex_rule_offset: Default::default(),
            inverse_kinematic_chains: Default::default(),
            inverse_kinematic_chain_offset: Default::default(),
            mouths: Default::default(),
            mouth_offset: Default::default(),
            local_pose_parameters: Default::default(),
            local_pose_parameters_offset: Default::default(),
            surface_properties: String::from("default"),
            keyvalues: Default::default(),
            local_inverse_kinematics_auto_play_locks: Default::default(),
            local_inverse_kinematics_auto_play_lock_offset: Default::default(),
            mass: Default::default(),
            contents: ModelFileHeaderContents::SOLID,
            include_models: Default::default(),
            include_model_offset: Default::default(),
            animation_block_file_name: Default::default(),
            animation_blocks: Default::default(),
            animation_block_offset: Default::default(),
            sorted_bone_table_by_name: Default::default(),
            sorted_bone_table_by_name_index: Default::default(),
            constant_directional_light_dot: Default::default(),
            root_lod: Default::default(),
            max_allowed_root_lod: Default::default(),
            flex_flex_controller_remaps: Default::default(),
            flex_flex_controller_remap_offset: Default::default(),
            vertex_animation_scale: Default::default(),
            second_header: Default::default(),
            second_header_offset: Default::default(),
        }
    }
}

impl WriteToWriter for ModelFileHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_integer(self.identifier.to_integer());
        writer.write_integer(self.version);
        writer.write_integer(self.checksum);
        writer.write_char_array("Model Compiled With Source Wrench!", 64);
        self.file_length_index = writer.write_integer_index();
        writer.write_vector3(self.eye_position);
        writer.write_vector3(self.illumination_position);
        debug_assert!(self.bounding_box.is_valid(), "Bounding box is invalid!");
        writer.write_vector3(self.bounding_box.minimum);
        writer.write_vector3(self.bounding_box.maximum);
        debug_assert!(self.bounding_box.is_valid(), "Clipping box is invalid!");
        writer.write_vector3(self.clipping_box.minimum);
        writer.write_vector3(self.clipping_box.maximum);
        // TODO: Assert flags for flags that conflict.
        writer.write_integer(self.flags.bits());
        writer.write_array_size(self.bones.len())?;
        self.bone_offset = writer.write_integer_index();
        writer.write_array_size(self.bone_controllers.len())?;
        self.bone_controller_offset = writer.write_integer_index();
        writer.write_array_size(self.hitbox_sets.len())?;
        self.hitbox_set_offset = writer.write_integer_index();
        writer.write_array_size(self.local_animation_descriptions.len())?;
        self.local_animation_description_offset = writer.write_integer_index();
        writer.write_array_size(self.local_sequence_descriptions.len())?;
        self.local_sequence_description_offset = writer.write_integer_index();
        writer.write_integer(0);
        writer.write_integer(0);
        writer.write_array_size(self.materials.len())?;
        self.material_offset = writer.write_integer_index();
        writer.write_array_size(self.material_paths.len())?;
        self.material_path_offset = writer.write_integer_index();
        writer.write_array_size(self.materials.len())?;
        writer.write_array_size(self.material_replacements.len())?;
        self.material_replacement_offset = writer.write_integer_index();
        writer.write_array_size(self.body_parts.len())?;
        self.body_part_offset = writer.write_integer_index();
        writer.write_array_size(self.local_attachments.len())?;
        self.local_attachment_offset = writer.write_integer_index();
        writer.write_array_size(self.local_nodes.len())?;
        self.local_node_offset = writer.write_integer_index();
        self.local_node_names_offset = writer.write_integer_index();
        writer.write_array_size(self.flex_descriptions.len())?;
        self.flex_description_offset = writer.write_integer_index();
        writer.write_array_size(self.flex_controllers.len())?;
        self.flex_controller_offset = writer.write_integer_index();
        writer.write_array_size(self.flex_rules.len())?;
        self.flex_rule_offset = writer.write_integer_index();
        writer.write_array_size(self.inverse_kinematic_chains.len())?;
        self.inverse_kinematic_chain_offset = writer.write_integer_index();
        writer.write_array_size(self.mouths.len())?;
        self.mouth_offset = writer.write_integer_index();
        writer.write_array_size(self.local_pose_parameters.len())?;
        self.local_pose_parameters_offset = writer.write_integer_index();
        writer.write_string_to_table(0, &self.surface_properties);
        writer.write_string_to_table(0, &self.keyvalues);
        if (self.keyvalues.len()) > i32::MAX as usize {
            return Err(FileWriteError::KeyvaluesToLarge);
        }
        writer.write_integer((self.keyvalues.len()) as i32);
        writer.write_array_size(self.local_inverse_kinematics_auto_play_locks.len())?;
        self.local_inverse_kinematics_auto_play_lock_offset = writer.write_integer_index();
        writer.write_float(self.mass);
        // TODO: Assert flags for flags that conflict.
        writer.write_integer(self.contents.bits());
        writer.write_array_size(self.include_models.len())?;
        self.include_model_offset = writer.write_integer_index();
        writer.write_integer(0);
        writer.write_string_to_table(0, &self.animation_block_file_name);
        writer.write_array_size(self.animation_blocks.len())?;
        self.animation_block_offset = writer.write_integer_index();
        writer.write_integer(0);
        self.sorted_bone_table_by_name_index = writer.write_integer_index();
        writer.write_integer(0);
        writer.write_integer(0);
        writer.write_unsigned_byte(self.constant_directional_light_dot);
        writer.write_unsigned_byte(self.root_lod);
        writer.write_unsigned_byte(self.max_allowed_root_lod);
        writer.write_unsigned_byte_array(&[0]);
        writer.write_integer(0);
        writer.write_array_size(self.flex_flex_controller_remaps.len())?;
        self.flex_flex_controller_remap_offset = writer.write_integer_index();
        writer.write_float(self.vertex_animation_scale);
        writer.write_integer_array(&[0]);
        self.second_header_offset = writer.write_integer_index();
        writer.write_integer_array(&[0]);

        writer.write_to_integer_offset(self.second_header_offset, writer.data.len())?;
        self.second_header.write(writer)?;

        writer.write_to_integer_offset(self.bone_offset, writer.data.len())?;
        for bone in &mut self.bones {
            bone.write(writer)?;
        }
        writer.align(4);
        // TODO: Write Bone Procedurals

        writer.write_to_integer_offset(self.bone_controller_offset, writer.data.len())?;
        // TODO: Write Bone Controllers

        writer.write_to_integer_offset(self.local_attachment_offset, writer.data.len())?;
        // TODO: Write Attachments

        writer.write_to_integer_offset(self.hitbox_set_offset, writer.data.len())?;
        for hitbox_set in &mut self.hitbox_sets {
            hitbox_set.write(writer)?;
        }
        writer.align(4);

        for hitbox_set in &mut self.hitbox_sets {
            hitbox_set.write_hitboxes(writer)?;
            writer.align(4);
        }

        writer.write_to_integer_offset(self.sorted_bone_table_by_name_index, writer.data.len())?;
        writer.write_unsigned_byte_array(&self.sorted_bone_table_by_name);
        writer.align(4);

        writer.write_to_integer_offset(self.local_animation_description_offset, writer.data.len())?;
        for animation_description in &mut self.local_animation_descriptions {
            animation_description.write(writer)?;
        }
        writer.align(4);

        for animation_description in &mut self.local_animation_descriptions {
            animation_description.write_sections(writer)?;
            writer.align(16);
            animation_description.write_animations(writer)?;
            writer.align(4);
        }
        // TODO: Write Local Animation Description IK errors, Local Hierarchy, Movement, Bone Save Frames

        writer.write_to_integer_offset(self.local_sequence_description_offset, writer.data.len())?;
        for sequence_description in &mut self.local_sequence_descriptions {
            sequence_description.write(writer)?;
        }

        // TODO: Write Local Sequence Descriptions Pose Keys, events, auto layers, auto layer rules, sequence group, local activity modifier

        for sequence_description in &mut self.local_sequence_descriptions {
            sequence_description.write_bone_weights(writer)?;
        }

        // TODO: Write Local Sequence Descriptions ik locks

        for sequence_description in &mut self.local_sequence_descriptions {
            sequence_description.write_animations(writer)?;
        }
        writer.align(4);

        writer.write_to_integer_offset(self.local_node_names_offset, writer.data.len())?;
        // TODO: Write Local Node Names

        writer.write_to_integer_offset(self.local_node_offset, writer.data.len())?;
        // TODO: Write Local Nodes

        writer.write_to_integer_offset(self.body_part_offset, writer.data.len())?;
        for body_part in &mut self.body_parts {
            body_part.write(writer)?;
        }

        for body_part in &mut self.body_parts {
            body_part.write_models(writer)?;
        }

        for body_part in &mut self.body_parts {
            body_part.write_model_meshes(writer)?;
        }
        writer.align(4);

        // TODO: Write Body Parts Eyeballs, flexes

        writer.write_to_integer_offset(self.flex_description_offset, writer.data.len())?;
        // TODO: Write Flex Descriptions

        writer.write_to_integer_offset(self.flex_controller_offset, writer.data.len())?;
        // TODO: Write Flex Controllers

        writer.write_to_integer_offset(self.flex_rule_offset, writer.data.len())?;
        // TODO: Write Flex Rules

        writer.write_to_integer_offset(self.flex_flex_controller_remap_offset, writer.data.len())?;
        // TODO: Write Flex Controller Remaps

        writer.write_to_integer_offset(self.inverse_kinematic_chain_offset, writer.data.len())?;
        // TODO: Write Inverse Kinematic Chains

        writer.write_to_integer_offset(self.local_inverse_kinematics_auto_play_lock_offset, writer.data.len())?;
        // TODO: Write Local Inverse Kinematic Auto Play Locks

        writer.write_to_integer_offset(self.mouth_offset, writer.data.len())?;
        // TODO: Write Mouths

        writer.write_to_integer_offset(self.local_pose_parameters_offset, writer.data.len())?;
        // TODO: Write Local Pose Parameters

        writer.write_to_integer_offset(self.include_model_offset, writer.data.len())?;
        // TODO: Write Include Models

        writer.write_to_integer_offset(self.animation_block_offset, writer.data.len())?;
        // TODO: Write Animation Blocks

        writer.write_to_integer_offset(self.material_offset, writer.data.len())?;
        for material in &mut self.materials {
            material.write(writer)?;
        }
        writer.align(4);

        writer.write_to_integer_offset(self.material_path_offset, writer.data.len())?;
        for material_path in &self.material_paths {
            writer.write_string_to_table(0, material_path);
        }
        writer.align(4);

        writer.write_to_integer_offset(self.material_replacement_offset, writer.data.len())?;
        for material_replacement in &self.material_replacements {
            for material in material_replacement {
                writer.write_short(*material);
            }
        }
        writer.align(4);

        self.second_header.write_source_bone_transforms(writer)?;

        self.second_header.write_linear(writer)?;

        self.second_header.write_bone_flex_driver(writer)?;

        writer.write_string_table()?;

        writer.write_to_integer_offset(self.file_length_index, writer.data.len())?;

        Ok(())
    }
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum ModelFileHeaderIdentifier {
    #[default]
    Studio,
    Animation,
}

impl ModelFileHeaderIdentifier {
    pub fn to_integer(&self) -> i32 {
        match self {
            ModelFileHeaderIdentifier::Studio => (84 << 24) + (83 << 16) + (68 << 8) + 73,
            ModelFileHeaderIdentifier::Animation => (71 << 24) + (65 << 16) + (68 << 8) + 73,
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct ModelFileHeaderFlags: i32 {
        const AUTOMATIC_GENERATED_HITBOXES       = 0x00000001;
        const FORCE_OPAQUE                       = 0x00000004;
        const FORCE_TRANSLUCENT                  = 0x00000008;
        const STATIC_PROP                        = 0x00000010;
        const HAS_SHADOW_LOD                     = 0x00000040;
        const USE_SHADOW_LOD_MATERIALS           = 0x00000100;
        const OBSOLETE                           = 0x00000200;
        const NO_FORCED_FADE                     = 0x00000800;
        const FORCE_PHONEME_CROSS_FADE           = 0x00001000;
        const CONSTANT_DIRECTIONAL_LIGHT_DOT     = 0x00002000;
        const AMBIENT_BOOST                      = 0x00010000;
        const DO_NOT_CAST_SHADOWS                = 0x00020000;
        const CAST_TEXTURE_SHADOWS               = 0x00040000;
        const VERTEX_ANIMATION_FIXED_POINT_SCALE = 0x00200000;
    }

    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct ModelFileHeaderContents: i32 {
        const SOLID   = 0x00000001;
        const GRATE   = 0x00000008;
        const MONSTER = 0x02000000;
        const LADDER  = 0x20000000;
    }
}

#[derive(Debug, Default)]
pub struct ModelFileSecondHeader {
    pub write_base: usize,
    pub source_bone_transforms: Vec<()>,
    pub source_bone_transform_offset: usize,
    pub illumination_position_attachment_index: i32,
    pub max_eye_deflection: f32,
    #[allow(dead_code)]
    pub linear_bones: Option<()>,
    pub linear_bone_index: usize,
    pub name: String,
    pub bone_flex_drivers: Vec<()>,
    pub bone_flex_driver_offset: usize,
}

impl WriteToWriter for ModelFileSecondHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();
        writer.write_array_size(self.source_bone_transforms.len())?;
        self.source_bone_transform_offset = writer.write_integer_index();
        writer.write_integer(self.illumination_position_attachment_index);
        writer.write_float(self.max_eye_deflection);
        self.linear_bone_index = writer.write_integer_index();
        writer.write_string_to_table(self.write_base, &self.name);
        writer.write_array_size(self.bone_flex_drivers.len())?;
        self.bone_flex_driver_offset = writer.write_integer_index();
        writer.write_integer_array(&[0; 56]);

        Ok(())
    }
}

impl ModelFileSecondHeader {
    fn write_source_bone_transforms(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.source_bone_transform_offset, writer.data.len())?;
        // TODO: Write Source Bone Transforms
        Ok(())
    }

    fn write_linear(&mut self, _writer: &mut FileWriter) -> Result<(), FileWriteError> {
        // TODO: Write Linear Bones
        Ok(())
    }

    fn write_bone_flex_driver(&mut self, _writer: &mut FileWriter) -> Result<(), FileWriteError> {
        // TODO: Write Bone Flex Driver
        Ok(())
    }
}

#[derive(Debug)]
pub struct ModelFileBone {
    pub write_base: usize,
    pub name: String,
    pub parent: i32,
    pub bone_controllers: [i32; 6],
    pub position: Vector3,
    pub quaternion: Quaternion,
    pub rotation: Angles,
    pub animation_position_scale: Vector3,
    pub animation_rotation_scale: Vector3,
    pub pose: Matrix4,
    pub alignment: Quaternion,
    pub flags: ModelFileBoneFlags,
    pub procedural_type: Option<ModelFileBoneProceduralType>,
    pub procedural_offset: usize,
    pub physics_index: i32,
    pub surface_properties: String,
    pub contents: ModelFileHeaderContents,
}

impl Default for ModelFileBone {
    fn default() -> Self {
        Self {
            write_base: Default::default(),
            name: Default::default(),
            parent: Default::default(),
            bone_controllers: [-1; 6],
            position: Default::default(),
            quaternion: Default::default(),
            rotation: Default::default(),
            animation_position_scale: Default::default(),
            animation_rotation_scale: Default::default(),
            pose: Default::default(),
            alignment: Default::default(),
            flags: Default::default(),
            procedural_type: Default::default(),
            procedural_offset: Default::default(),
            physics_index: Default::default(),
            surface_properties: String::from("default"),
            contents: ModelFileHeaderContents::SOLID,
        }
    }
}

impl WriteToWriter for ModelFileBone {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();
        writer.write_string_to_table(self.write_base, &self.name);
        writer.write_integer(self.parent);
        writer.write_integer_array(&self.bone_controllers);
        writer.write_vector3(self.position);
        writer.write_quaternion(self.quaternion);
        writer.write_angles(self.rotation);
        writer.write_vector3(self.animation_position_scale);
        writer.write_vector3(self.animation_rotation_scale);
        writer.write_float_array(&[
            self.pose.entries[0][0] as f32,
            self.pose.entries[0][1] as f32,
            self.pose.entries[0][2] as f32,
            self.pose.entries[0][3] as f32,
            self.pose.entries[1][0] as f32,
            self.pose.entries[1][1] as f32,
            self.pose.entries[1][2] as f32,
            self.pose.entries[1][3] as f32,
            self.pose.entries[2][0] as f32,
            self.pose.entries[2][1] as f32,
            self.pose.entries[2][2] as f32,
            self.pose.entries[2][3] as f32,
        ]);
        writer.write_quaternion(self.alignment);
        writer.write_integer(self.flags.bits());
        writer.write_integer(self.procedural_type.as_ref().map_or(0, |procedural| procedural.to_integer()));
        self.procedural_offset = writer.write_integer_index();
        writer.write_integer(self.physics_index);
        writer.write_string_to_table(self.write_base, &self.surface_properties);
        writer.write_integer(self.contents.bits());
        writer.write_integer_array(&[0; 8]);

        Ok(())
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct ModelFileBoneFlags: i32 {
        const ALWAYS_PROCEDURAL        = 0x00000004;
        const SCREEN_ALIGN_SPHERE      = 0x00000008;
        const SCREEN_ALIGN_CYLINDER    = 0x00000010;
        const USED_BY_HITBOX           = 0x00000100;
        const USED_BY_ATTACHMENT       = 0x00000200;
        const USED_BY_VERTEX_AT_LOD0   = 0x00000400;
        const USED_BY_VERTEX_AT_LOD1   = 0x00000800;
        const USED_BY_VERTEX_AT_LOD2   = 0x00001000;
        const USED_BY_VERTEX_AT_LOD3   = 0x00002000;
        const USED_BY_VERTEX_AT_LOD4   = 0x00004000;
        const USED_BY_VERTEX_AT_LOD5   = 0x00008000;
        const USED_BY_VERTEX_AT_LOD6   = 0x00010000;
        const USED_BY_VERTEX_AT_LOD7   = 0x00020000;
        const USED_BY_VERTEX_MASK      = 0x0003FC00;
        const USED_BY_BONE_MERGE       = 0x00040000;
        const USED_BY_ANYTHING_AT_LOD0 = 0x00040700;
        const USED_BY_ANYTHING_AT_LOD1 = 0x00040b00;
        const USED_BY_ANYTHING_AT_LOD2 = 0x00041300;
        const USED_BY_ANYTHING_AT_LOD3 = 0x00042300;
        const USED_BY_ANYTHING_AT_LOD4 = 0x00044300;
        const USED_BY_ANYTHING_AT_LOD5 = 0x00048300;
        const USED_BY_ANYTHING_AT_LOD6 = 0x00050300;
        const USED_BY_ANYTHING_AT_LOD7 = 0x00060300;
        const USED_BY_ANYTHING         = 0x0007FF00;
        const FIXED_ALIGNMENT          = 0x00100000;
        const HAS_SAVE_FRAME_POSITION  = 0x00200000;
        const HAS_SAVE_FRAME_ROTATION  = 0x00400000;
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ModelFileBoneProceduralType {
    // TODO: Add Structure Values To Enum Options.
    AxisInterpolation,
    QuaternionInterpolation,
    AimAtBone,
    AimAtAttachment,
    Jiggle,
}

impl ModelFileBoneProceduralType {
    fn to_integer(&self) -> i32 {
        match self {
            Self::AxisInterpolation => 1,
            Self::QuaternionInterpolation => 2,
            Self::AimAtBone => 3,
            Self::AimAtAttachment => 4,
            Self::Jiggle => 5,
        }
    }
}

#[derive(Debug)]
pub struct ModelFileHitboxSet {
    pub write_base: usize,
    pub name: String,
    pub hitboxes: Vec<ModelFileHitBox>,
    pub hitbox_offset: usize,
}

impl Default for ModelFileHitboxSet {
    fn default() -> Self {
        Self {
            write_base: Default::default(),
            name: String::from("default"),
            hitboxes: Default::default(),
            hitbox_offset: Default::default(),
        }
    }
}

impl WriteToWriter for ModelFileHitboxSet {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();
        writer.write_string_to_table(self.write_base, &self.name);
        writer.write_array_size(self.hitboxes.len())?;
        self.hitbox_offset = writer.write_integer_index();

        Ok(())
    }
}

impl ModelFileHitboxSet {
    fn write_hitboxes(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.hitbox_offset, writer.data.len() - self.write_base)?;

        for hitbox in &mut self.hitboxes {
            hitbox.write(writer)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ModelFileHitBox {
    pub write_base: usize,
    pub bone: i32,
    pub group: i32,
    pub bounding_box: BoundingBox,
    pub name: Option<String>,
}

impl WriteToWriter for ModelFileHitBox {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();
        writer.write_integer(self.bone);
        writer.write_integer(self.group);
        writer.write_vector3(self.bounding_box.minimum);
        writer.write_vector3(self.bounding_box.maximum);
        match &self.name {
            Some(name) => writer.write_string_to_table(self.write_base, name),
            None => writer.write_integer(0),
        }
        writer.write_integer_array(&[0; 8]);

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ModelFileAnimationDescription {
    pub write_base: usize,
    pub name: String,
    pub fps: f32,
    pub flags: ModelFileAnimationDescriptionFlags,
    pub frame_count: i32,
    pub movements: Vec<()>,
    pub movement_offset: usize,
    pub animation_block: i32,
    pub animation_sections: Vec<ModelFileAnimationSection>,
    pub animation_offset: usize,
    pub inverse_kinematic_rules: Vec<()>,
    pub inverse_kinematic_rule_offset: usize,
    pub local_hierarchy: Vec<()>,
    pub local_hierarchy_offset: usize,
    pub sections_offset: usize,
    pub frames_per_section: i32,
    pub zero_frame_span_count: i16,
    pub zero_frames: Vec<()>,
    pub zero_frame_offset: usize,
}

impl WriteToWriter for ModelFileAnimationDescription {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();
        writer.write_negative_offset(writer.data.len())?;
        writer.write_string_to_table(self.write_base, &self.name);
        debug_assert!(self.fps > 0.0, "FPS is less than or equal to zero!");
        writer.write_float(self.fps);
        writer.write_integer(self.flags.bits());
        debug_assert!(self.frame_count > 0, "Frame count is less than or equal to zero!");
        writer.write_integer(self.frame_count);
        writer.write_array_size(self.movements.len())?;
        self.movement_offset = writer.write_integer_index();
        writer.write_integer_array(&[0; 6]);
        writer.write_integer(self.animation_block);
        self.animation_offset = writer.write_integer_index();
        writer.write_array_size(self.inverse_kinematic_rules.len())?;
        self.inverse_kinematic_rule_offset = writer.write_integer_index();
        writer.write_integer(0); // TODO: Write Rules To Animation Block When Implemented.
        writer.write_array_size(self.local_hierarchy.len())?;
        self.local_hierarchy_offset = writer.write_integer_index();
        debug_assert!(!self.animation_sections.is_empty(), "No animation sections!");
        self.sections_offset = writer.write_integer_index();
        debug_assert!(self.frames_per_section >= 0, "Frames per section is less than zero!");
        if self.frames_per_section > 0 {
            debug_assert!(
                self.animation_sections.len() > 1,
                "Frames per section is greater than zero when there is no sections!"
            );
        }
        writer.write_integer(self.frames_per_section);
        writer.write_short(self.zero_frame_span_count);
        writer.write_array_size_short(self.zero_frames.len())?;
        self.zero_frame_offset = writer.write_integer_index();
        writer.write_integer(0);

        Ok(())
    }
}

impl ModelFileAnimationDescription {
    fn write_sections(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        if self.animation_sections.len() == 1 {
            return Ok(());
        }

        writer.write_to_integer_offset(self.sections_offset, writer.data.len() - self.write_base)?;

        for section in &mut self.animation_sections {
            section.write(writer)?;
        }

        Ok(())
    }

    fn write_animations(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.animation_offset, writer.data.len() - self.write_base)?;

        if self.animation_sections.len() == 1 {
            let section = &mut self.animation_sections[0];
            section.write_animation(writer, true, self.write_base)?;
            return Ok(());
        }

        for section in &mut self.animation_sections {
            section.write_animation(writer, false, self.write_base)?;
        }

        Ok(())
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct ModelFileAnimationDescriptionFlags: i32 {
        const LOOPING  = 0x0001;
        const DELTA    = 0x0004;
        const ALL_ZERO = 0x0020;
    }
}

#[derive(Debug, Default)]
pub struct ModelFileAnimationSection {
    pub write_base: usize,
    pub animation_block: i32,
    pub animation_index: usize,
    pub animation_data: Vec<ModelFileAnimation>,
}

impl WriteToWriter for ModelFileAnimationSection {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();
        writer.write_integer(self.animation_block);
        self.animation_index = writer.write_integer_index();

        Ok(())
    }
}

impl ModelFileAnimationSection {
    fn write_animation(&mut self, writer: &mut FileWriter, single: bool, animation_description_index: usize) -> Result<(), FileWriteError> {
        if !single {
            writer.write_to_integer_offset(self.animation_index, writer.data.len() - animation_description_index)?;
        }

        if self.animation_data.is_empty() {
            let mut animation = ModelFileAnimation {
                bone: 255,
                ..Default::default()
            };
            animation.write(writer)?;
            return Ok(());
        }

        for animation in &mut self.animation_data {
            animation.write(writer)?;
            writer.write_to_short_offset(animation.next_offset, writer.data.len() - animation.write_base)?;
        }

        writer.write_to_short_offset(self.animation_data.last().unwrap().next_offset, 0)?;

        writer.write_integer(0); // This is for crowbar as studioMDL does write this as well.
        Ok(())
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    struct ModelFileAnimationFlags: u8 {
        const RAW_POSITION = 0x01;
        const ANIMATED_POSITION = 0x04;
        const ANIMATED_ROTATION = 0x08;
        const DELTA = 0x10;
        const RAW_ROTATION = 0x20;
    }
}

#[derive(Debug, Default)]
pub struct ModelFileAnimation {
    pub write_base: usize,
    pub delta: bool,
    pub bone: u8,
    pub rotation: Option<ModelFileAnimationData<Angles>>,
    pub position: Option<ModelFileAnimationData<Vector3>>,
    pub next_offset: usize,
}

impl WriteToWriter for ModelFileAnimation {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();

        writer.write_unsigned_byte(self.bone);

        let mut flags = ModelFileAnimationFlags::empty();

        if self.delta {
            flags |= ModelFileAnimationFlags::DELTA;
        }

        if let Some(data) = &self.rotation {
            match data {
                ModelFileAnimationData::Single(_) => flags |= ModelFileAnimationFlags::RAW_ROTATION,
                ModelFileAnimationData::Array(_) => flags |= ModelFileAnimationFlags::ANIMATED_ROTATION,
            }
        }

        if let Some(data) = &self.position {
            match data {
                ModelFileAnimationData::Single(_) => flags |= ModelFileAnimationFlags::RAW_POSITION,
                ModelFileAnimationData::Array(_) => flags |= ModelFileAnimationFlags::ANIMATED_POSITION,
            }
        }

        writer.write_unsigned_byte(flags.bits());
        self.next_offset = writer.write_short_index();

        if let Some(data) = &mut self.rotation {
            match data {
                ModelFileAnimationData::Single(value) => {
                    writer.write_quaternion64(value.to_quaternion());
                }
                ModelFileAnimationData::Array(value) => {
                    value.write(writer)?;
                }
            }
        }

        if let Some(data) = &mut self.position {
            match data {
                ModelFileAnimationData::Single(value) => {
                    writer.write_vector48(*value);
                }
                ModelFileAnimationData::Array(value) => {
                    value.write(writer)?;
                }
            }
        }

        if let Some(ModelFileAnimationData::Array(data)) = &mut self.rotation {
            data.write_data(writer)?;
        }

        if let Some(ModelFileAnimationData::Array(data)) = &mut self.position {
            data.write_data(writer)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ModelFileAnimationData<T> {
    Single(T),
    Array(ModelFileAnimationValue),
}

#[derive(Debug, Default)]
pub struct ModelFileAnimationValue {
    pub write_base: usize,
    pub offsets: [usize; 3],
    pub values: [Option<Vec<ModelFileAnimationEncoding>>; 3],
}

impl WriteToWriter for ModelFileAnimationValue {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();

        for axis in 0..3 {
            self.offsets[axis] = writer.write_short_index();
        }

        Ok(())
    }
}

impl ModelFileAnimationValue {
    fn write_data(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        for axis in 0..3 {
            if let Some(values) = &self.values[axis] {
                writer.write_to_short_offset(self.offsets[axis], writer.data.len() - self.write_base)?;

                for value in values {
                    match value {
                        ModelFileAnimationEncoding::Header(header) => {
                            writer.write_unsigned_byte(header.valid);
                            writer.write_unsigned_byte(header.total);
                        }
                        ModelFileAnimationEncoding::Value(value) => {
                            writer.write_short(*value);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ModelFileAnimationEncoding {
    Header(ModelFileAnimationEncodingHeader),
    Value(i16),
}

#[derive(Debug, Default)]
pub struct ModelFileAnimationEncodingHeader {
    pub valid: u8,
    pub total: u8,
}

#[derive(Debug)]
pub struct ModelFileSequenceDescription {
    pub write_base: usize,
    pub name: String,
    pub activity_name: String,
    pub flags: ModelFileSequenceDescriptionFlags,
    pub activity: i32,
    pub activity_weight: i32,
    pub events: Vec<()>,
    pub event_offset: usize,
    pub bounding_box: BoundingBox,
    pub animations: Vec<i16>,
    pub animation_offset: usize,
    pub blend_size: [i32; 2],
    pub parameters: [i32; 2],
    pub parameters_start: [f32; 2],
    pub parameters_end: [f32; 2],
    pub fade_in_time: f32,
    pub fade_out_time: f32,
    pub local_entry_node: i32,
    pub local_exit_node: i32,
    pub reverse_transition: bool,
    pub inversive_kinematic_count: i32,
    pub auto_layers: Vec<()>,
    pub auto_layer_offset: usize,
    pub weight_list: Vec<f32>,
    pub weight_list_offset: usize,
    #[allow(dead_code)]
    pub pose_keys: Vec<f32>, // TODO: What the fuck are these for? Is the name correct for this? Seems fine if they don't exist but don't trust that.
    pub pose_key_index: usize,
    pub inversive_kinematics_locks: Vec<()>,
    pub inversive_kinematics_lock_offset: usize,
    pub keyvalues: String,
    pub pose_cycle: i32,
    pub activity_modifiers: Vec<String>,
    pub activity_modifier_offset: usize,
}

impl Default for ModelFileSequenceDescription {
    fn default() -> Self {
        Self {
            write_base: Default::default(),
            name: Default::default(),
            activity_name: Default::default(),
            flags: Default::default(),
            activity: -1,
            activity_weight: Default::default(),
            events: Default::default(),
            event_offset: Default::default(),
            bounding_box: Default::default(),
            animations: Default::default(),
            animation_offset: Default::default(),
            blend_size: Default::default(),
            parameters: [-1; 2],
            parameters_start: Default::default(),
            parameters_end: Default::default(),
            fade_in_time: Default::default(),
            fade_out_time: Default::default(),
            local_entry_node: Default::default(),
            local_exit_node: Default::default(),
            reverse_transition: Default::default(),
            inversive_kinematic_count: Default::default(),
            auto_layers: Default::default(),
            auto_layer_offset: Default::default(),
            weight_list: Default::default(),
            weight_list_offset: Default::default(),
            pose_keys: Default::default(),
            pose_key_index: Default::default(),
            inversive_kinematics_locks: Default::default(),
            inversive_kinematics_lock_offset: Default::default(),
            keyvalues: Default::default(),
            pose_cycle: Default::default(),
            activity_modifiers: Default::default(),
            activity_modifier_offset: Default::default(),
        }
    }
}

impl WriteToWriter for ModelFileSequenceDescription {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();
        writer.write_negative_offset(writer.data.len())?;
        writer.write_string_to_table(self.write_base, &self.name);
        writer.write_string_to_table(self.write_base, &self.activity_name);
        writer.write_integer(self.flags.bits());
        writer.write_integer(self.activity);
        writer.write_integer(self.activity_weight);
        writer.write_array_size(self.events.len())?;
        self.event_offset = writer.write_integer_index();
        writer.write_vector3(self.bounding_box.minimum);
        writer.write_vector3(self.bounding_box.maximum);
        writer.write_array_size(self.animations.len())?;
        self.animation_offset = writer.write_integer_index();
        writer.write_integer(0);
        writer.write_integer_array(&self.blend_size);
        writer.write_integer_array(&self.parameters);
        writer.write_float_array(&self.parameters_start);
        writer.write_float_array(&self.parameters_end);
        writer.write_integer(0);
        writer.write_float(self.fade_in_time);
        writer.write_float(self.fade_out_time);
        writer.write_integer(self.local_entry_node);
        writer.write_integer(self.local_exit_node);
        writer.write_integer(self.reverse_transition as i32);
        writer.write_float(0.0);
        writer.write_float(0.0);
        writer.write_float(0.0);
        writer.write_integer(0);
        writer.write_integer(0);
        writer.write_integer(self.inversive_kinematic_count);
        writer.write_array_size(self.auto_layers.len())?;
        self.auto_layer_offset = writer.write_integer_index();
        self.weight_list_offset = writer.write_integer_index();
        self.pose_key_index = writer.write_integer_index();
        writer.write_array_size(self.inversive_kinematics_locks.len())?;
        self.inversive_kinematics_lock_offset = writer.write_integer_index();
        writer.write_string_to_table(self.write_base, &self.keyvalues);
        if (self.keyvalues.len()) > i32::MAX as usize {
            return Err(FileWriteError::KeyvaluesToLarge);
        }
        writer.write_integer(self.keyvalues.len() as i32);
        writer.write_integer(self.pose_cycle);
        writer.write_array_size(self.activity_modifiers.len())?;
        self.activity_modifier_offset = writer.write_integer_index();
        writer.write_integer_array(&[0; 5]);

        Ok(())
    }
}

impl ModelFileSequenceDescription {
    fn write_bone_weights(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.weight_list_offset, writer.data.len() - self.write_base)?;

        for weight in &self.weight_list {
            writer.write_float(*weight);
        }

        Ok(())
    }

    fn write_animations(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.animation_offset, writer.data.len() - self.write_base)?;

        for animation in &self.animations {
            writer.write_short(*animation);
        }

        Ok(())
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct ModelFileSequenceDescriptionFlags: i32 {
        const LOOPING    = 0x0001;
        const SNAP       = 0x0002;
        const DELTA      = 0x0004;
        const AUTO_PLAY  = 0x0008;
        const POST       = 0x0010;
        const CYCLE_POSE = 0x0080;
        const REALTIME   = 0x0100;
        const LOCAL      = 0x0200;
        const ACTIVITY   = 0x1000;
        const EVENT      = 0x2000;
        const WORLD      = 0x4000;
    }
}

#[derive(Debug, Default)]
pub struct ModelFileBodyPart {
    pub write_base: usize,
    pub name: String,
    pub models: Vec<ModelFileModel>,
    pub model_offset: usize,
    pub base: i32,
}

impl WriteToWriter for ModelFileBodyPart {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();

        writer.write_string_to_table(self.write_base, &self.name);
        writer.write_array_size(self.models.len())?;
        writer.write_integer(self.base);
        self.model_offset = writer.write_integer_index();

        Ok(())
    }
}

impl ModelFileBodyPart {
    fn write_models(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.model_offset, writer.data.len() - self.write_base)?;

        for model in &mut self.models {
            model.write(writer)?;
        }
        writer.align(4);

        Ok(())
    }

    fn write_model_meshes(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        for model in &mut self.models {
            writer.write_to_integer_offset(model.mesh_offset, writer.data.len() - model.write_base)?;

            for mesh in &mut model.meshes {
                mesh.model_index = model.write_base;
                mesh.write(writer)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ModelFileModel {
    pub write_base: usize,
    pub name: String,
    pub meshes: Vec<ModelFileMesh>,
    pub mesh_offset: usize,
    pub vertex_count: i32,
    pub vertex_offset: i32,
    pub tangent_offset: i32,
    pub eyeballs: Vec<()>,
    pub eyeball_offset: usize,
}

impl WriteToWriter for ModelFileModel {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();

        writer.write_char_array(&self.name, 64);
        writer.write_integer(0);
        writer.write_float(0.0);
        writer.write_array_size(self.meshes.len())?;
        self.mesh_offset = writer.write_integer_index();
        writer.write_integer(self.vertex_count);
        writer.write_integer(self.vertex_offset);
        writer.write_integer(self.tangent_offset);
        writer.write_integer(0);
        writer.write_integer(0);
        writer.write_array_size(self.eyeballs.len())?;
        self.eyeball_offset = writer.write_integer_index();
        writer.write_integer(0);
        writer.write_integer(0);
        writer.write_integer_array(&[0; 8]);

        Ok(())
    }
}

impl ModelFileModel {}

#[derive(Debug, Default)]
pub struct ModelFileMesh {
    pub write_base: usize,
    pub material: i32,
    pub model_index: usize,
    pub vertex_count: i32,
    pub vertex_offset: i32,
    pub flexes: Vec<()>,
    pub flex_offset: usize,
    pub is_eye_mesh: bool,
    pub eye_index: i32,
    pub mesh_identifier: i32,
    pub vertex_lod_count: [i32; 8],
}

impl WriteToWriter for ModelFileMesh {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();

        writer.write_integer(self.material);
        writer.write_negative_offset(self.write_base - self.model_index)?;
        writer.write_integer(self.vertex_count);
        writer.write_integer(self.vertex_offset);
        writer.write_array_size(self.flexes.len())?;
        self.flex_offset = writer.write_integer_index();
        writer.write_integer(self.is_eye_mesh as i32);
        writer.write_integer(self.eye_index);
        writer.write_integer(self.mesh_identifier);
        writer.write_vector3(Vector3::default());
        writer.write_integer(0);
        writer.write_integer_array(&self.vertex_lod_count);
        writer.write_integer_array(&[0; 8]);

        Ok(())
    }
}

impl ModelFileMesh {}

#[derive(Debug, Default)]
pub struct ModelFileMaterial {
    pub write_base: usize,
    pub name: String,
}

impl WriteToWriter for ModelFileMaterial {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();

        writer.write_string_to_table(self.write_base, &self.name);
        writer.write_integer(0);
        writer.write_integer(0);
        writer.write_integer(0);
        writer.write_integer(0);
        writer.write_integer(0);
        writer.write_integer_array(&[0; 10]);

        Ok(())
    }
}
