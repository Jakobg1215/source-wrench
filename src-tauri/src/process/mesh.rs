use std::collections::HashMap;

use crate::{import::ImportedFileData, input::CompilationDataInput, process::structures::ProcessedBodyGroupData};

use super::{bones::BoneTable, structures::ProcessedModelData, ProcessingDataError};

pub fn process_mesh_data(
    input: &CompilationDataInput,
    import: &HashMap<String, ImportedFileData>,
    bone_table: &BoneTable,
) -> Result<ProcessedModelData, ProcessingDataError> {
    let mut model_data = ProcessedModelData::new();

    for body_group in &input.body_groups {
        let mut processed_body_group = ProcessedBodyGroupData::new(body_group.name.clone());

        for body_part in &body_group.parts {
            // UNWRAP: This should be valid from the import stage.
            let source_file = import.get(&body_part.model_source).unwrap();

            let mut mapped_materials = HashMap::new();

            for (index, material) in source_file.mesh.materials.iter().enumerate() {
                mapped_materials.insert(index, model_data.add_material(material.clone()));
            }

            let mut tri_lists: Vec<TriangleList> = Vec::new();

            for face in &source_file.mesh.faces {
                let face_material = mapped_materials.get(&face.material_index).unwrap();

                let tri_list = match tri_lists.iter_mut().find(|list| list.material == *face_material) {
                    Some(list) => list,
                    None => {
                        tri_lists.push(TriangleList::new(*face_material));
                        let length = tri_lists.len();
                        // UNWRAP: We added it above.
                        tri_lists.get_mut(length).unwrap()
                    }
                };

                if face.vertex_indices.len() > 3 {
                    todo!("Triangulate face")
                } else {
                    tri_list
                        .triangles
                        .push(Triangle::new(face.vertex_indices[0], face.vertex_indices[1], face.vertex_indices[2]))
                }
            }
        }
    }

    todo!()
}

struct TriangleList {
    material: usize,
    triangles: Vec<Triangle>,
}

impl TriangleList {
    fn new(material: usize) -> Self {
        Self {
            material,
            triangles: Vec::new(),
        }
    }
}

struct Triangle {
    point1: usize,
    point2: usize,
    point3: usize,
}

impl Triangle {
    fn new(point1: usize, point2: usize, point3: usize) -> Self {
        Self { point1, point2, point3 }
    }
}
