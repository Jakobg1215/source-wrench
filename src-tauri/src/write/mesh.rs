use crate::utilities::binarydata::DataWriter;

use super::StructWriting;

#[derive(Debug, Default)]
pub struct MeshFileHeader {
    pub check_sum: i32,
    pub material_replacement_lists: Vec<MaterialReplacementListHeader>,
    pub body_parts: Vec<BodyPartHeader>,
    material_replacement_list_index: usize,
    body_part_index: usize,
}

impl StructWriting for MeshFileHeader {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        writer.write_int(7); // version
        writer.write_int(24); // vertCacheSize
        writer.write_unsigned_short(53); // maxBonesPerStrip
        writer.write_unsigned_short(9); // maxBonesPerTri
        writer.write_int(3); // maxBonesPerVert
        writer.write_int(self.check_sum); // checkSum
        writer.write_int(self.material_replacement_lists.len() as i32); // numLODs
        self.material_replacement_list_index = writer.write_index(); // materialReplacementListOffset
        writer.write_int(self.body_parts.len() as i32); // numBodyParts
        self.body_part_index = writer.write_index(); // bodyPartOffset

        writer.write_to_index(self.body_part_index, writer.get_size() as i32);

        for body_part in &mut self.body_parts {
            body_part.write_to_writer(writer);
        }

        for body_part in &mut self.body_parts {
            body_part.write_model_index(writer);
            for model in &mut body_part.models {
                model.write_to_writer(writer);
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                model.write_model_lod_index(writer);
                for model_lod in &mut model.model_lods {
                    model_lod.write_to_writer(writer);
                }
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                for model_lod in &mut model.model_lods {
                    model_lod.write_mesh_index(writer);
                    for mesh in &mut model_lod.meshes {
                        mesh.write_to_writer(writer);
                    }
                }
            }
        }

        for body_part in &mut self.body_parts {
            for model in &mut body_part.models {
                for model_lod in &mut model.model_lods {
                    for mesh in &mut model_lod.meshes {
                        mesh.write_strip_group_index(writer);
                        for strip_group in &mut mesh.strip_groups {
                            strip_group.write_to_writer(writer);
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
                            strip_group.write_strip_index(writer);
                            for strip in &mut strip_group.strips {
                                strip.write_to_writer(writer);
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
                            strip_group.write_vertex_index(writer);
                            for vertex in &mut strip_group.vertices {
                                vertex.write_to_writer(writer);
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
                            strip_group.write_index_index(writer);
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
                                strip.write_bone_state_change_index(writer);
                                for bone_state_change in &mut strip.bone_state_changes {
                                    bone_state_change.write_to_writer(writer)
                                }
                            }
                        }
                    }
                }
            }
        }

        writer.write_to_index(self.material_replacement_list_index, writer.get_size() as i32);

        for material_replacement_list in &mut self.material_replacement_lists {
            material_replacement_list.write_to_writer(writer);
        }

        for material_replacement_list in &mut self.material_replacement_lists {
            material_replacement_list.write_material_replacement_index(writer);
            for replacement in &mut material_replacement_list.material_replacements {
                replacement.write_to_writer(writer);
            }
        }

        writer.write_string_table();
    }
}

#[derive(Debug, Default)]
pub struct MaterialReplacementListHeader {
    index_start: usize,
    pub material_replacements: Vec<MaterialReplacementHeader>,
    material_replacement_index: usize,
}

impl StructWriting for MaterialReplacementListHeader {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.index_start = writer.get_size();
        writer.write_int(self.material_replacements.len() as i32); // numReplacements
        self.material_replacement_index = writer.write_index(); // replacementOffset
    }
}

impl MaterialReplacementListHeader {
    fn write_material_replacement_index(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.material_replacement_index, (writer.get_size() - self.index_start) as i32);
    }
}

#[derive(Debug, Default)]
pub struct MaterialReplacementHeader {
    index_start: usize,
    pub material_id: i16,
    pub replacement_material_name: String,
}

impl StructWriting for MaterialReplacementHeader {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.index_start = writer.get_size();
        writer.write_short(self.material_id); // materialID
        writer.add_string_to_table(self.index_start, &self.replacement_material_name);
        // replacementMaterialNameOffset
    }
}

#[derive(Debug, Default)]
pub struct BodyPartHeader {
    index_start: usize,
    pub models: Vec<ModelHeader>,
    model_index: usize,
}

impl StructWriting for BodyPartHeader {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.index_start = writer.get_size();
        writer.write_int(self.models.len() as i32); // numModels
        self.model_index = writer.write_index(); // modelOffset
    }
}

impl BodyPartHeader {
    fn write_model_index(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.model_index, (writer.get_size() - self.index_start) as i32);
    }
}

