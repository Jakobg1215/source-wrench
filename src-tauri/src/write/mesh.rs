use super::{FileWriteError, FileWriter, WriteToWriter};

#[derive(Debug, Default)]
pub struct MeshFileHeader {
    pub check_sum: i32,
    pub material_replacement_lists: Vec<MeshMaterialReplacementListHeader>,
    pub body_parts: Vec<MeshBodyPartHeader>,
    material_replacement_list_index: usize,
    body_part_index: usize,
}

impl WriteToWriter for MeshFileHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_integer(7); // version
        writer.write_integer(24); // vertCacheSize
        writer.write_unsigned_short(53); // maxBonesPerStrip
        writer.write_unsigned_short(9); // maxBonesPerTri
        writer.write_integer(3); // maxBonesPerVert
        writer.write_integer(self.check_sum); // checkSum
        writer.write_integer(self.material_replacement_lists.len() as i32); // numLODs
        self.material_replacement_list_index = writer.write_integer_index(); // materialReplacementListOffset
        writer.write_integer(self.body_parts.len() as i32); // numBodyParts
        self.body_part_index = writer.write_integer_index(); // bodyPartOffset

        writer.write_to_integer_offset(self.body_part_index, writer.data.len())?;

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

        writer.write_to_integer_offset(self.material_replacement_list_index, writer.data.len())?;

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

#[derive(Debug, Default)]
pub struct MeshMaterialReplacementListHeader {
    index_start: usize,
    pub material_replacements: Vec<MeshMaterialReplacementHeader>,
    material_replacement_index: usize,
}

impl WriteToWriter for MeshMaterialReplacementListHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.index_start = writer.data.len();
        writer.write_integer(self.material_replacements.len() as i32); // numReplacements
        self.material_replacement_index = writer.write_integer_index(); // replacementOffset
        Ok(())
    }
}

impl MeshMaterialReplacementListHeader {
    fn write_material_replacement_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.material_replacement_index, writer.data.len() - self.index_start)
    }
}

#[derive(Debug, Default)]
pub struct MeshMaterialReplacementHeader {
    index_start: usize,
    pub material_id: i16,
    pub replacement_material_name: String,
}

impl WriteToWriter for MeshMaterialReplacementHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.index_start = writer.data.len();
        writer.write_short(self.material_id); // materialID
        writer.write_string_to_table(self.index_start, &self.replacement_material_name);
        // replacementMaterialNameOffset
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct MeshBodyPartHeader {
    index_start: usize,
    pub models: Vec<MeshModelHeader>,
    model_index: usize,
}

impl WriteToWriter for MeshBodyPartHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.index_start = writer.data.len();
        writer.write_integer(self.models.len() as i32); // numModels
        self.model_index = writer.write_integer_index(); // modelOffset
        Ok(())
    }
}

impl MeshBodyPartHeader {
    fn write_model_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.model_index, writer.data.len() - self.index_start)
    }
}

#[derive(Debug, Default)]
pub struct MeshModelHeader {
    index_start: usize,
    pub model_lods: Vec<MeshModelLODHeader>,
    model_lod_index: usize,
}

impl WriteToWriter for MeshModelHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.index_start = writer.data.len();
        writer.write_integer(self.model_lods.len() as i32); // numLODs
        self.model_lod_index = writer.write_integer_index(); // lodOffset
        Ok(())
    }
}

impl MeshModelHeader {
    fn write_model_lod_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.model_lod_index, writer.data.len() - self.index_start)
    }
}

#[derive(Debug, Default)]
pub struct MeshModelLODHeader {
    index_start: usize,
    pub meshes: Vec<MeshMeshHeader>,
    pub switch_point: f32,
    mesh_index: usize,
}

impl WriteToWriter for MeshModelLODHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.index_start = writer.data.len();
        writer.write_integer(self.meshes.len() as i32); // numMeshes
        self.mesh_index = writer.write_integer_index(); // meshOffset
        writer.write_float(self.switch_point); // switchPoint
        Ok(())
    }
}

