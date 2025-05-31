use crate::process::MAX_HARDWARE_BONES_PER_STRIP;

use super::{FileWriteError, FileWriter};
use bitflags::bitflags;

#[derive(Debug, Default)]
pub struct Header {
    pub this: usize,
    pub version: i32,
    pub vertex_cache_size: i32,
    pub max_bones_per_strip: u16,
    pub max_bones_per_triangle: u16,
    pub max_bones_per_vertex: i32,
    pub checksum: i32,
    pub material_replacement_lists: Vec<MaterialReplacementListHeader>,
    pub material_replacement_list_offset: usize,
    pub body_parts: Vec<BodyPartHeader>,
    pub body_part_offset: usize,
}

impl Header {
    pub fn write_data(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.this = writer.this();

        writer.write_integer(self.version);
        writer.write_integer(self.vertex_cache_size);
        writer.write_unsigned_short(self.max_bones_per_strip);
        writer.write_unsigned_short(self.max_bones_per_triangle);
        writer.write_integer(self.max_bones_per_vertex);
        writer.write_integer(self.checksum);
        writer.write_array_size_integer(&self.material_replacement_lists)?;
        self.material_replacement_list_offset = writer.write_integer_index();
        writer.write_array_size_integer(&self.body_parts)?;
        self.body_part_offset = writer.write_integer_index();

        writer.write_to_integer_offset(self.body_part_offset, writer.this() - self.this)?;

        for body_part in &mut self.body_parts {
            body_part.write_data(writer)?;
        }

        for body_part in &mut self.body_parts {
            body_part.write_models(writer)?;
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                model.write_lods(writer)?;
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                for lod in &mut model.model_lods {
                    lod.write_meshes(writer)?;
                }
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                for lod in &mut model.model_lods {
                    for mesh in &mut lod.meshes {
                        mesh.write_strip_groups(writer)?;
                    }
                }
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                for lod in &mut model.model_lods {
                    for mesh in &mut lod.meshes {
                        for strip_group in &mut mesh.strip_groups {
                            strip_group.write_strips(writer)?;
                        }
                    }
                }
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                for lod in &mut model.model_lods {
                    for mesh in &mut lod.meshes {
                        for strip_group in &mut mesh.strip_groups {
                            strip_group.write_vertices(writer)?;
                        }
                    }
                }
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                for lod in &mut model.model_lods {
                    for mesh in &mut lod.meshes {
                        for strip_group in &mut mesh.strip_groups {
                            strip_group.write_indices(writer)?;
                        }
                    }
                }
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                for lod in &mut model.model_lods {
                    for mesh in &mut lod.meshes {
                        for strip_group in &mut mesh.strip_groups {
                            for strip in &mut strip_group.strips {
                                strip.write_bone_changes(writer)?;
                            }
                        }
                    }
                }
            }
        }

        writer.write_to_integer_offset(self.material_replacement_list_offset, writer.this() - self.this)?;

        for material_replacement_list in &mut self.material_replacement_lists {
            material_replacement_list.write_data(writer)?;
        }

        for material_replacement_list in &mut self.material_replacement_lists {
            material_replacement_list.write_material_replacements(writer)?;
        }

        writer.write_string_table()?;

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct MaterialReplacementListHeader {
    pub this: usize,
    pub material_replacements: Vec<MaterialReplacementHeader>,
    pub material_replacement_index: usize,
}

impl MaterialReplacementListHeader {
    fn write_data(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.this = writer.this();

        writer.write_array_size_integer(&self.material_replacements)?;
        self.material_replacement_index = writer.write_integer_index();

        Ok(())
    }

    fn write_material_replacements(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.material_replacement_index, writer.this() - self.this)?;

