use crate::{
    process::{ProcessedAnimationPosition, ProcessedAnimationRotation},
    utilities::{
        binarydata::DataWriter,
        mathematics::{Angles, Matrix, Quaternion, Vector3},
    },
};

use super::StructWriting;

pub struct Header {
    write_start_index: usize,
    write_length_index: usize,
    pub bones: Vec<Bone>,
    write_bones_index: usize,
    pub hitbox_sets: Vec<HitboxSet>,
    write_hitbox_sets_index: usize,
    pub animations: Vec<AnimationDescription>,
    write_animations_index: usize,
    pub sequences: Vec<SequenceDescription>,
    write_sequences_index: usize,
    pub materials: Vec<Material>,
    write_materials_index: usize,
    pub material_paths: Vec<String>,
    write_material_paths_index: usize,
    write_skin_families_index: usize,
    pub body_parts: Vec<BodyPart>,
    write_body_parts_index: usize,
    pub bones_index: Vec<usize>,
    write_bone_table_by_name_index: usize,
    pub second_header: SecondHeader,
    write_second_header_index: usize,
}

impl StructWriting for Header {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.write_start_index = writer.get_size();
        writer.write_string("IDST", 4); // id
        writer.write_int(48); // version
        writer.write_int(69420); // checksum
        writer.write_string("Model Compiled With Source Wrench!", 64); // name
        self.write_length_index = writer.write_index(); // length
        writer.write_vector3(&Vector3::default()); // eyeposition
        writer.write_vector3(&Vector3::default()); // illumposition
        writer.write_vector3(&Vector3::new(10.0, 10.0, 0.0)); // hull_min
        writer.write_vector3(&Vector3::new(-10.0, -10.0, 10.0)); // hull_max
        writer.write_vector3(&Vector3::default()); // view_bbmin
        writer.write_vector3(&Vector3::default()); // view_bbmax
        writer.write_int(0); // flags
        writer.write_int(self.bones.len() as i32); // numbones
        self.write_bones_index = writer.write_index(); // boneindex
        writer.write_int(0); // numbonecontrollers
        writer.write_int(self.write_bones_index as i32); // bonecontrollerindex
        writer.write_int(self.hitbox_sets.len() as i32); // numhitboxsets
        self.write_hitbox_sets_index = writer.write_index(); // hitboxsetindex
        writer.write_int(self.animations.len() as i32); // numlocalanim
        self.write_animations_index = writer.write_index(); // localanimindex
        writer.write_int(self.sequences.len() as i32); // numlocalseq
        self.write_sequences_index = writer.write_index(); // localseqindex
        writer.write_int(0); // activitylistversion
        writer.write_int(0); // eventsindexed
        writer.write_int(self.materials.len() as i32); // numtextures
        self.write_materials_index = writer.write_index(); // textureindex
        writer.write_int(self.material_paths.len() as i32); // numcdtextures
        self.write_material_paths_index = writer.write_index(); // cdtextureindex
        writer.write_int(self.materials.len() as i32); // numskinref
        writer.write_int(self.materials.len() as i32); // numskinfamilies
        self.write_skin_families_index = writer.write_index(); // skinindex
        writer.write_int(self.body_parts.len() as i32); // numbodyparts
        self.write_body_parts_index = writer.write_index(); // bodypartindex
        writer.write_int(0); // numlocalattachments
        writer.write_int(self.write_body_parts_index as i32); // localattachmentindex
        writer.write_int(0); // numlocalnodes
        writer.write_int(self.write_body_parts_index as i32); // localnodeindex
        writer.write_int(self.write_body_parts_index as i32); // localnodenameindex
        writer.write_int(0); // numflexdesc
        writer.write_int(self.write_body_parts_index as i32); // flexdescindex
        writer.write_int(0); // numflexcontrollers
        writer.write_int(self.write_body_parts_index as i32); // flexcontrollerindex
        writer.write_int(0); // numflexrules
        writer.write_int(self.write_body_parts_index as i32); // flexruleindex
        writer.write_int(0); // numikchains
        writer.write_int(self.write_body_parts_index as i32); // ikchainindex
        writer.write_int(0); // nummouths
        writer.write_int(self.write_body_parts_index as i32); // mouthindex
        writer.write_int(0); // numlocalposeparameters
        writer.write_int(self.write_body_parts_index as i32); // localposeparamindex
        writer.add_string_to_table(self.write_start_index, "Default"); // surfacepropindex
        writer.write_int(self.write_body_parts_index as i32); // keyvalueindex
        writer.write_int(0); // keyvaluesize
        writer.write_int(0); // numlocalikautoplaylocks
        writer.write_int(self.write_body_parts_index as i32); // localikautoplaylockindex
        writer.write_float(0.0); // mass
        writer.write_int(0); // contents
        writer.write_int(0); // numincludemodels
        writer.write_int(self.write_body_parts_index as i32); // includemodelindex
        writer.write_int(0); // virtualModel
        writer.add_string_to_table(self.write_start_index, "");
        writer.write_int(0); // numanimblocks
        writer.write_int(self.write_body_parts_index as i32); // animblockindex
        writer.write_int(0); // animblockModel
        self.write_bone_table_by_name_index = writer.write_index(); // bonetablebynameindex
        writer.write_int(0); // pVertexBase
        writer.write_int(0); // pIndexBase
        writer.write_unsigned_byte(0); // directional_light_dot
        writer.write_unsigned_byte(0); // rootLOD
        writer.write_unsigned_byte(0); // numAllowedRootLODs
        writer.write_unsigned_byte_array(&vec![0]);
        writer.write_int(0); // unused4
        writer.write_int(0); // numflexcontrollerui
        writer.write_int(self.write_bone_table_by_name_index as i32); // flexcontrolleruiindex
        writer.write_float(0.0); // flVertAnimFixedPointScale
        writer.write_int_array(&vec![0]); // unused3
        self.write_second_header_index = writer.write_index(); // studiohdr2index
        writer.write_int_array(&vec![0]); // unused2