impl MeshModelLODHeader {
    fn write_mesh_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.mesh_index, writer.data.len() - self.index_start)
    }
}

#[derive(Debug, Default)]
pub struct MeshMeshHeader {
    index_start: usize,
    pub strip_groups: Vec<MeshStripGroupHeader>,
    strip_group_index: usize,
    pub flags: u8,
}

impl WriteToWriter for MeshMeshHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.index_start = writer.data.len();
        writer.write_integer(self.strip_groups.len() as i32); // numStripGroups
        self.strip_group_index = writer.write_integer_index(); // stripGroupHeaderOffset
        writer.write_unsigned_byte(self.flags); // flags
        Ok(())
    }
}

impl MeshMeshHeader {
    fn write_strip_group_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.strip_group_index, writer.data.len() - self.index_start)
    }
}

#[derive(Debug, Default)]
pub struct MeshStripGroupHeader {
    index_start: usize,
    pub vertices: Vec<VertexHeader>,
    vertex_index: usize,
    pub indices: Vec<u16>,
    index_index: usize,
    pub strips: Vec<StripHeader>,
    strip_index: usize,
    pub flags: u8,
}

impl WriteToWriter for MeshStripGroupHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.index_start = writer.data.len();
        writer.write_integer(self.vertices.len() as i32); // numVerts
        self.vertex_index = writer.write_integer_index(); // vertOffset
        writer.write_integer(self.indices.len() as i32); // numIndices
        self.index_index = writer.write_integer_index(); // indexOffset
        writer.write_integer(self.strips.len() as i32); // numStrips
        self.strip_index = writer.write_integer_index(); // stripOffset
        writer.write_unsigned_byte(self.flags); // flags
        Ok(())
    }
}

impl MeshStripGroupHeader {
    fn write_vertex_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.vertex_index, writer.data.len() - self.index_start)
    }

    fn write_integer_index_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.index_index, writer.data.len() - self.index_start)
    }

    fn write_strip_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.strip_index, writer.data.len() - self.index_start)
    }
}

#[derive(Debug, Default)]
pub struct VertexHeader {
    pub vertex_index: u16,
    pub bone_count: u8,
    pub bone_weight_bones: [u8; 3],
}

impl WriteToWriter for VertexHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_unsigned_byte_array(&[0, 1, 2]); // boneWeightIndex
        writer.write_unsigned_byte(self.bone_count); // numBones
        writer.write_unsigned_short(self.vertex_index); // origMeshVertID
        writer.write_unsigned_byte_array(self.bone_weight_bones.as_ref()); // boneID
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct StripHeader {
    index_start: usize,
    pub indices_count: i32,
    pub indices_offset: i32,
    pub vertices_count: i32,
    pub vertices_offset: i32,
    pub bone_count: i16,
    pub flags: u8,
    pub bone_state_changes: Vec<MeshBoneStateChangeHeader>,
    bone_state_change_index: usize,
}

impl WriteToWriter for StripHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        self.index_start = writer.data.len();
        writer.write_integer(self.indices_count); // numIndices
        writer.write_integer(self.indices_offset); // indexOffset

        writer.write_integer(self.vertices_count); // numVerts
        writer.write_integer(self.vertices_offset); // vertOffset

        writer.write_short(self.bone_count); // numBones
        writer.write_unsigned_byte(self.flags); // flags
        writer.write_integer(self.bone_state_changes.len() as i32); // numBoneStateChanges
        self.bone_state_change_index = writer.write_integer_index(); // boneStateChangeOffset
        Ok(())
    }
}

impl StripHeader {
    fn write_bone_state_change_index(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_to_integer_offset(self.bone_state_change_index, writer.data.len() - self.index_start)
    }
}

#[derive(Debug, Default)]
pub struct MeshBoneStateChangeHeader {
    pub hardware_id: i32,
    pub new_bone_id: i32,
}

impl WriteToWriter for MeshBoneStateChangeHeader {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError> {
        writer.write_integer(self.hardware_id); // hardwareID
        writer.write_integer(self.new_bone_id); // newBoneID
        Ok(())
    }
}
