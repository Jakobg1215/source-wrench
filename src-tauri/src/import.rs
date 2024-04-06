use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display, Formatter},
    path::Path,
};

use crate::{
    input::CompilationDataInput,
    utilities::{
        logging::{log, LogLevel},
        mathematics::{Quaternion, Vector2, Vector3},
    },
};

mod smd;

#[derive(Debug)]
pub enum ImportingError {
    FileNotFound,
    NoFileExtension,
    UnsupportedFile,
    FailedImport,
}

impl Display for ImportingError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        let error_message: &str = match self {
            ImportingError::FileNotFound => "File Could Not Be Found!",
            ImportingError::NoFileExtension => "File Has No Extension!",
            ImportingError::UnsupportedFile => "File Is Unsupported!",
            ImportingError::FailedImport => "File Failed To Import!",
        };

        fmt.write_str(error_message)
    }
}

impl Error for ImportingError {}

pub fn load_all_source_files(input_data: &CompilationDataInput) -> Result<HashMap<String, ImportedFileData>, ImportingError> {
    log("Loading all source files.", LogLevel::Info);

    let mut loaded_files: HashMap<String, ImportedFileData> = HashMap::new();

    for body_group in &input_data.body_groups {
        for body_part in &body_group.parts {
            if loaded_files.contains_key(&body_part.model_source) {
                continue;
            }

            let file_path = Path::new(&body_part.model_source);
            let file_exists = match file_path.try_exists() {
                Ok(exists) => exists,
                Err(error) => {
                    log(error.to_string(), LogLevel::Verbose);
                    return Err(ImportingError::FileNotFound);
                }
            };

            if !file_exists {
                log(format!("File {} could not be found!", body_part.model_source), LogLevel::Verbose);
                return Err(ImportingError::FileNotFound);
            }

            let file_extension = match file_path.extension() {
                Some(extension) => extension,
                None => {
                    log(format!("File {} has no extension!", body_part.model_source), LogLevel::Verbose);
                    return Err(ImportingError::NoFileExtension);
                }
            };

            let imported_file: Result<ImportedFileData, String> = match file_extension.to_str().unwrap() {
                "smd" => smd::load_smd(file_path, None).map_err(|error| error.to_string()), // TODO: Support vta file
                _ => {
                    log(format!("File {} is an unsupported format!", body_part.model_source), LogLevel::Verbose);
                    return Err(ImportingError::UnsupportedFile);
                }
            };

            match imported_file {
                Ok(file) => loaded_files.insert(body_part.model_source.clone(), file),
                Err(error) => {
                    log(format!("File {} failed to import due to: {}!", body_part.model_source, error), LogLevel::Debug);
                    return Err(ImportingError::FailedImport);
                }
            };
        }
    }

    Ok(loaded_files)
}

pub struct ImportedFileData {
    skeleton: Vec<ImportedBone>,
    animation: Vec<ImportedAnimationFrame>,
    pub mesh: ImportedModel,
    flexes: Vec<ImportedFlexKey>,
}

impl ImportedFileData {
    pub fn new() -> Self {
        Self {
            skeleton: Vec::new(),
            animation: Vec::new(),
            mesh: ImportedModel::new(),
            flexes: Vec::new(),
        }
    }
}

impl ImportedFileData {
    pub fn add_bone(&mut self, new_bone: ImportedBone) -> usize {
        self.skeleton.push(new_bone);
        self.skeleton.len() - 1
    }

    pub fn add_frame(&mut self, new_frame: ImportedAnimationFrame) -> usize {
        self.animation.push(new_frame);
        self.animation.len() - 1
    }
}

pub struct ImportedBone {
    name: String,
    position: Vector3,
    orintation: Quaternion,
    parent: Option<usize>,
}

impl ImportedBone {
    pub fn new(name: String, position: Vector3, orintation: Quaternion, parent: Option<usize>) -> Self {
        Self {
            name,
            position,
            orintation,
            parent,
        }
    }
}

pub struct ImportedAnimationFrame {
    bones: Vec<ImportedBoneAnimation>,
}

impl ImportedAnimationFrame {
    pub fn new() -> Self {
        Self { bones: Vec::new() }
    }
}

impl ImportedAnimationFrame {
    pub fn add_bone(&mut self, new_bone: ImportedBoneAnimation) {
        self.bones.push(new_bone);
    }
}

pub struct ImportedBoneAnimation {
    bone: usize,
    position: Vector3,
    orintation: Quaternion,
}

impl ImportedBoneAnimation {
    pub fn new(bone: usize, position: Vector3, orintation: Quaternion) -> Self {
        Self { bone, position, orintation }
    }
}

pub struct ImportedModel {
    materials: Vec<String>,
    vertices: Vec<ImportedVertex>,
    faces: Vec<ImportedFace>,
}

impl ImportedModel {
    pub fn new() -> Self {
        Self {
            materials: Vec::new(),
            vertices: Vec::new(),
            faces: Vec::new(),
        }
    }
}

impl ImportedModel {
    pub fn add_material(&mut self, new_material: String) -> usize {
        let existing = self.materials.iter().position(|material| material == &new_material);
        if let Some(index) = existing {
            return index;
        }

        self.materials.push(new_material);
        self.materials.len() - 1
    }

    pub fn add_vertex(&mut self, new_vertex: ImportedVertex) -> usize {
        self.vertices.push(new_vertex);
        self.vertices.len() - 1
    }

    pub fn add_face(&mut self, new_face: ImportedFace) {
        self.faces.push(new_face);
    }
}

pub struct ImportedVertex {
    position: Vector3,
    normal: Vector3,
    uv: Vector2,
    weights: Vec<(usize, f64)>,
}

impl ImportedVertex {
    pub fn new(position: Vector3, normal: Vector3, uv: Vector2) -> Self {
        Self {
            position,
            normal,
            uv,
            weights: Vec::new(),
        }
    }
}

impl ImportedVertex {
    pub fn add_weight(&mut self, bone_index: usize, weight: f64) {
        self.weights.push((bone_index, weight));
    }
}

pub struct ImportedFace {
    material_index: usize,
    vertex_indices: Vec<usize>,
}

impl ImportedFace {
    pub fn new(material_index: usize) -> Self {
        Self {
            material_index,
            vertex_indices: Vec::new(),
        }
    }
}

impl ImportedFace {
    pub fn add_vertex_index(&mut self, vertex_index: usize) {
        self.vertex_indices.push(vertex_index);
    }
}

pub struct ImportedFlexKey {}

impl ImportedFlexKey {}