        self.write_second_header(writer);

        self.write_bones(writer);

        self.write_hitbox_sets(writer);

        self.write_bone_table_by_name(writer);

        self.write_animations(writer);

        self.write_sequences(writer);

        self.write_body_parts(writer);

        self.write_materials(writer);

        self.write_skin_families(writer);

        self.write_material_paths(writer);

        writer.write_string_table();

        writer.write_to_index(self.write_length_index, writer.get_size() as i32);
    }
}

impl Header {
    pub fn new() -> Self {
        Self {
            write_start_index: usize::MAX,
            write_length_index: usize::MAX,
            bones: Vec::new(),
            write_bones_index: usize::MAX,
            hitbox_sets: Vec::new(),
            write_hitbox_sets_index: usize::MAX,
            animations: Vec::new(),
            write_animations_index: usize::MAX,
            sequences: Vec::new(),
            write_sequences_index: usize::MAX,
            materials: Vec::new(),
            write_materials_index: usize::MAX,
            material_paths: Vec::new(),
            write_material_paths_index: usize::MAX,
            write_skin_families_index: usize::MAX,
            body_parts: Vec::new(),
            write_body_parts_index: usize::MAX,
            bones_index: Vec::new(),
            write_bone_table_by_name_index: usize::MAX,
            second_header: SecondHeader::new(),
            write_second_header_index: usize::MAX,
        }
    }

    fn write_second_header(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_second_header_index, writer.get_size() as i32);

        self.second_header.write_to_writer(writer);
    }

    fn write_bones(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_bones_index, writer.get_size() as i32);

        for bone in &mut self.bones {
            bone.write_to_writer(writer);
        }
    }

    fn write_hitbox_sets(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_hitbox_sets_index, writer.get_size() as i32);

        for hitbox_set in &mut self.hitbox_sets {
            hitbox_set.write_to_writer(writer);
        }

        for hitbox_set in &mut self.hitbox_sets {
            hitbox_set.write_hitboxes(writer);
        }
    }

    fn write_bone_table_by_name(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_bone_table_by_name_index, writer.get_size() as i32);

        for bone in &self.bones_index {
            writer.write_unsigned_byte(*bone as u8);
        }

        writer.align(4);
    }

    fn write_animations(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_animations_index, writer.get_size() as i32);

        for animation in &mut self.animations {
            animation.write_to_writer(writer);
        }

        for animation in &mut self.animations {
            writer.align(16);
            animation.write_animation(writer);
        }
    }

    fn write_sequences(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_sequences_index, writer.get_size() as i32);

        for sequence in &mut self.sequences {
            sequence.write_to_writer(writer);
        }

        for sequence in &mut self.sequences {
            sequence.write_weight_list(writer);
            sequence.write_animation_indexes(writer);
            writer.align(4);
        }
    }

    fn write_body_parts(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_body_parts_index, writer.get_size() as i32);

        for body_part in &mut self.body_parts {
            body_part.write_to_writer(writer);
        }

        for body_part in &mut self.body_parts {
            body_part.write_models(writer);
        }
    }

    fn write_materials(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_materials_index, writer.get_size() as i32);

        for material in &mut self.materials {
            material.write_to_writer(writer);
        }
    }

    fn write_material_paths(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_material_paths_index, writer.get_size() as i32);
        for path in &self.material_paths {
            writer.add_string_to_table(0, &path);
        }
    }

    fn write_skin_families(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_skin_families_index, writer.get_size() as i32);
        for material in 0..self.materials.len() {
            writer.write_short(material as i16);
        }
    }
}

