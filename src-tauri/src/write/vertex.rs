use crate::utilities::mathematics::{Vector2, Vector3, Vector4};

use super::{FileWriteError, FileWriter, WriteToWriter};

pub struct VertexFileHeader {
    checksum: i32,
    fixups: Vec<VerticesFixUp>,
    fixups_index: usize,
    pub vertexes: Vec<Vertex>,
    vertexes_index: usize,
    pub tangents: Vec<Vector4>,
    tangents_index: usize,
}

impl WriteToWriter for VertexFileHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_integer(1448297545); // id
        writer.write_integer(4); // version
        writer.write_integer(self.checksum); // checksum
        writer.write_integer(1); // numLODs
        writer.write_integer_array(&[self.vertexes.len() as i32; 8]); // numLODVertexes
        writer.write_integer(self.fixups.len() as i32); // numFixups
        self.fixups_index = writer.write_integer_index(); // fixupTableStart
        self.vertexes_index = writer.write_integer_index(); // vertexDataStart
        self.tangents_index = writer.write_integer_index(); // tangentDataStart

        writer.write_to_integer_offset(self.fixups_index, writer.data.len())?;

        writer.write_to_integer_offset(self.vertexes_index, writer.data.len())?;
        for vertex in &mut self.vertexes {
            vertex.write(writer)?;
        }

        writer.write_to_integer_offset(self.tangents_index, writer.data.len())?;
        for tangent in &self.tangents {
            writer.write_vector4(*tangent);
        }
        Ok(())
    }
}

impl VertexFileHeader {
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

pub struct Vertex {
    weights: [f64; 3],
    bones: [usize; 3],
    bone_count: usize,
    position: Vector3,
    normal: Vector3,
    uv: Vector2,
}

impl WriteToWriter for Vertex {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_float_array(&[self.weights[0] as f32, self.weights[1] as f32, self.weights[2] as f32]); // weight
        writer.write_unsigned_byte_array(&[self.bones[0] as u8, self.bones[1] as u8, self.bones[2] as u8]); // bone
        writer.write_unsigned_byte(self.bone_count as u8); // numBones
        writer.write_vector3(self.position); // m_vecPosition
        writer.write_vector3(self.normal); // m_vecNormal
        writer.write_vector2(self.uv); // m_vecTexCoord
        Ok(())
    }
}

impl Vertex {
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
