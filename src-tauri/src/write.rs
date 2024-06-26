use std::fs::write;

use mesh::MaterialReplacementListHeader;

use crate::{
    process::ProcessedData,
    utilities::{binarydata::DataWriter, mathematics::Vector3},
};

use self::{
    mesh::{BodyPartHeader, BoneStateChangeHeader, MeshFileHeader, MeshHeader, ModelHeader, ModelLODHeader, StripGroupHeader, StripHeader, VertexHeader},
    model::{Animation, AnimationData, AnimationDescription, BodyPart, Bone, Header, Hitbox, HitboxSet, Material, Mesh, Model, SequenceDescription},
    vertex::{Vertex, VertexFileHeader},
};

mod mesh;
mod model;
mod vertex;

pub trait StructWriting {
    fn write_to_writer(&mut self, writer: &mut DataWriter);
}

pub fn write_files(name: String, processed_data: ProcessedData, export_path: String) {
    let mut mdl_writer = DataWriter::default();
    let mut mdl_header = Header::new();
    mdl_header.second_header.model_name = name;

    for processed_bone in processed_data.bone_data.processed_bones {
        let mut bone = Bone::new();
        bone.name = processed_bone.name;
        bone.parent_bone_index = match processed_bone.parent {
            Some(index) => index as i32,
            None => -1,
        };
        bone.position = processed_bone.position;
        bone.rotation = processed_bone.rotation;
        bone.animation_position_scale = processed_bone.animation_position_scale;
        bone.animation_rotation_scale = processed_bone.animation_rotation_scale;
        mdl_header.bones.push(bone);
    }

    mdl_header.bones_index = processed_data.bone_data.sorted_bones_by_name;

    let mut hitbox_set = HitboxSet::new();
    hitbox_set.name = "default".to_string();

    let mut hitbox = Hitbox::new();
    hitbox.minumum = Vector3::new(10.0, 10.0, 0.0);
    hitbox.maximum = Vector3::new(-10.0, -10.0, 10.0);
    mdl_header.hitbox_sets.push(hitbox_set);

    for processed_animation in processed_data.animation_data {
        let mut animation_description = AnimationDescription::new();
        animation_description.name = processed_animation.name;
        animation_description.frame_count = processed_animation.frame_count;

        let mut animation = Animation::new();
        for processed_bone in processed_animation.bones {
            let mut animation_data = AnimationData::new();
            animation_data.bone_index = processed_bone.bone;
            animation_data.animation_position = processed_bone.position;
            animation_data.animation_rotation = processed_bone.rotation;

            animation.animation_data.push(animation_data);
        }

        mdl_header.animations.push(animation_description);
    }

    for processed_sequence in processed_data.sequence_data {
        let mut sequence_description = SequenceDescription::new();
        sequence_description.name = processed_sequence.name;
        sequence_description.animation_indexes = processed_sequence.animations;
        sequence_description.weight_list = vec![1.0; mdl_header.bones.len()];
        mdl_header.sequences.push(sequence_description);
    }

    let mut vvd_writer = DataWriter::default();
    let mut vvd_header = VertexFileHeader::new(69420);

    let mut vtx_writer = DataWriter::default();
    let mut vtx_header = MeshFileHeader::default();
    vtx_header.check_sum = 69420;

    mdl_header.material_paths.push(String::from("\\"));

    let mut mesh_id = 0;
    let mut previous_base: Option<(i32, usize)> = None;
    for processed_body_part in processed_data.model_data.body_parts {
        let mut body_part = BodyPart::new();
        body_part.name = processed_body_part.name;

        match previous_base {
            Some(base) => {
                body_part.base = base.0 * base.1 as i32;
            }
            None => {
                body_part.base = 1;
            }
        }

        previous_base = Some((body_part.base, processed_body_part.parts.len()));

        let mut mesh_body_part_header = BodyPartHeader::default();

        for processed_part in processed_body_part.parts {
            let mut model = Model::new();
            model.name = processed_part.name;
            model.vertex_count = processed_part.meshes.iter().map(|mesh| mesh.vertex_data.len()).sum::<usize>() as i32;
            model.vertex_index = (vvd_header.vertexes.len() * 48) as i32;
            model.tangent_index = (vvd_header.tangents.len() * 16) as i32;

            let mut mesh_model_header = ModelHeader::default();
            let mut mesh_model_lod_header = ModelLODHeader::default();

            let mut vertex_count = 0;
            for processed_mesh in processed_part.meshes {
                let mut body_mesh = Mesh::new();

                body_mesh.material_index = processed_mesh.material;
                body_mesh.vertex_count = processed_mesh.vertex_data.len();
                body_mesh.vertex_index = vertex_count;
                body_mesh.mesh_id = mesh_id;
                mesh_id += 1;
                vertex_count += processed_mesh.vertex_data.len();
                for vertex in processed_mesh.vertex_data {
                    let mut uv_fix = vertex.uv; // FIXME: This should be in the mesh processing stage.
                    uv_fix.y = 1.0 - uv_fix.y;
                    let vvd_vertex = Vertex::new(vertex.weights, vertex.bones, vertex.bone_count, vertex.position, vertex.normal, uv_fix);
                    vvd_header.vertexes.push(vvd_vertex);
                    vvd_header.tangents.push(vertex.tangent);
                }

                let mut mesh_mesh_header = MeshHeader::default();

                for strip_group in processed_mesh.strip_groups {
                    let mut mesh_strip_group_header = StripGroupHeader::default();
                    mesh_strip_group_header.flags = 2;

                    for vertex in strip_group.vertices {
                        let mut mesh_vertex = VertexHeader::default();
                        mesh_vertex.bone_count = vertex.bone_count as u8;
                        mesh_vertex.vertex_index = vertex.vertex_index as u16;
                        mesh_vertex.bone_weight_bones = [vertex.bones[0] as u8, vertex.bones[1] as u8, vertex.bones[2] as u8];
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
                            let mut mesh_bone_state_change = BoneStateChangeHeader::default();
                            mesh_bone_state_change.hardware_id = bone_change.hardware_bone as i32;
                            mesh_bone_state_change.new_bone_id = bone_change.bone_table_bone as i32;

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
    vtx_header.material_replacement_lists.push(MaterialReplacementListHeader::default());

    for processed_material in processed_data.model_data.materials {
        let mut material = Material::new();
        material.name = processed_material;
        mdl_header.materials.push(material);
    }

    mdl_header.write_to_writer(&mut mdl_writer);
    vvd_header.write_to_writer(&mut vvd_writer);
    vtx_header.write_to_writer(&mut vtx_writer);

    // FIXME: This is a temporary solution to write the files.
    let _ = write(
        format!("{}/{}.{}", export_path, mdl_header.second_header.model_name, "mdl"),
        mdl_writer.get_data(),
    );
    let _ = write(
        format!("{}/{}.{}", export_path, mdl_header.second_header.model_name, "vvd"),
        vvd_writer.get_data(),
    );
    let _ = write(
        format!("{}/{}.{}", export_path, mdl_header.second_header.model_name, "dx90.vtx"),
        vtx_writer.get_data(),
    );
}
