use std::{collections::HashMap, path::Path};

use thiserror::Error;

use crate::{
    input::CompilationDataInput,
    utilities::{
        logging::{log, LogLevel},
        mathematics::{Quaternion, Vector2, Vector3},
    },
};

mod smd;

#[derive(Error, Debug)]
pub enum ImportingError {
    #[error("File Was Not Found")]
    FileNotFound,
    #[error("File Had No Extension")]
    NoFileExtension,
    #[error("File Is Not Supported")]
    UnsupportedFile,
    #[error("File Failed To Import")]
    FailedImport,
}

#[derive(Default)]
pub struct ImportedFileData {
    pub skeleton: Vec<ImportedBone>,
    pub animation: Vec<ImportedAnimationFrame>,
    pub mesh: ImportedMesh,
    pub flexes: Vec<ImportedFlexKey>,
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

    pub fn get_bone_by_index(&self, index: usize) -> &ImportedBone {
        // UNWRAP: The index should be valid
        self.skeleton.get(index).unwrap()
    }
}
pub struct ImportedBone {
    pub name: String,
    pub position: Vector3,
    pub orientation: Quaternion,
    pub parent: Option<usize>,
}

impl ImportedBone {
    pub fn new(name: String, position: Vector3, orientation: Quaternion, parent: Option<usize>) -> Self {
        Self {
            name,
            position,
            orientation,
            parent,
        }
    }
}

#[derive(Default)]
pub struct ImportedAnimationFrame {
    pub bones: Vec<ImportedBoneAnimation>,
}

impl ImportedAnimationFrame {
    pub fn add_bone(&mut self, new_bone: ImportedBoneAnimation) {
        self.bones.push(new_bone);
    }
}

pub struct ImportedBoneAnimation {
    pub bone: usize,
    pub position: Vector3,
    pub orientation: Quaternion,
}

impl ImportedBoneAnimation {
    pub fn new(bone: usize, position: Vector3, orientation: Quaternion) -> Self {
        Self { bone, position, orientation }
    }
}

#[derive(Default)]
pub struct ImportedMesh {
    pub materials: HashMap<String, Vec<Vec<usize>>>,
    pub vertices: Vec<ImportedVertex>,
}

impl ImportedMesh {
    fn add_vertex(&mut self, vertex: ImportedVertex) {
        self.vertices.push(vertex);
    }
}

pub struct ImportedVertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub uv: Vector2,
    pub weights: Vec<(usize, f64)>,
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

pub struct ImportedFlexKey {}

impl ImportedFlexKey {}

pub fn load_all_source_files(input_data: &CompilationDataInput) -> Result<HashMap<String, ImportedFileData>, ImportingError> {
    log("Loading all source files.", LogLevel::Info);

    let mut loaded_files: HashMap<String, ImportedFileData> = HashMap::new();

    for body_group in &input_data.body_groups {
        for body_part in &body_group.parts {
            load_file(&mut loaded_files, &body_part.model_source)?;
        }
    }

    for animation in &input_data.animations {
        load_file(&mut loaded_files, &animation.source_file)?;
    }

    Ok(loaded_files)
}

fn load_file(loaded_files: &mut HashMap<String, ImportedFileData>, source_path: &str) -> Result<(), ImportingError> {
    if loaded_files.contains_key(source_path) {
        return Ok(());
    }

    let file_path = Path::new(source_path);
    let file_exists = match file_path.try_exists() {
        Ok(exists) => exists,
        Err(error) => {
            log(error.to_string(), LogLevel::Verbose);
            return Err(ImportingError::FileNotFound);
        }
    };

    if !file_exists {
        log(format!("File {} could not be found!", source_path), LogLevel::Verbose);
        return Err(ImportingError::FileNotFound);
    }

    let file_extension = match file_path.extension() {
        Some(extension) => extension,
        None => {
            log(format!("File {} has no extension!", source_path), LogLevel::Verbose);
            return Err(ImportingError::NoFileExtension);
        }
    };

    let imported_file: Result<ImportedFileData, String> = match file_extension.to_str().unwrap() {
        "smd" => smd::load_smd(file_path, None).map_err(|error| error.to_string()), // TODO: Support vta file
        _ => {
            log(format!("File {} is an unsupported format!", source_path), LogLevel::Verbose);
            return Err(ImportingError::UnsupportedFile);
        }
    };

    match imported_file {
        Ok(file) => loaded_files.insert(source_path.to_string(), file),
        Err(error) => {
            log(format!("File {} failed to import due to: {}!", source_path, error), LogLevel::Debug);
            return Err(ImportingError::FailedImport);
        }
    };

    Ok(())
}
