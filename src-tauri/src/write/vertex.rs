use crate::utilities::mathematics::{Vector2, Vector3, Vector4};

use super::{FileWriteError, FileWriter, WriteToWriter, MAX_LOD_COUNT};

#[derive(Debug, Default)]
pub struct VertexFileHeader {
    pub version: i32,
    pub checksum: i32,
    pub lod_count: i32,
    pub lod_vertex_count: [i32; MAX_LOD_COUNT],
    pub fixups: Vec<VertexFileFixup>,
    pub fixup_offset: usize,
    pub vertices: Vec<VertexFileVertex>,
    pub vertex_offset: usize,
    pub tangents: Vec<Vector4>,
    pub tangent_offset: usize,
}

const VERTEX_FILE_IDENTIFIER: i32 = (86 << 24) + (83 << 16) + (68 << 8) + 73;

impl WriteToWriter for VertexFileHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
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
        debug_assert!(
            self.lod_vertex_count.windows(2).all(|count| count[0] >= count[1]),
            "LOD Vertex Count Is Non-Descending! self.lod_vertex_count: {:?}",
            self.lod_vertex_count
        );
        writer.write_integer_array(&self.lod_vertex_count);
        writer.write_array_size(self.fixups.len())?;
        self.fixup_offset = writer.write_integer_index();
        debug_assert!(
            self.vertices.len() == self.lod_vertex_count[0] as usize,
            "Vertices Count Not The Same As LOD Count! self.vertices.len(): {} self.lod_vertex_count[0]: {}",
            self.vertices.len(),
            self.lod_vertex_count[0]
        );
        self.vertex_offset = writer.write_integer_index();
        debug_assert!(
            self.tangents.len() == self.lod_vertex_count[0] as usize,
            "Tangents Count Not The Same As LOD Count! self.tangents.len(): {} self.lod_vertex_count[0]: {}",
            self.tangents.len(),
            self.lod_vertex_count[0]
        );
        self.tangent_offset = writer.write_integer_index();

        writer.write_to_integer_offset(self.fixup_offset, writer.data.len())?;

        for fixup in &mut self.fixups {
            fixup.write(writer)?;
        }

        writer.write_to_integer_offset(self.vertex_offset, writer.data.len())?;

        for vertex in &mut self.vertices {
            vertex.write(writer)?;
        }

        writer.write_to_integer_offset(self.tangent_offset, writer.data.len())?;

        for tangent in &self.tangents {
            debug_assert!(tangent.is_finite(), "Tangent Is Not Finite! tangent: {:?}", tangent);
            debug_assert!(
                Vector3::new(tangent.x, tangent.y, tangent.z).is_normalized(),
                "Tangent Is Not Normalized! tangent: {:?}",
                tangent
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
pub struct VertexFileFixup {
    pub lod: i32,
    pub vertex_index: i32,
    pub vertex_count: i32,
}

impl WriteToWriter for VertexFileFixup {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
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

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct VertexFileVertex {
    pub weights: [f32; 3],
    pub bones: [u8; 3],
    pub bone_count: u8,
    pub position: Vector3,
    pub normal: Vector3,
    pub texture_coordinate: Vector2,
}

impl WriteToWriter for VertexFileVertex {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        debug_assert!(
            (self.weights.iter().sum::<f32>() - 1.0).abs() < f32::EPSILON,
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

        Ok(())
    }
}