pub struct SecondHeader {
    write_start_index: usize,
    pub model_name: String,
}

impl StructWriting for SecondHeader {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.write_start_index = writer.get_size();

        writer.write_int(0); // numsrcbonetransform
        writer.write_int(self.write_start_index as i32); // srcbonetransformindex
        writer.write_int(0); // illumpositionattachmentindex
        writer.write_float(0.0); // flMaxEyeDeflection
        writer.write_int(0); // linearboneindex
        writer.add_string_to_table(self.write_start_index, &self.model_name); // sznameindex
        writer.write_int(0); // m_nBoneFlexDriverCount
        writer.write_int(self.write_start_index as i32); // m_nBoneFlexDriverIndex
        writer.write_int_array(&vec![0; 56]); // reserved
    }
}

impl SecondHeader {
    pub fn new() -> Self {
        Self {
            write_start_index: usize::MAX,
            model_name: String::new(),
        }
    }
}

pub struct Bone {
    write_start_index: usize,
    pub name: String,
    pub parent_bone_index: i32,
    pub position: Vector3,
    pub rotation: Angles,
    pub animation_position_scale: Vector3,
    pub animation_rotation_scale: Vector3,
}

impl StructWriting for Bone {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.write_start_index = writer.get_size();

        writer.add_string_to_table(self.write_start_index, &self.name); // sznameindex
        writer.write_int(self.parent_bone_index); // parent
        writer.write_int_array(&vec![-1; 6]); // bonecontroller
        writer.write_vector3(&self.position); // pos
        writer.write_quaternion(&self.rotation.to_quaternion()); // quat
        writer.write_angles(&self.rotation); // rot
        writer.write_vector3(&self.animation_position_scale); // posscale
        writer.write_vector3(&self.animation_rotation_scale); // rotscale
        writer.write_matrix(Matrix::identity()); // poseToBone
        writer.write_quaternion(&Quaternion::zero()); // qAlignment
        writer.write_int(1024); // flags
        writer.write_int(0); // proctype
        writer.write_int(-1); // procindex
        writer.write_int(0); // physicsbone
        writer.add_string_to_table(self.write_start_index, "Default"); // surfacepropidx
        writer.write_int(0); // contents
        writer.write_int_array(&vec![0; 8]); // unused

        assert_eq!(writer.get_size() - self.write_start_index, 216, "The Bone byte size is not 216 bytes!");
    }
}

impl Bone {
    pub fn new() -> Self {
        Self {
            write_start_index: usize::MAX,
            name: String::new(),
            parent_bone_index: -1,
            position: Vector3::default(),
            rotation: Angles::default(),
            animation_position_scale: Vector3::default(),
            animation_rotation_scale: Vector3::default(),
        }
    }
}

pub struct HitboxSet {
    write_start_index: usize,
    pub name: String,
    pub hitboxes: Vec<Hitbox>,
    write_hitboxes_index: usize,
}

impl StructWriting for HitboxSet {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.write_start_index = writer.get_size();

        writer.add_string_to_table(self.write_start_index, &self.name); // sznameindex
        writer.write_int(self.hitboxes.len() as i32); // numhitboxes
        self.write_hitboxes_index = writer.write_index(); // hitboxindex
    }
}

impl HitboxSet {
    pub fn new() -> Self {
        Self {
            write_start_index: usize::MAX,
            name: String::new(),
            hitboxes: Vec::new(),
            write_hitboxes_index: usize::MAX,
        }
    }

    fn write_hitboxes(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_hitboxes_index, (writer.get_size() - self.write_start_index) as i32);

        for hitbox in &mut self.hitboxes {
            hitbox.write_to_writer(writer);
        }
    }
}

