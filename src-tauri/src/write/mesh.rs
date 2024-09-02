use super::{FileWriteError, FileWriter, WriteToWriter};
use bitflags::bitflags;

#[derive(Debug, Default)]
pub struct MeshFileHeader {
    pub version: i32,
    pub vertex_cache_size: i32,
    pub max_bones_per_strip: u16,
    pub max_bones_per_triangle: u16,
    pub max_bones_per_vertex: i32,
    pub checksum: i32,
    pub material_replacement_lists: Vec<MeshFileMaterialReplacementListHeader>,
    pub material_replacement_list_offset: usize,
    pub body_parts: Vec<MeshFileBodyPartHeader>,
    pub body_part_offset: usize,
}

impl WriteToWriter for MeshFileHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_integer(self.version);
        writer.write_integer(self.vertex_cache_size);
        writer.write_unsigned_short(self.max_bones_per_strip);
        writer.write_unsigned_short(self.max_bones_per_triangle);
        writer.write_integer(self.max_bones_per_vertex);
        writer.write_integer(self.checksum);
        writer.write_array_size(self.material_replacement_lists.len())?;
        self.material_replacement_list_offset = writer.write_integer_index();
        writer.write_array_size(self.body_parts.len())?;
        self.body_part_offset = writer.write_integer_index();

        writer.write_to_integer_offset(self.body_part_offset, writer.data.len())?;

        for body_part in &mut self.body_parts {
            body_part.write(writer)?;
        }

        for body_part in &mut self.body_parts {
            body_part.write_model_index(writer)?;
            for model in &mut body_part.models {
                model.write(writer)?;
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                model.write_model_lod_index(writer)?;
                for model_lod in &mut model.model_lods {
                    model_lod.write(writer)?;
                }
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                for model_lod in &mut model.model_lods {
                    model_lod.write_mesh_index(writer)?;
                    for mesh in &mut model_lod.meshes {
                        mesh.write(writer)?;
                    }
                }
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                for model_lod in &mut model.model_lods {
                    for mesh in &mut model_lod.meshes {
                        mesh.write_strip_group_index(writer)?;
                        for strip_group in &mut mesh.strip_groups {
                            strip_group.write(writer)?;
                        }
                    }
                }
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                for model_lod in &mut model.model_lods {
                    for mesh in &mut model_lod.meshes {
                        for strip_group in &mut mesh.strip_groups {
                            strip_group.write_strip_index(writer)?;
                            for strip in &mut strip_group.strips {
                                strip.write(writer)?;
                            }
                        }
                    }
                }
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                for model_lod in &mut model.model_lods {
                    for mesh in &mut model_lod.meshes {
                        for strip_group in &mut mesh.strip_groups {
                            strip_group.write_vertex_index(writer)?;
                            for vertex in &mut strip_group.vertices {
                                vertex.write(writer)?;
                            }
                        }
                    }
                }
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                for model_lod in &mut model.model_lods {
                    for mesh in &mut model_lod.meshes {
                        for strip_group in &mut mesh.strip_groups {
                            strip_group.write_integer_index_index(writer)?;
                            for index in &strip_group.indices {
                                writer.write_unsigned_short(*index);
                            }
                        }
                    }
                }
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                for model_lod in &mut model.model_lods {
                    for mesh in &mut model_lod.meshes {
                        for strip_group in &mut mesh.strip_groups {
                            for strip in &mut strip_group.strips {
                                strip.write_bone_state_change_index(writer)?;
                                for bone_state_change in &mut strip.bone_state_changes {
                                    bone_state_change.write(writer)?;
                                }
                            }
                        }
                    }
                }
            }
        }

        writer.write_to_integer_offset(self.material_replacement_list_offset, writer.data.len())?;

        for material_replacement_list in &mut self.material_replacement_lists {
            material_replacement_list.write(writer)?;
        }

        for material_replacement_list in &mut self.material_replacement_lists {
            material_replacement_list.write_material_replacement_index(writer)?;
            for replacement in &mut material_replacement_list.material_replacements {
                replacement.write(writer)?;
            }
        }

        writer.write_string_table()?;

        Ok(())
    }
}

pub const MAX_HARDWARE_BONES_PER_STRIP: usize = (86 << 24) + (83 << 16) + (68 << 8) + 73;

#[derive(Debug, Default)]
pub struct MeshFileMaterialReplacementListHeader {
    pub write_base: usize,
    pub material_replacements: Vec<MeshFileMaterialReplacementHeader>,
    pub material_replacement_offset: usize,
}

impl WriteToWriter for MeshFileMaterialReplacementListHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();
        writer.write_integer(self.material_replacements.len() as i32);
        self.material_replacement_offset = writer.write_integer_index();
        Ok(())
    }
}

impl MeshFileMaterialReplacementListHeader {
    fn write_material_replacement_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.material_replacement_offset, writer.data.len() - self.write_base)
    }
}

#[derive(Debug, Default)]
pub struct MeshFileMaterialReplacementHeader {
    pub write_base: usize,
    pub material_id: i16,
    pub replacement_material_name: String,
}

impl WriteToWriter for MeshFileMaterialReplacementHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();
        writer.write_short(self.material_id);
        writer.write_string_to_table(self.write_base, &self.replacement_material_name);
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct MeshFileBodyPartHeader {
    pub write_base: usize,
    pub models: Vec<MeshFileModelHeader>,
    pub model_offset: usize,
}

impl WriteToWriter for MeshFileBodyPartHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();
        writer.write_array_size(self.models.len())?;
        self.model_offset = writer.write_integer_index();
        Ok(())
    }
}

