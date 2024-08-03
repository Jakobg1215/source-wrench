use std::{collections::HashMap, fs::write, mem::size_of};

use half::f16;
use mesh::{MeshMaterialReplacementListHeader, MeshModelHeader};
use model::{
    ModelAnimationDescription, ModelAnimationSection, ModelBodyPart, ModelBone, ModelBoneFlags, ModelHeader, ModelHitBox, ModelHitboxSet, ModelMaterial,
    ModelMesh, ModelModel, ModelSequenceDescription, SecondModelHeader,
};
use thiserror::Error as ThisError;

use crate::{
    process::ProcessedData,
    utilities::mathematics::{clamp, Angles, BoundingBox, Matrix, Quaternion, Vector2, Vector3, Vector4},
};

use self::{
    mesh::{
        MeshBodyPartHeader, MeshBoneStateChangeHeader, MeshFileHeader, MeshMeshHeader, MeshModelLODHeader, MeshStripGroupHeader, StripHeader, VertexHeader,
    },
    vertex::{VertexFileHeader, VertexFileVertex},
};

mod mesh;
mod model;
mod vertex;

pub const MAX_LOD_COUNT: usize = 8;

#[derive(Debug, ThisError)]
pub enum FileWriteError {
    #[error("Array provided is too large to write to file.")]
    ArraySizeToLarge,
    #[error("Keyvalues provided are too large to write to file.")]
    KeyvaluesToLarge,
    #[error("Offset provided is too large to write to file.")]
    OffsetToLarge,
}

#[derive(Debug, Default)]
pub struct FileWriter {
    pub data: Vec<u8>,
    string_table: HashMap<String, Vec<(usize, usize)>>,
}

impl FileWriter {
    pub fn write_unsigned_byte(&mut self, value: u8) {
        self.data.extend(value.to_le_bytes());
    }

    pub fn write_unsigned_byte_array(&mut self, values: &[u8]) {
        for value in values {
            self.write_unsigned_byte(*value);
        }
    }

    pub fn write_short(&mut self, value: i16) {
        self.data.extend(value.to_le_bytes());
    }

    pub fn write_unsigned_short(&mut self, value: u16) {
        self.data.extend(value.to_le_bytes());
    }

    pub fn write_integer(&mut self, value: i32) {
        self.data.extend(value.to_le_bytes());
    }

    pub fn write_integer_array(&mut self, values: &[i32]) {
        for value in values {
            self.write_integer(*value);
        }
    }

    pub fn write_float(&mut self, value: f32) {
        self.data.extend(value.to_le_bytes());
    }

    pub fn write_float_array(&mut self, values: &[f32]) {
        for value in values {
            self.write_float(*value);
        }
    }

    pub fn write_long(&mut self, value: i64) {
        self.data.extend(value.to_le_bytes());
    }

    pub fn write_char_array(&mut self, value: &str, length: usize) {
        let mut bytes = value.as_bytes().to_vec();
        bytes.resize(length, 0);
        self.data.extend(bytes);
    }

    pub fn write_vector2(&mut self, value: Vector2) {
        self.write_float(value.x as f32);
        self.write_float(value.y as f32);
    }

    pub fn write_vector3(&mut self, value: Vector3) {
        self.write_float(value.x as f32);
        self.write_float(value.y as f32);
        self.write_float(value.z as f32);
    }

    pub fn write_vector4(&mut self, value: Vector4) {
        self.write_float(value.x as f32);
        self.write_float(value.y as f32);
        self.write_float(value.z as f32);
        self.write_float(value.w as f32);
    }

    pub fn write_quaternion(&mut self, value: Quaternion) {
        self.write_float(value.x as f32);
        self.write_float(value.y as f32);
        self.write_float(value.z as f32);
        self.write_float(value.w as f32);
    }

    pub fn write_angles(&mut self, value: Angles) {
        self.write_float(value.x as f32);
        self.write_float(value.y as f32);
        self.write_float(value.z as f32);
    }

    pub fn write_string_to_table(&mut self, base: usize, value: &str) {
        let string_offset = self.write_integer_index();

        match self.string_table.get_mut(value) {
            Some(table) => {
                table.push((base, string_offset));
            }
            None => {
                self.string_table.insert(String::from(value), vec![(base, string_offset)]);
            }
        }
    }

