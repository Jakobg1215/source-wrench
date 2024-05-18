use crate::utilities::binarydata::DataWriter;

use super::StructWriting;

pub struct MeshHeader {
    checksum: i32,
    material_replacement_index: usize,
    pub body_groups: Vec<MeshBodyGroup>,
    body_groups_index: usize,
}

impl StructWriting for MeshHeader {
    fn write_to_writer(&mut self, mut writer: &mut DataWriter) {
        writer.write_int(7); // version
        writer.write_int(24); // vertCacheSize
        writer.write_unsigned_short(53); // maxBonesPerStrip
        writer.write_unsigned_short(9); // maxBonesPerTri
        writer.write_int(3); // maxBonesPerVert
        writer.write_int(self.checksum); // checkSum
        writer.write_int(1); // numLODs
        self.material_replacement_index = writer.write_index(); // materialReplacementListOffset
        writer.write_int(self.body_groups.len() as i32); // numBodyParts
        self.body_groups_index = writer.write_index(); // bodyPartOffset

        writer.write_to_index(self.body_groups_index, writer.get_size() as i32);
        for body_group in &mut self.body_groups {
            body_group.write_to_writer(&mut writer);
        }

        for body_group in &mut self.body_groups {
            body_group.write_parts(&mut writer);
        }

        writer.write_to_index(self.material_replacement_index, writer.get_size() as i32);
        MeshMaterialReplacementListHeader::new().write_to_writer(&mut writer);
    }
}

impl MeshHeader {
    pub fn new(checksum: i32) -> Self {
        Self {
            checksum,
            material_replacement_index: usize::MAX,
            body_groups: Vec::new(),
            body_groups_index: usize::MAX,
        }
    }
}

pub struct MeshMaterialReplacementListHeader {}

impl StructWriting for MeshMaterialReplacementListHeader {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        writer.write_int(0); // numReplacements
        writer.write_int(-8); // replacementOffset
    }
}

impl MeshMaterialReplacementListHeader {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct MeshBodyGroup {
    index_start: usize,
    pub parts: Vec<MeshBodyPart>,
    parts_index: usize,
}

impl StructWriting for MeshBodyGroup {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.index_start = writer.get_size();

        writer.write_int(self.parts.len() as i32); // numModels
        self.parts_index = writer.write_index(); // modelOffset
    }
}

impl MeshBodyGroup {
    pub fn new() -> Self {
        Self {
            index_start: usize::MAX,
            parts: Vec::new(),
            parts_index: usize::MAX,
        }
    }

    fn write_parts(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.parts_index, (writer.get_size() - self.index_start) as i32);

        for part in &mut self.parts {
            part.write_to_writer(writer);
        }

        for part in &mut self.parts {
            part.write_models(writer);
        }
    }
}

pub struct MeshBodyPart {
    index_start: usize,
    pub models: Vec<MeshBodyModel>,
    models_index: usize,
}

impl StructWriting for MeshBodyPart {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.index_start = writer.get_size();

        writer.write_int(self.models.len() as i32); // numLODs
        self.models_index = writer.write_index(); // lodOffset
    }
}

impl MeshBodyPart {
    pub fn new() -> Self {
        Self {
            index_start: usize::MAX,
            models: Vec::new(),
            models_index: usize::MAX,
        }
    }

    fn write_models(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.models_index, (writer.get_size() - self.index_start) as i32);

        for model in &mut self.models {
            model.write_to_writer(writer);
        }

        for model in &mut self.models {
            model.write_meshes(writer);
        }
    }
}

pub struct MeshBodyModel {
    index_start: usize,
    pub meshes: Vec<MeshBodyMesh>,
    meshes_index: usize,
    switch_point: f64,
}

impl StructWriting for MeshBodyModel {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.index_start = writer.get_size();

        writer.write_int(self.meshes.len() as i32); // numMeshes
        self.meshes_index = writer.write_index(); // meshOffset
        writer.write_float(self.switch_point as f32); // switchPoint
    }
}

impl MeshBodyModel {
    pub fn new(switch_point: f64) -> Self {
        Self {
            index_start: usize::MAX,
            meshes: Vec::new(),
            meshes_index: usize::MAX,
            switch_point,
        }
    }

    fn write_meshes(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.meshes_index, (writer.get_size() - self.index_start) as i32);

        for mesh in &mut self.meshes {
            mesh.write_to_writer(writer);
        }

        for mesh in &mut self.meshes {
            mesh.write_strip_groups(writer);
        }
    }
}

pub struct MeshBodyMesh {
    index_start: usize,
    pub strip_groups: Vec<MeshStripGroup>,
    strip_groups_index: usize,
}

impl StructWriting for MeshBodyMesh {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.index_start = writer.get_size();

        writer.write_int(self.strip_groups.len() as i32); // numStripGroups
        self.strip_groups_index = writer.write_index(); // stripGroupHeaderOffset
        writer.write_unsigned_byte(0); // flags
    }
}

impl MeshBodyMesh {
    pub fn new() -> Self {
        Self {
            index_start: usize::MAX,
            strip_groups: Vec::new(),
            strip_groups_index: usize::MAX,
        }
    }