impl MeshFileBodyPartHeader {
    fn write_model_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.model_offset, writer.data.len() - self.write_base)
    }
}

#[derive(Debug, Default)]
pub struct MeshFileModelHeader {
    pub write_base: usize,
    pub model_lods: Vec<MeshFileModelLODHeader>,
    pub model_lod_offset: usize,
}

impl WriteToWriter for MeshFileModelHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();
        writer.write_array_size(self.model_lods.len())?;
        self.model_lod_offset = writer.write_integer_index();
        Ok(())
    }
}

impl MeshFileModelHeader {
    fn write_model_lod_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.model_lod_offset, writer.data.len() - self.write_base)
    }
}

#[derive(Debug, Default)]
pub struct MeshFileModelLODHeader {
    pub write_base: usize,
    pub meshes: Vec<MeshFileMeshHeader>,
    pub switch_point: f32,
    pub mesh_offset: usize,
}

impl WriteToWriter for MeshFileModelLODHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();
        writer.write_array_size(self.meshes.len())?;
        self.mesh_offset = writer.write_integer_index();
        writer.write_float(self.switch_point);
        Ok(())
    }
}

impl MeshFileModelLODHeader {
    fn write_mesh_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.mesh_offset, writer.data.len() - self.write_base)
    }
}

#[derive(Debug, Default)]
pub struct MeshFileMeshHeader {
    pub write_base: usize,
    pub strip_groups: Vec<MeshFileStripGroupHeader>,
    pub strip_group_offset: usize,
    pub flags: MeshFileMeshHeaderFlags,
}

impl WriteToWriter for MeshFileMeshHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();
        writer.write_array_size(self.strip_groups.len())?;
        self.strip_group_offset = writer.write_integer_index();
        writer.write_unsigned_byte(self.flags.bits());
        Ok(())
    }
}

impl MeshFileMeshHeader {
    fn write_strip_group_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.strip_group_offset, writer.data.len() - self.write_base)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct MeshFileMeshHeaderFlags: u8 {
        const IS_TEETH = 0x01;
        const IS_EYES = 0x02;
    }
}

#[derive(Debug, Default)]
pub struct MeshFileStripGroupHeader {
    pub write_base: usize,
    pub vertices: Vec<MeshFileVertexHeader>,
    pub vertex_offset: usize,
    pub indices: Vec<u16>,
    pub index_offset: usize,
    pub strips: Vec<MeshFileStripHeader>,
    pub strip_offset: usize,
    pub flags: MeshFileStripGroupHeaderFlags,
}

impl WriteToWriter for MeshFileStripGroupHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();
        writer.write_array_size(self.vertices.len())?;
        self.vertex_offset = writer.write_integer_index();
        writer.write_array_size(self.indices.len())?;
        self.index_offset = writer.write_integer_index();
        writer.write_array_size(self.strips.len())?;
        self.strip_offset = writer.write_integer_index();
        writer.write_unsigned_byte(self.flags.bits());
        Ok(())
    }
}

impl MeshFileStripGroupHeader {
    fn write_vertex_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.vertex_offset, writer.data.len() - self.write_base)
    }

    fn write_integer_index_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.index_offset, writer.data.len() - self.write_base)
    }

    fn write_strip_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.strip_offset, writer.data.len() - self.write_base)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct MeshFileStripGroupHeaderFlags: u8 {
        const IS_FLEXED           = 0x01;
        const IS_HARDWARE_SKINNED = 0x02;
        const IS_DELTA_FLEXED     = 0x04;
    }
}

#[derive(Debug, Default)]
pub struct MeshFileVertexHeader {
    pub vertex_index: u16,
    pub bone_count: u8,
    pub bone_weight_bones: [u8; 3],
}

impl WriteToWriter for MeshFileVertexHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_unsigned_byte_array(&[0, 1, 2]);
        writer.write_unsigned_byte(self.bone_count);
        writer.write_unsigned_short(self.vertex_index);
        writer.write_unsigned_byte_array(self.bone_weight_bones.as_ref());
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct MeshFileStripHeader {
    pub write_base: usize,
    pub indices_count: i32,
    pub indices_index: i32,
    pub vertices_count: i32,
    pub vertices_index: i32,
    pub bone_count: i16,
    pub flags: MeshFileStripFlags,
    pub bone_state_changes: Vec<MeshFileBoneStateChangeHeader>,
    pub bone_state_change_offset: usize,
}

impl WriteToWriter for MeshFileStripHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.write_base = writer.data.len();
        writer.write_integer(self.indices_count);
        writer.write_integer(self.indices_index);
        writer.write_integer(self.vertices_count);
        writer.write_integer(self.vertices_index);
        writer.write_short(self.bone_count);
        writer.write_unsigned_byte(self.flags.bits());
        writer.write_array_size(self.bone_state_changes.len())?;
        self.bone_state_change_offset = writer.write_integer_index();
        Ok(())
    }
}

impl MeshFileStripHeader {
    fn write_bone_state_change_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.bone_state_change_offset, writer.data.len() - self.write_base)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct MeshFileStripFlags: u8 {
        const IS_TRIANGLE_LIST  = 0x01;
        const IS_TRIANGLE_STRIP = 0x02;
    }
}

#[derive(Debug, Default)]
pub struct MeshFileBoneStateChangeHeader {
    pub hardware_id: i32,
    pub bone_table_index: i32,
}

impl WriteToWriter for MeshFileBoneStateChangeHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_integer(self.hardware_id);
        writer.write_integer(self.bone_table_index);
        Ok(())
    }
}