    pub fn write_string_table(&mut self) -> Result<(), FileWriteError> {
        let mut entries = self.string_table.drain().collect::<Vec<(String, Vec<(usize, usize)>)>>();

        entries.sort_by(|(to, _), (from, _)| to.cmp(from));

        for (string, string_entries) in entries {
            let string_index = self.data.len();
            self.write_null_terminated_string(&string);

            for (base, index) in string_entries {
                self.write_to_integer_offset(index, string_index - base)?;
            }
        }

        Ok(())
    }

    pub fn write_null_terminated_string(&mut self, value: &str) {
        self.data.extend_from_slice(value.as_bytes());
        self.data.push(0);
    }

    pub fn write_integer_index(&mut self) -> usize {
        self.write_integer(0);
        self.data.len() - size_of::<i32>()
    }

    pub fn write_short_index(&mut self) -> usize {
        self.write_short(0);
        self.data.len() - size_of::<i16>()
    }

    pub fn write_to_integer_offset(&mut self, index: usize, offset: usize) -> Result<(), FileWriteError> {
        if offset > i32::MAX as usize {
            return Err(FileWriteError::OffsetToLarge);
        }

        let bytes = (offset as i32).to_le_bytes();

        self.data[index..index + bytes.len()].clone_from_slice(&bytes as &[u8]);
        Ok(())
    }

    pub fn write_to_short_offset(&mut self, index: usize, offset: usize) -> Result<(), FileWriteError> {
        if offset > i16::MAX as usize {
            return Err(FileWriteError::OffsetToLarge);
        }

        let bytes = (offset as i16).to_le_bytes();

        self.data[index..index + bytes.len()].clone_from_slice(&bytes as &[u8]);
        Ok(())
    }

    pub fn write_quaternion64(&mut self, value: Quaternion) {
        let x = clamp((value.x * 1048576.0) as i64 + 1048576, 0, 2097151);
        let y = clamp((value.y * 1048576.0) as i64 + 1048576, 0, 2097151);
        let z = clamp((value.z * 1048576.0) as i64 + 1048576, 0, 2097151);
        let w = (value.w < 0.0) as i64;
        self.write_long((x << 43) | (y << 22) | (z << 1) | w);
    }

    pub fn write_vector48(&mut self, value: Vector3) {
        self.data.extend(f16::from_f64(value.x).to_le_bytes());
        self.data.extend(f16::from_f64(value.y).to_le_bytes());
        self.data.extend(f16::from_f64(value.z).to_le_bytes());
    }

    pub fn write_array_size(&mut self, size: usize) -> Result<(), FileWriteError> {
        if size > i32::MAX as usize {
            return Err(FileWriteError::ArraySizeToLarge);
        }

        self.write_integer(size as i32);
        Ok(())
    }

    pub fn align(&mut self, alignment: usize) {
        let remainder = self.data.len() % alignment;

        if remainder == 0 {
            return;
        }

        let padding = alignment - remainder;

        self.data.resize(self.data.len() + padding, 0);
    }
}

pub trait WriteToWriter {
    fn write(&mut self, writer: &mut FileWriter) -> Result<(), FileWriteError>;
}