    fn write_strip_groups(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.strip_groups_index, (writer.get_size() - self.index_start) as i32);

        for strip_group in &mut self.strip_groups {
            strip_group.write_to_writer(writer);
        }

        for strip_group in &mut self.strip_groups {
            strip_group.write_strip(writer);
        }

        for strip_group in &mut self.strip_groups {
            strip_group.write_vertices(writer);
        }

        for strip_group in &mut self.strip_groups {
            strip_group.write_indices(writer);
        }

        for strip_group in &mut self.strip_groups {
            strip_group.write_bone_state_change(writer);
        }
    }
}

pub struct MeshStripGroup {
    index_start: usize,
    pub vertices: Vec<MeshVertex>,
    vertices_index: usize,
    pub indices: Vec<u16>,
    indices_index: usize,
    pub strips: Vec<MeshStrip>,
    strips_index: usize,
}

impl StructWriting for MeshStripGroup {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.index_start = writer.get_size();

        writer.write_int(self.vertices.len() as i32); // numVerts
        self.vertices_index = writer.write_index(); // vertOffset
        writer.write_int(self.indices.len() as i32); // numIndices
        self.indices_index = writer.write_index(); // indexOffset
        writer.write_int(self.strips.len() as i32); // numStrips
        self.strips_index = writer.write_index(); // stripOffset
        writer.write_unsigned_byte(2); // flags
    }
}

impl MeshStripGroup {
    pub fn new() -> Self {
        Self {
            index_start: usize::MAX,
            vertices: Vec::new(),
            vertices_index: usize::MAX,
            indices: Vec::new(),
            indices_index: usize::MAX,
            strips: Vec::new(),
            strips_index: usize::MAX,
        }
    }

    fn write_strip(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.strips_index, (writer.get_size() - self.index_start) as i32);

        for strip in &mut self.strips {
            strip.write_to_writer(writer);
        }
    }

    fn write_vertices(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.vertices_index, (writer.get_size() - self.index_start) as i32);

        for vertex in &mut self.vertices {
            vertex.write_to_writer(writer);
        }
    }

    fn write_indices(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.indices_index, (writer.get_size() - self.index_start) as i32);
        self.indices.reverse(); // FIXME: This is a hack to fix inverted faces.
        writer.write_unsigned_short_array(&self.indices);
    }

    fn write_bone_state_change(&mut self, writer: &mut DataWriter) {
        for strip in &mut self.strips {
            writer.write_to_index(strip.bone_state_changes_index, (writer.get_size() - strip.index_start) as i32);
            for bone_state_change in &mut strip.bone_state_changes {
                bone_state_change.write_to_writer(writer);
            }
        }
    }
}

pub struct MeshVertex {
    pub bone_count: usize,
    pub vertex_index: usize,
    pub bone_weight_bones: [usize; 3],
}

impl StructWriting for MeshVertex {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        writer.write_unsigned_byte_array(&vec![0, 1, 2]); // boneWeightIndex
        writer.write_unsigned_byte(self.bone_count as u8); // numBones
        writer.write_unsigned_short(self.vertex_index as u16); // origMeshVertID
        writer.write_unsigned_byte_array(&self.bone_weight_bones.map(|index| index as u8).to_vec());
        // boneID
    }
}

impl MeshVertex {
    pub fn new() -> Self {
        Self {
            bone_count: usize::MAX,
            vertex_index: usize::MAX,
            bone_weight_bones: [usize::MAX; 3],
        }
    }
}

pub struct MeshStrip {
    index_start: usize,
    pub indices_count: i32,
    pub indices_offset: i32,
    pub vertices_count: i32,
    pub vertices_offset: i32,
    pub bone_count: i16,
    pub bone_state_changes: Vec<MeshBoneStateChange>,
    bone_state_changes_index: usize,
}

impl StructWriting for MeshStrip {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.index_start = writer.get_size();

        writer.write_int(self.indices_count); // numIndices
        writer.write_int(self.indices_offset); // indexOffset
        writer.write_int(self.vertices_count); // numVerts
        writer.write_int(self.vertices_offset); // vertOffset
        writer.write_short(self.bone_count); // numBones
        writer.write_unsigned_byte(1); // flags
        writer.write_int(self.bone_state_changes.len() as i32); // numBoneStateChanges
        self.bone_state_changes_index = writer.write_index(); // boneStateChangeOffset
    }
}

impl MeshStrip {
    pub fn new(indices_count: i32, indices_offset: i32, vertices_count: i32, vertices_offset: i32, bone_count: i16) -> Self {
        Self {
            index_start: 0,
            indices_count,
            indices_offset,
            vertices_count,
            vertices_offset,
            bone_count,
            bone_state_changes: Vec::new(),
            bone_state_changes_index: 0,
        }
    }
}

pub struct MeshBoneStateChange {
    hardware_id: usize,
    new_bone_id: usize,
}

impl StructWriting for MeshBoneStateChange {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        writer.write_int(self.hardware_id as i32); // hardwareID
        writer.write_int(self.new_bone_id as i32); // newBoneID
    }
}

impl MeshBoneStateChange {
    pub fn new(hardware_id: usize, new_bone_id: usize) -> Self {
        Self { hardware_id, new_bone_id }
    }
}