#[derive(Debug, Default)]
pub struct ModelHeader {
    index_start: usize,
    pub model_lods: Vec<ModelLODHeader>,
    model_lod_index: usize,
}

impl StructWriting for ModelHeader {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.index_start = writer.get_size();
        writer.write_int(self.model_lods.len() as i32); // numLODs
        self.model_lod_index = writer.write_index(); // lodOffset
    }
}

impl ModelHeader {
    fn write_model_lod_index(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.model_lod_index, (writer.get_size() - self.index_start) as i32);
    }
}

#[derive(Debug, Default)]
pub struct ModelLODHeader {
    index_start: usize,
    pub meshes: Vec<MeshHeader>,
    pub switch_point: f32,
    mesh_index: usize,
}

impl StructWriting for ModelLODHeader {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.index_start = writer.get_size();
        writer.write_int(self.meshes.len() as i32); // numMeshes
        self.mesh_index = writer.write_index(); // meshOffset
        writer.write_float(self.switch_point); // switchPoint
    }
}

impl ModelLODHeader {
    fn write_mesh_index(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.mesh_index, (writer.get_size() - self.index_start) as i32);
    }
}

#[derive(Debug, Default)]
pub struct MeshHeader {
    index_start: usize,
    pub strip_groups: Vec<StripGroupHeader>,
    strip_group_index: usize,
    pub flags: u8,
}

impl StructWriting for MeshHeader {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.index_start = writer.get_size();
        writer.write_int(self.strip_groups.len() as i32); // numStripGroups
        self.strip_group_index = writer.write_index(); // stripGroupHeaderOffset
        writer.write_unsigned_byte(self.flags); // flags
    }
}

impl MeshHeader {
    fn write_strip_group_index(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.strip_group_index, (writer.get_size() - self.index_start) as i32);
    }
}

#[derive(Debug, Default)]
pub struct StripGroupHeader {
    index_start: usize,
    pub vertices: Vec<VertexHeader>,
    vertex_index: usize,
    pub indices: Vec<u16>,
    index_index: usize,
    pub strips: Vec<StripHeader>,
    strip_index: usize,
    pub flags: u8,
}

impl StructWriting for StripGroupHeader {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.index_start = writer.get_size();
        writer.write_int(self.vertices.len() as i32); // numVerts
        self.vertex_index = writer.write_index(); // vertOffset
        writer.write_int(self.indices.len() as i32); // numIndices
        self.index_index = writer.write_index(); // indexOffset
        writer.write_int(self.strips.len() as i32); // numStrips
        self.strip_index = writer.write_index(); // stripOffset
        writer.write_unsigned_byte(self.flags); // flags
    }
}

impl StripGroupHeader {
    fn write_vertex_index(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.vertex_index, (writer.get_size() - self.index_start) as i32);
    }

    fn write_index_index(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.index_index, (writer.get_size() - self.index_start) as i32);
    }

    fn write_strip_index(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.strip_index, (writer.get_size() - self.index_start) as i32);
    }
}

#[derive(Debug, Default)]
pub struct VertexHeader {
    pub vertex_index: u16,
    pub bone_count: u8,
    pub bone_weight_bones: [u8; 3],
}

impl StructWriting for VertexHeader {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        writer.write_unsigned_byte_array(&vec![0, 1, 2]); // boneWeightIndex
        writer.write_unsigned_byte(self.bone_count); // numBones
        writer.write_unsigned_short(self.vertex_index); // origMeshVertID
        writer.write_unsigned_byte_array(&self.bone_weight_bones.to_vec()); // boneID
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
    pub bone_state_changes: Vec<BoneStateChangeHeader>,
    bone_state_change_index: usize,
}

impl StructWriting for StripHeader {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        self.index_start = writer.get_size();
        writer.write_int(self.indices_count); // numIndices
        writer.write_int(self.indices_offset); // indexOffset

        writer.write_int(self.vertices_count); // numVerts
        writer.write_int(self.vertices_offset); // vertOffset

        writer.write_short(self.bone_count); // numBones
        writer.write_unsigned_byte(self.flags); // flags
        writer.write_int(self.bone_state_changes.len() as i32); // numBoneStateChanges
        self.bone_state_change_index = writer.write_index(); // boneStateChangeOffset
    }
}

impl StripHeader {
    fn write_bone_state_change_index(&mut self, writer: &mut DataWriter) {
        writer.write_to_index(self.bone_state_change_index, (writer.get_size() - self.index_start) as i32);
    }
}

#[derive(Debug, Default)]
pub struct BoneStateChangeHeader {
    pub hardware_id: i32,
    pub new_bone_id: i32,
}

impl StructWriting for BoneStateChangeHeader {
    fn write_to_writer(&mut self, writer: &mut DataWriter) {
        writer.write_int(self.hardware_id); // hardwareID
        writer.write_int(self.new_bone_id); // newBoneID
    }
}
