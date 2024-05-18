use crate::utilities::{
    binarydata::DataWriter,
    mathematics::{Vector2, Vector3, Vector4},
};

use super::StructWriting;

pub struct VerticesHeader {
    checksum: i32,
    fixups: Vec<VerticesFixUp>,
    fixups_index: usize,
    pub vertexes: Vec<VerticesVertex>,
    vertexes_index: usize,
    pub tangents: Vec<Vector4>,
    tangents_index: usize,
}

impl StructWriting for VerticesHeader {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        writer.write_int(1448297545); // id
        writer.write_int(4); // version
        writer.write_int(self.checksum); // checksum
        writer.write_int(1); // numLODs
        writer.write_int_array(&vec![self.vertexes.len() as i32; 8]); // numLODVertexes
        writer.write_int(self.fixups.len() as i32); // numFixups
        self.fixups_index = writer.write_index(); // fixupTableStart
        self.vertexes_index = writer.write_index(); // vertexDataStart
        self.tangents_index = writer.write_index(); // tangentDataStart

        writer.write_to_index(self.fixups_index, writer.get_size() as i32);

        writer.write_to_index(self.vertexes_index, writer.get_size() as i32);
        for vertex in &mut self.vertexes {
            vertex.write_to_writer(writer);
        }

        writer.write_to_index(self.tangents_index, writer.get_size() as i32);
        for tangent in &self.tangents {
            writer.write_vector4(tangent);
        }
    }
}

impl VerticesHeader {
    pub fn new(checksum: i32) -> Self {
        Self {
            checksum,
            fixups: Vec::new(),
            fixups_index: usize::MAX,
            vertexes: Vec::new(),
            vertexes_index: usize::MAX,
            tangents: Vec::new(),
            tangents_index: usize::MAX,
        }
    }
}

pub struct VerticesFixUp {}

pub struct VerticesVertex {
    weights: [f64; 3],
    bones: [usize; 3],
    bone_count: usize,
    position: Vector3,
    normal: Vector3,
    uv: Vector2,
}

impl StructWriting for VerticesVertex {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        writer.write_float_array(&self.weights.iter().map(|f| *f as f32).collect()); // weight
        writer.write_unsigned_byte_array(&self.bones.iter().map(|b| *b as u8).collect()); // bone
        writer.write_unsigned_byte(self.bone_count as u8); // numBones
        writer.write_vector3(&self.position); // m_vecPosition
        writer.write_vector3(&self.normal); // m_vecNormal
        writer.write_vector2(&self.uv); // m_vecTexCoord
    }
}

impl VerticesVertex {
    pub fn new(weights: [f64; 3], bones: [usize; 3], bone_count: usize, position: Vector3, normal: Vector3, uv: Vector2) -> Self {
        Self {
            weights,
            bones,
            bone_count,
            position,
            normal,
            uv,
        }
    }
}