pub struct Hitbox {
    write_start_index: usize,
    pub bone_index: usize,
    pub group: usize,
    pub minumum: Vector3,
    pub maximum: Vector3,
    pub name: Option<String>,
}

impl StructWriting for Hitbox {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.write_start_index = writer.get_size();

        writer.write_int(self.bone_index as i32); // bone
        writer.write_int(self.group as i32); // group
        writer.write_vector3(&self.minumum); // bbmin
        writer.write_vector3(&self.maximum); // bbmax
        match &self.name {
            Some(name) => {
                writer.add_string_to_table(self.write_start_index, name);
            }
            None => {
                writer.write_int(0);
            }
        } // szhitboxnameindex
        writer.write_int_array(&vec![0; 8]); // unused
    }
}

impl Hitbox {
    pub fn new() -> Self {
        Self {
            write_start_index: usize::MAX,
            bone_index: usize::MAX,
            group: 0,
            minumum: Vector3::default(),
            maximum: Vector3::default(),
            name: None,
        }
    }
}

pub struct AnimationDescription {
    write_start_index: usize,
    pub name: String,
    pub frame_count: usize,
    pub animation: Animation,
    write_animation_index: usize,
}

impl StructWriting for AnimationDescription {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.write_start_index = writer.get_size();

        writer.write_int(-(writer.get_size() as i32));
        writer.add_string_to_table(self.write_start_index, &self.name); // sznameindex
        writer.write_float(30.0); // fps
        writer.write_int(0); // flags
        writer.write_int(self.frame_count as i32); // numframes
        writer.write_int(0); // nummovements
        writer.write_int(self.write_start_index as i32); // movementindex
        writer.write_int_array(&vec![0; 6]); // unused1

        // TODO: Write out the block data when animation blocking is implemented.
        writer.write_int(0); // animblock
        self.write_animation_index = writer.write_index(); // animindex
        writer.write_int(0); // numikrules
        writer.write_int(self.write_animation_index as i32); // ikruleindex

        // TODO: Write out the block data when animation blocking is implemented.
        writer.write_int(0); // animblockikruleindex
        writer.write_int(0); // numlocalhierarchy
        writer.write_int(self.write_animation_index as i32); // localhierarchyindex

        // TODO: Write out when animation sectioning is implemented.
        writer.write_int(0); // sectionindex
        writer.write_int(0); // sectionframes

        // TODO: write out when zero frame is implemented.
        writer.write_short(0); // zeroframespan
        writer.write_short(0); // zeroframecount
        writer.write_int(0); // zeroframeindex
        writer.write_int(0); // zeroframestalltime
    }
}

impl AnimationDescription {
    pub fn new() -> Self {
        Self {
            write_start_index: usize::MAX,
            name: String::new(),
            frame_count: 0,
            animation: Animation::new(),
            write_animation_index: usize::MAX,
        }
    }

    fn write_animation(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_animation_index, (writer.get_size() - self.write_start_index) as i32);
        self.animation.write_to_writer(writer);
    }
}

pub struct Animation {
    write_start_index: usize,
    pub animation_data: Vec<AnimationData>,
    write_offset_index: usize,
}

impl StructWriting for Animation {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.write_start_index = writer.get_size();

        if self.animation_data.len() == 0 {
            writer.write_unsigned_byte(255); // bone
            writer.write_unsigned_byte(0); // flags
            writer.write_short(0);
            writer.align(4);
            return;
        }

        for animation_data in &mut self.animation_data {
            writer.write_unsigned_byte(animation_data.bone_index as u8); // bone
            writer.write_unsigned_byte(animation_data.flag_mask()); // flags
            self.write_offset_index = writer.write_index_short(); // offset

            animation_data.write_to_writer(writer);

            writer.write_to_index_short(self.write_offset_index, (writer.get_size() - self.write_start_index) as i16);
        }

        writer.write_to_index_short(self.write_offset_index, 0);

        writer.align(4);
    }
}

impl Animation {
    pub fn new() -> Self {
        Self {
            write_start_index: usize::MAX,
            animation_data: Vec::new(),
            write_offset_index: usize::MAX,
        }
    }
}

pub struct AnimationData {
    pub bone_index: usize,
    pub animation_position: Option<ProcessedAnimationPosition>,
    pub animation_rotation: Option<ProcessedAnimationRotation>,
}

impl StructWriting for AnimationData {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        match &self.animation_position {
            Some(value) => match value {
                ProcessedAnimationPosition::Raw(vector) => {
                    writer.write_vector48(vector);
                }
                ProcessedAnimationPosition::Compressed => todo!(),
            },
            None => {}
        }

