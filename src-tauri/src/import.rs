use std::{collections::HashMap, path::Path};

use indexmap::IndexMap;
use thiserror::Error;

use crate::{
    input::CompilationDataInput,
    utilities::{
        logging::{log, LogLevel},
        mathematics::{Quaternion, Vector2, Vector3},
    },
};

use self::smd::ParseSMDError;

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
    #[error("Failed To Read SMD File")]
    SmdReadError(#[from] ParseSMDError),
}

#[derive(Default)]
pub struct ImportedFileData {
    pub skeleton: Vec<ImportedBone>,
    pub remapped_bones: HashMap<usize, usize>,
    pub animation: Vec<IndexMap<usize, ImportedBoneAnimation>>,
    pub mesh: ImportedMesh,
    pub flexes: Vec<ImportedFlexKey>,
}

impl ImportedFileData {
    pub fn add_bone(&mut self, new_bone: ImportedBone) -> usize {
        self.skeleton.push(new_bone);
        self.animation.push(IndexMap::new());
        self.skeleton.len() - 1
    }

    pub fn get_frame_count(&self) -> usize {
        *self.animation.iter().flat_map(|bone| bone.keys()).max().unwrap_or(&usize::MAX) + 1
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

#[derive(Clone, Copy)]
pub struct ImportedBoneAnimation {
    pub position: Vector3,
    pub orientation: Quaternion,
}

impl ImportedBoneAnimation {
    pub fn new(position: Vector3, orientation: Quaternion) -> Self {
        Self { position, orientation }
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

#[derive(Clone)]
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
            return Err(ImportingError::FileNotFound); // FIXME: This should be a different error.
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

    let imported_file = match file_extension.to_str().expect("Failed To Convert File Extension To String!") {
        "smd" => smd::load_smd(file_path, None)?, // TODO: Support vta file
        _ => {
            log(format!("File {} is an unsupported format!", source_path), LogLevel::Verbose);
            return Err(ImportingError::UnsupportedFile);
        }
    };

    loaded_files.insert(source_path.to_string(), imported_file);

    Ok(())
}