        for replacement in &mut self.material_replacements {
            replacement.write_data(writer);
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct MaterialReplacementHeader {
    pub this: usize,
    pub material_id: i16,
    pub replacement_material_name: String,
}

impl MaterialReplacementHeader {
    fn write_data(&mut self, writer: &mut FileWriter) {
        self.this = writer.this();

        debug_assert!(self.material_id >= 0);
        writer.write_short(self.material_id);
        writer.write_string_to_table(self.this, &self.replacement_material_name);
    }
}

#[derive(Debug, Default)]
pub struct BodyPartHeader {
    pub this: usize,
    pub models: Vec<ModelHeader>,
    pub model_index: usize,
}

impl BodyPartHeader {
    fn write_data(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.this = writer.this();

        writer.write_array_size_integer(&self.models)?;
        self.model_index = writer.write_integer_index();

        Ok(())
    }

    fn write_models(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.model_index, writer.this() - self.this)?;

        for model in &mut self.models {
            model.write_data(writer)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ModelHeader {
    pub this: usize,
    pub model_lods: Vec<ModelLODHeader>,
    pub model_lod_index: usize,
}

impl ModelHeader {
    fn write_data(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.this = writer.this();

        writer.write_array_size_integer(&self.model_lods)?;
        self.model_lod_index = writer.write_integer_index();

        Ok(())
    }

    fn write_lods(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.model_lod_index, writer.this() - self.this)?;

        for lod in &mut self.model_lods {
            lod.write_data(writer)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ModelLODHeader {
    pub this: usize,
    pub meshes: Vec<MeshHeader>,
    pub mesh_index: usize,
    pub switch_point: f32,
}

impl ModelLODHeader {
    fn write_data(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.this = writer.this();

        writer.write_array_size_integer(&self.meshes)?;
        self.mesh_index = writer.write_integer_index();
        debug_assert!(self.switch_point.is_finite());
        writer.write_float(self.switch_point);

        Ok(())
    }

    fn write_meshes(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.mesh_index, writer.this() - self.this)?;

        for mesh in &mut self.meshes {
            mesh.write_data(writer)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct MeshHeader {
    pub this: usize,
    pub strip_groups: Vec<StripGroupHeader>,
    pub strip_group_index: usize,
    pub flags: MeshHeaderFlags,
}

impl MeshHeader {
    fn write_data(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.this = writer.this();

        writer.write_array_size_integer(&self.strip_groups)?;
        self.strip_group_index = writer.write_integer_index();
        debug_assert!(!self.flags.contains(MeshHeaderFlags::IS_TEETH | MeshHeaderFlags::IS_EYES));
        writer.write_unsigned_byte(self.flags.bits());

        Ok(())
    }

    fn write_strip_groups(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.strip_group_index, writer.this() - self.this)?;

        for strip_group in &mut self.strip_groups {
            strip_group.write_data(writer)?;
        }

        Ok(())
    }
}

bitflags! {
    #[derive(Debug, Default)]
    pub struct MeshHeaderFlags: u8 {
        const IS_TEETH = 0x01;
        const IS_EYES = 0x02;
    }
}

#[derive(Debug, Default)]
pub struct StripGroupHeader {
    pub this: usize,
    pub vertices: Vec<Vertex>,
    pub vertex_index: usize,
    pub indices: Vec<u16>,
    pub index_index: usize,
    pub strips: Vec<StripHeader>,
    pub strip_index: usize,
    pub flags: StripGroupHeaderFlags,
}

impl StripGroupHeader {
    fn write_data(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.this = writer.this();

        writer.write_array_size_integer(&self.vertices)?;
        self.vertex_index = writer.write_integer_index();
        writer.write_array_size_integer(&self.indices)?;
        self.index_index = writer.write_integer_index();
        writer.write_array_size_integer(&self.strips)?;
        self.strip_index = writer.write_integer_index();
        writer.write_unsigned_byte(self.flags.bits());

        Ok(())
    }

    fn write_strips(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.strip_index, writer.this() - self.this)?;

        for strip in &mut self.strips {
            strip.write_data(writer)?;
        }

        Ok(())
    }

    fn write_vertices(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.vertex_index, writer.this() - self.this)?;

        for vertex in &mut self.vertices {
            vertex.write_data(writer);
        }

        Ok(())
    }

    fn write_indices(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.index_index, writer.this() - self.this)?;

        writer.write_unsigned_short_array(&self.indices);

        Ok(())
    }
}

bitflags! {
    #[derive(Debug, Default)]
    pub struct StripGroupHeaderFlags: u8 {
        const IS_FLEXED           = 0x01;
        const IS_HARDWARE_SKINNED = 0x02;
        const IS_DELTA_FLEXED     = 0x04;
    }
}

#[derive(Debug, Default)]
pub struct Vertex {
    pub this: usize,
    pub bone_count: u8,
    pub vertex_id: u16,
    pub bone_ids: [u8; 3],
}

impl Vertex {
    fn write_data(&mut self, writer: &mut FileWriter) {
        self.this = writer.this();

        writer.write_unsigned_byte_array(&[0, 1, 2]);
        debug_assert!(self.bone_count >= 1);
        writer.write_unsigned_byte(self.bone_count);
        writer.write_unsigned_short(self.vertex_id);
        writer.write_unsigned_byte_array(&self.bone_ids);
    }
}

#[derive(Debug, Default)]
pub struct StripHeader {
    pub this: usize,
    pub indices_count: i32,
    pub indices_offset: i32,
    pub vertices_count: i32,
    pub vertices_offset: i32,
    pub bone_count: i16,
    pub flags: StripHeaderFlags,
    pub bone_state_changes: Vec<BoneStateChangeHeader>,
    pub bone_state_change_index: usize,
}

impl StripHeader {
    fn write_data(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.this = writer.this();

        debug_assert!(self.indices_count >= 0);
        writer.write_integer(self.indices_count);
        debug_assert!(self.indices_offset >= 0);
        writer.write_integer(self.indices_offset);
        debug_assert!(self.vertices_count >= 0);
        writer.write_integer(self.vertices_count);
        debug_assert!(self.vertices_offset >= 0);
        writer.write_integer(self.vertices_offset);
        debug_assert!(self.bone_count >= 1);
        writer.write_short(self.bone_count);
        debug_assert!(!self.flags.is_empty());
        writer.write_unsigned_byte(self.flags.bits());
        debug_assert!(self.bone_state_changes.len() <= MAX_HARDWARE_BONES_PER_STRIP);
        writer.write_array_size_integer(&self.bone_state_changes)?;
        self.bone_state_change_index = writer.write_integer_index();

        Ok(())
    }

    fn write_bone_changes(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.bone_state_change_index, writer.this() - self.this)?;

        for bone_state_change in &mut self.bone_state_changes {
            bone_state_change.write_data(writer);
        }
        Ok(())
    }
}

bitflags! {
    #[derive(Debug, Default)]
    pub struct StripHeaderFlags: u8 {
        const IS_TRIANGLE_LIST  = 0x01;
    }
}

#[derive(Debug, Default)]
pub struct BoneStateChangeHeader {
    pub this: usize,
    pub hardware_id: i32,
    pub bone_table_index: i32,
}

impl BoneStateChangeHeader {
    fn write_data(&mut self, writer: &mut FileWriter) {
        self.this = writer.this();

        debug_assert!(self.hardware_id >= 0);
        writer.write_integer(self.hardware_id);
        debug_assert!(self.bone_table_index >= 0);
        writer.write_integer(self.bone_table_index);
    }
}
