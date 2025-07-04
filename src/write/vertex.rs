use crate::utilities::mathematics::{Vector2, Vector3, Vector4};

use super::{FileWriteError, FileWriter, MAX_LOD_COUNT};

#[derive(Debug, Default)]
pub struct Header {
    pub this: usize,
    pub version: i32,
    pub checksum: i32,
    pub lod_count: i32,
    pub lod_vertex_count: [i32; MAX_LOD_COUNT],
    pub fixups: Vec<Fixup>,
    pub fixup_index: usize,
    pub vertices: Vec<Vertex>,
    pub vertex_index: usize,
    pub tangents: Vec<Vector4>,
    pub tangent_index: usize,
}

const VERTEX_FILE_IDENTIFIER: i32 = (86 << 24) + (83 << 16) + (68 << 8) + 73;

impl Header {
    pub fn write_data(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.this = writer.this();

        writer.write_integer(VERTEX_FILE_IDENTIFIER);
        writer.write_integer(self.version);
        writer.write_integer(self.checksum);
        debug_assert!(self.lod_count > 0, "LOD Count Is Less Than 1! self.lod_count: {}", self.lod_count);
        debug_assert!(
            self.lod_count <= MAX_LOD_COUNT as i32,
            "LOD Count Is Greater Than {}! self.lod_count: {}",
            MAX_LOD_COUNT,
            self.lod_count
        );
        writer.write_integer(self.lod_count);
        debug_assert!(
            self.lod_vertex_count.iter().all(|&count| count >= 0),
            "Vertex LOD Count Is Less Than 0!, self.lod_vertex_count: {:?}",
            self.lod_vertex_count
        );
        writer.write_integer_array(&self.lod_vertex_count);
        writer.write_array_size_integer(&self.fixups)?;
        self.fixup_index = writer.write_integer_index();
        debug_assert!(
            self.vertices.len() == self.lod_vertex_count[0] as usize,
            "Vertices Count Not The Same As LOD Count! self.vertices.len(): {} self.lod_vertex_count[0]: {}",
            self.vertices.len(),
            self.lod_vertex_count[0]
        );
        self.vertex_index = writer.write_integer_index();
        debug_assert!(
            self.tangents.len() == self.lod_vertex_count[0] as usize,
            "Tangents Count Not The Same As LOD Count! self.tangents.len(): {} self.lod_vertex_count[0]: {}",
            self.tangents.len(),
            self.lod_vertex_count[0]
        );
        self.tangent_index = writer.write_integer_index();

        writer.write_to_integer_offset(self.fixup_index, writer.this() - self.this)?;

        for fixup in &mut self.fixups {
            fixup.write_data(writer);
        }

        writer.write_to_integer_offset(self.vertex_index, writer.this() - self.this)?;

        for vertex in &mut self.vertices {
            vertex.write_data(writer);
        }

        writer.write_to_integer_offset(self.tangent_index, writer.this() - self.this)?;

        for tangent in &self.tangents {
            debug_assert!(tangent.is_finite(), "Tangent Is Not Finite! tangent: {tangent:?}");
            debug_assert!(
                Vector3::new(tangent.x, tangent.y, tangent.z).is_normalized(),
                "Tangent Is Not Normalized! tangent: {tangent:?}"
            );
            debug_assert!(
                tangent.w == 1.0 || tangent.w == -1.0,
                "Tangent Mirroring Value Not 1 or -1! tangent.w: {}",
                tangent.w
            );
            writer.write_vector4(*tangent);
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Fixup {
    pub this: usize,
    pub lod: i32,
    pub vertex_index: i32,
    pub vertex_count: i32,
}

impl Fixup {
    fn write_data(&mut self, writer: &mut FileWriter) {
        self.this = writer.this();

        debug_assert!(self.lod > 0, "LOD Is Less Than 1! self.lod: {}", self.lod);
        debug_assert!(
            self.lod <= MAX_LOD_COUNT as i32,
            "LOD Is Greater Than {}! self.lod: {}",
            MAX_LOD_COUNT,
            self.lod
        );
        writer.write_integer(self.lod);
        debug_assert!(self.vertex_index > 0, "Vertex Index Is Less Than 1! self.vertex_index: {}", self.vertex_index);
        writer.write_integer(self.vertex_index);
        debug_assert!(self.vertex_count > 0, "Vertex Count Is Less Than 1! self.vertex_count: {}", self.vertex_count);
        writer.write_integer(self.vertex_count);
    }
}

#[derive(Debug, Default)]
pub struct Vertex {
    pub this: usize,
    pub weights: [f32; 3],
    pub bones: [u8; 3],
    pub bone_count: u8,
    pub position: Vector3,
    pub normal: Vector3,
    pub texture_coordinate: Vector2,
}

impl Vertex {
    fn write_data(&mut self, writer: &mut FileWriter) {
        self.this = writer.this();

        debug_assert!(
            (self.weights.iter().sum::<f32>() - 1.0).abs() < 1e-6,
            "Vertex Weight Sum Not Equal To 1!: self.weights: {:?} sum: {}",
            self.weights,
            self.weights.iter().sum::<f32>()
        );
        writer.write_float_array(&self.weights);
        writer.write_unsigned_byte_array(&self.bones);
        debug_assert!(self.bone_count > 0, "Bone Count Is Less Than 1! self.bone_count: {}", self.bone_count);
        debug_assert!(self.bone_count <= 3, "Bone Count Is Greater Than 3! self.bone_count: {}", self.bone_count);
        writer.write_unsigned_byte(self.bone_count);
        debug_assert!(self.position.is_finite(), "Position Is Not Finite! self.position: {:?}", self.position);
        writer.write_vector3(self.position);
        debug_assert!(self.normal.is_finite(), "Normal Is Not Finite! self.normal: {:?}", self.normal);
        debug_assert!(self.normal.is_normalized(), "Normal Is Not Normalized! self.normal: {:?}", self.normal);
        writer.write_vector3(self.normal);
        debug_assert!(
            self.texture_coordinate.is_finite(),
            "Texture Coordinate Is Not Finite! self.texture_coordinate: {:?}",
            self.texture_coordinate
        );
        writer.write_vector2(self.texture_coordinate);
    }
}