        match &self.animation_rotation {
            Some(value) => match value {
                ProcessedAnimationRotation::Raw(angle) => {
                    writer.write_quaternion64(angle);
                }
                ProcessedAnimationRotation::Compressed => todo!(),
            },
            None => {}
        }
    }
}

impl AnimationData {
    pub fn new() -> Self {
        Self {
            bone_index: 0,
            animation_position: None,
            animation_rotation: None,
        }
    }

    fn flag_mask(&self) -> u8 {
        let mut mask = 0;

        match &self.animation_position {
            Some(value) => match value {
                ProcessedAnimationPosition::Raw(_) => {
                    mask |= 0x01;
                }
                ProcessedAnimationPosition::Compressed => todo!(),
            },
            None => {}
        }

        match &self.animation_rotation {
            Some(value) => match value {
                ProcessedAnimationRotation::Raw(_) => {
                    mask |= 0x20;
                }
                ProcessedAnimationRotation::Compressed => todo!(),
            },
            None => {}
        }

        mask
    }
}

pub struct SequenceDescription {
    write_start_index: usize,
    pub name: String,
    write_animation_index_array_index: usize,
    pub animation_indexes: Vec<usize>,
    pub weight_list: Vec<f64>,
    write_weight_list_index: usize,
}

impl StructWriting for SequenceDescription {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.write_start_index = writer.get_size();

        writer.write_int(-(writer.get_size() as i32)); // baseptr
        writer.add_string_to_table(self.write_start_index, &self.name); // szlabelindex
        writer.add_string_to_table(self.write_start_index, ""); // szactivitynameindex
        writer.write_int(0); // flags
        writer.write_int(-1); // activity
        writer.write_int(0); // actweight
        writer.write_int(0); // numevents
        writer.write_int(self.write_start_index as i32); // eventindex
        writer.write_vector3(&Vector3::new(10.0, 10.0, 0.0)); // bbmin
        writer.write_vector3(&Vector3::new(-10.0, -10.0, 10.0)); // bbmax
        writer.write_int(1); // numblends
        self.write_animation_index_array_index = writer.write_index(); // animindexindex
        writer.write_int(0); // movementindex
        writer.write_int_array(&vec![1; 2]); // groupsize
        writer.write_int_array(&vec![-1; 2]); // paramindex
        writer.write_float_array(&vec![0.0; 2]); // paramstart
        writer.write_float_array(&vec![0.0; 2]); // paramend
        writer.write_int(0); // paramparent
        writer.write_float(0.2); // fadeintime
        writer.write_float(0.2); // fadeouttime
        writer.write_int(0); // localentrynode
        writer.write_int(0); // localexitnode
        writer.write_int(0); // nodeflags
        writer.write_float(0.0); // entryphase
        writer.write_float(0.0); // exitphase
        writer.write_float(0.0); // lastframe
        writer.write_int(0); // nextseq
        writer.write_int(0); // pose
        writer.write_int(0); // numikrules
        writer.write_int(0); // numautolayers
        writer.write_int(self.write_animation_index_array_index as i32); // autolayerindex
        self.write_weight_list_index = writer.write_index(); // weightlistindex
        writer.write_int(0); // posekeyindex
        writer.write_int(0); // numiklocks
        writer.write_int(0); // iklockindex
        writer.write_int(0); // keyvalueindex
        writer.write_int(0); // keyvaluesize
        writer.write_int(0); // cycleposeindex
        writer.write_int(0); // activitymodifierindex
        writer.write_int(0); // numactivitymodifiers
        writer.write_int_array(&vec![0; 5]); // unused
    }
}

impl SequenceDescription {
    pub fn new() -> Self {
        Self {
            write_start_index: usize::MAX,
            name: String::new(),
            animation_indexes: Vec::new(),
            write_animation_index_array_index: usize::MAX,
            weight_list: Vec::new(),
            write_weight_list_index: usize::MAX,
        }
    }

    fn write_animation_indexes(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_animation_index_array_index, (writer.get_size() - self.write_start_index) as i32);
        writer.write_short_array(&self.animation_indexes.to_vec().iter().map(|x| *x as i16).collect());
    }

    fn write_weight_list(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_weight_list_index, (writer.get_size() - self.write_start_index) as i32);
        writer.write_float_array(&self.weight_list.to_vec().iter().map(|x| *x as f32).collect());
    }
}