pub fn write_files(name: String, processed_data: ProcessedData, export_path: String) -> Result<(), FileWriteError> {
    let mut mdl_writer = FileWriter::default();
    let mut mdl_header = ModelHeader {
        version: 48,
        checksum: 69420,
        second_header: SecondModelHeader { name, ..Default::default() },
        ..Default::default()
    };

    for processed_bone in processed_data.bone_data.processed_bones {
        // TODO: This should be moved to the mathematics library.
        fn invert(rotation: Matrix, position: Vector3) -> (Matrix, Vector3) {
            let transposed = rotation.transpose();

            (
                transposed,
                Vector3::new(
                    -position.dot(Vector3::new(transposed[0][0], transposed[0][1], transposed[0][2])),
                    -position.dot(Vector3::new(transposed[1][0], transposed[1][1], transposed[1][2])),
                    -position.dot(Vector3::new(transposed[2][0], transposed[2][1], transposed[2][2])),
                ),
            )
        }

        let bone = ModelBone {
            name: processed_bone.name,
            parent: match processed_bone.parent {
                Some(index) => index as i32,
                None => -1,
            },
            position: processed_bone.position,
            rotation: processed_bone.rotation,
            quaternion: processed_bone.rotation.to_quaternion(),
            animation_position_scale: processed_bone.animation_position_scale,
            animation_rotation_scale: processed_bone.animation_rotation_scale,
            pose: invert(processed_bone.pose.0, processed_bone.pose.1),
            flags: ModelBoneFlags::USED_BY_VERTEX_AT_LOD0,
            ..Default::default()
        };
        mdl_header.bones.push(bone);
    }

    mdl_header.sorted_bone_table_by_name = processed_data.bone_data.sorted_bones_by_name.iter().map(|bone| *bone as u8).collect();

    let hitbox_set = ModelHitboxSet {
        name: "default".to_string(),
        hitboxes: vec![ModelHitBox {
            bounding_box: BoundingBox {
                minimum: Vector3 { x: -10.0, y: -10.0, z: 0.0 },
                maximum: Vector3 { x: 10.0, y: 10.0, z: 20.0 },
            },
            ..Default::default()
        }],
        ..Default::default()
    };

    mdl_header.hitbox_sets.push(hitbox_set);

    for processed_animation in processed_data.animation_data {
        let animation_description = ModelAnimationDescription {
            name: processed_animation.name,
            frame_count: processed_animation.frame_count as i32,
            animation_sections: vec![ModelAnimationSection::default()],
            ..Default::default()
        };

        mdl_header.local_animation_descriptions.push(animation_description);
    }

    for processed_sequence in processed_data.sequence_data {
        let sequence_description = ModelSequenceDescription {
            name: processed_sequence.name,
            animations: processed_sequence.animations.iter().map(|index| *index as i16).collect(),
            blend_size: [processed_sequence.animations.len() as i32; 2], // TODO: Change this to support multiple blend sizes.
            weight_list: vec![1.0; mdl_header.bones.len()],
            ..Default::default()
        };

        mdl_header.local_sequence_descriptions.push(sequence_description);
    }

    let mut vvd_writer = FileWriter::default();
    let mut vvd_header = VertexFileHeader {
        version: 4,
        checksum: 69420,
        lod_count: 1,
        ..Default::default()
    };

    let mut vtx_writer = FileWriter::default();
    let mut vtx_header = MeshFileHeader::default();
    vtx_header.check_sum = 69420;

    mdl_header.material_paths.push(String::from("\\"));

    let mut mesh_id = 0;
    let mut previous_base: Option<(i32, usize)> = None;
    for processed_body_part in processed_data.model_data.body_parts {
        let mut body_part = ModelBodyPart {
            name: processed_body_part.name,
            base: match previous_base {
                Some(base) => base.0 * base.1 as i32,
                None => 1,
            },
            ..Default::default()
        };

        previous_base = Some((body_part.base, processed_body_part.parts.len()));

        let mut mesh_body_part_header = MeshBodyPartHeader::default();

        for processed_part in processed_body_part.parts {
            let mut model = ModelModel {
                name: processed_part.name,
                vertex_count: processed_part.meshes.iter().map(|mesh| mesh.vertex_data.len()).sum::<usize>() as i32,
                vertex_offset: (vvd_header.vertices.len() * 48) as i32,
                tangent_offset: (vvd_header.tangents.len() * 16) as i32,
                ..Default::default()
            };

            let mut mesh_model_header = MeshModelHeader::default();
            let mut mesh_model_lod_header = MeshModelLODHeader::default();

            let mut vertex_count = 0;
            for processed_mesh in processed_part.meshes {
                let body_mesh = ModelMesh {
                    material: processed_mesh.material as i32,
                    vertex_count: processed_mesh.vertex_data.len() as i32,
                    vertex_offset: vertex_count as i32,
                    mesh_identifier: mesh_id,
                    vertex_lod_count: [processed_mesh.vertex_data.len() as i32; 8],
                    ..Default::default()
                };

                mesh_id += 1;
                vertex_count += processed_mesh.vertex_data.len();
                for vertex in processed_mesh.vertex_data {
                    let mut uv_fix = vertex.uv; // FIXME: This should be in the mesh processing stage.
                    uv_fix.y = 1.0 - uv_fix.y;
                    let vvd_vertex = VertexFileVertex {
                        weights: [vertex.weights[0] as f32, vertex.weights[1] as f32, vertex.weights[2] as f32],
                        bones: [vertex.bones[0] as u8, vertex.bones[1] as u8, vertex.bones[2] as u8],
                        bone_count: vertex.bone_count as u8,
                        position: vertex.position,
                        normal: vertex.normal,
                        texture_coordinate: vertex.uv,
                    };

                    vvd_header.vertices.push(vvd_vertex);
                    vvd_header.tangents.push(vertex.tangent);
                }

                let mut mesh_mesh_header = MeshMeshHeader::default();

                for strip_group in processed_mesh.strip_groups {
                    let mut mesh_strip_group_header = MeshStripGroupHeader::default();
                    mesh_strip_group_header.flags = 2;

                    for vertex in strip_group.vertices {
                        let mesh_vertex = VertexHeader {
                            bone_count: vertex.bone_count as u8,
                            vertex_index: vertex.vertex_index as u16,
                            bone_weight_bones: [vertex.bones[0] as u8, vertex.bones[1] as u8, vertex.bones[2] as u8],
                        };

                        mesh_strip_group_header.vertices.push(mesh_vertex);
                    }

                    mesh_strip_group_header.indices = strip_group.indices;

                    for strip in strip_group.strips {
                        let mut mesh_strip_header = StripHeader::default();
                        mesh_strip_header.flags = 1;
                        mesh_strip_header.indices_count = mesh_strip_group_header.indices.len() as i32;
                        mesh_strip_header.vertices_count = mesh_strip_group_header.vertices.len() as i32;
                        mesh_strip_header.bone_count = strip.bone_count as i16;

                        for bone_change in strip.hardware_bones {
                            let mesh_bone_state_change = MeshBoneStateChangeHeader {
                                hardware_id: bone_change.hardware_bone as i32,
                                new_bone_id: bone_change.bone_table_bone as i32,
                            };

                            mesh_strip_header.bone_state_changes.push(mesh_bone_state_change);
                        }

                        mesh_strip_group_header.strips.push(mesh_strip_header);
                    }

                    mesh_mesh_header.strip_groups.push(mesh_strip_group_header);
                }

                mesh_model_lod_header.meshes.push(mesh_mesh_header);
                model.meshes.push(body_mesh);
            }

            body_part.models.push(model);
            mesh_model_header.model_lods.push(mesh_model_lod_header);
            mesh_body_part_header.models.push(mesh_model_header);
        }

        mdl_header.body_parts.push(body_part);
        vtx_header.body_parts.push(mesh_body_part_header);
    }
    vtx_header.material_replacement_lists.push(MeshMaterialReplacementListHeader::default());
    vvd_header.lod_vertex_count = [vvd_header.vertices.len() as i32; MAX_LOD_COUNT];

    for processed_material in processed_data.model_data.materials {
        let material = ModelMaterial {
            name: processed_material,
            ..Default::default()
        };
        mdl_header.materials.push(material);
    }

    mdl_header.material_replacements.push((0..mdl_header.materials.len() as i16).collect());

    mdl_header.write(&mut mdl_writer)?;
    vvd_header.write(&mut vvd_writer)?;
    vtx_header.write(&mut vtx_writer)?;

    // FIXME: This is a temporary solution to write the files.
    let _ = write(format!("{}/{}.{}", export_path, mdl_header.second_header.name, "mdl"), mdl_writer.data);
    let _ = write(format!("{}/{}.{}", export_path, mdl_header.second_header.name, "vvd"), vvd_writer.data);
    let _ = write(format!("{}/{}.{}", export_path, mdl_header.second_header.name, "dx90.vtx"), vtx_writer.data);

    Ok(())
}