pub struct BodyPart {
    write_start_index: usize,
    pub name: String,
    pub base: i32,
    pub models: Vec<Model>,
    write_models_index: usize,
}

impl StructWriting for BodyPart {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.write_start_index = writer.get_size();

        writer.add_string_to_table(self.write_start_index, &self.name); // sznameindex
        writer.write_int(self.models.len() as i32); // nummodels
        writer.write_int(self.base); // base
        self.write_models_index = writer.write_index(); // modelindex
    }
}

impl BodyPart {
    pub fn new() -> Self {
        Self {
            write_start_index: usize::MAX,
            name: String::new(),
            base: 1,
            models: Vec::new(),
            write_models_index: usize::MAX,
        }
    }

    fn write_models(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_models_index, (writer.get_size() - self.write_start_index) as i32);

        for model in &mut self.models {
            model.write_to_writer(writer);
        }

        for model in &mut self.models {
            model.write_meshes(writer);
        }
    }
}

pub struct Model {
    write_start_index: usize,
    pub name: String,
    pub meshes: Vec<Mesh>,
    write_meshes_index: usize,
    pub vertex_count: i32,
    pub vertex_index: i32,
    pub tangent_index: i32,
}

impl StructWriting for Model {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.write_start_index = writer.get_size();

        writer.write_string(&self.name, 64); // name
        writer.write_int(0); // type
        writer.write_float(0.0); // boundingradius
        writer.write_int(self.meshes.len() as i32); // nummeshes
        self.write_meshes_index = writer.write_index(); // meshindex
        writer.write_int(self.vertex_count); // numvertices
        writer.write_int(self.vertex_index); // vertexindex
        writer.write_int(self.tangent_index); // tangentindex
        writer.write_int(0); // numattachments
        writer.write_int(0); // attachmentindex
        writer.write_int(0); // numeyeballs
        writer.write_int(0); // eyeballindex
        writer.write_int(0); // pVertexData
        writer.write_int(0); // pTangentData
        writer.write_int_array(&vec![0; 8]); // unused
    }
}

impl Model {
    pub fn new() -> Self {
        Self {
            write_start_index: usize::MAX,
            name: String::new(),
            meshes: Vec::new(),
            write_meshes_index: usize::MAX,
            vertex_count: 0,
            vertex_index: 0,
            tangent_index: 0,
        }
    }

    fn write_meshes(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.write_meshes_index, (writer.get_size() - self.write_start_index) as i32);

        for mesh in &mut self.meshes {
            mesh.write_model_index = self.write_start_index;
            mesh.write_to_writer(writer);
        }
    }
}

pub struct Mesh {
    write_start_index: usize,
    write_model_index: usize,
    pub material_index: usize,
    pub vertex_count: usize,
    pub vertex_index: usize,
    pub mesh_id: usize,
}

impl StructWriting for Mesh {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.write_start_index = writer.get_size();

        writer.write_int(self.material_index as i32); // material
        writer.write_int(-(self.write_start_index as i32 - self.write_model_index as i32)); // modelindex
        writer.write_int(self.vertex_count as i32); // numvertices
        writer.write_int(self.vertex_index as i32); // vertexindex
        writer.write_int(0); // numflexes
        writer.write_int(0); // flexindex
        writer.write_int(0); // materialtype
        writer.write_int(0); // materialparam
        writer.write_int(self.mesh_id as i32); // meshid
        writer.write_vector3(&Vector3::default()); // center
        writer.write_int(0); // modelvertexdata
        writer.write_int_array(&vec![self.vertex_count as i32; 8]); // numLODVertexes
        writer.write_int_array(&vec![0; 8]); // unused
    }
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            write_start_index: usize::MAX,
            write_model_index: usize::MAX,
            material_index: 0,
            vertex_count: 0,
            vertex_index: 0,
            mesh_id: 0,
        }
    }
}

pub struct Material {
    write_start_index: usize,
    pub name: String,
}

impl StructWriting for Material {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.write_start_index = writer.get_size();

        writer.add_string_to_table(self.write_start_index, &self.name); // sznameindex
        writer.write_int(0); // flags
        writer.write_int(0); // used
        writer.write_int(0); // unused1
        writer.write_int(0); // material
        writer.write_int(0); // clientmaterial
        writer.write_int_array(&vec![0; 10]); // unused
    }
}

impl Material {
    pub fn new() -> Self {
        Self {
            write_start_index: usize::MAX,
            name: String::new(),
        }
    }
}
