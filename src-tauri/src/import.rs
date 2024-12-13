use std::{
    io::Error,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use indexmap::IndexMap;
use serde::Serialize;
use thiserror::Error as ThisError;

use crate::utilities::{
    logging::{log, LogLevel},
    mathematics::{Quaternion, Vector2, Vector3},
};

mod obj;
mod smd;

use obj::ParseOBJError;
use smd::ParseSMDError;

#[derive(Debug, Default, Serialize)]
pub struct ImportFileData {
    pub skeleton: Vec<ImportBone>,
    pub animations: Vec<ImportAnimation>,
    pub parts: Vec<ImportPart>,
}

#[derive(Debug, Default, Serialize)]
pub struct ImportBone {
    pub name: String,
    pub parent: Option<usize>,
    #[serde(skip_serializing)]
    pub position: Vector3,
    #[serde(skip_serializing)]
    pub orientation: Quaternion,
}

#[derive(Debug, Default, Serialize)]
pub struct ImportAnimation {
    pub name: String,
    #[serde(skip_serializing)]
    pub frame_count: usize,
    #[serde(skip_serializing)]
    pub channels: Vec<ImportChannel>,
}

#[derive(Debug, Default)]
pub struct ImportChannel {
    pub bone: usize,
    pub position: Vec<ImportKeyFrame<Vector3>>,
    pub rotation: Vec<ImportKeyFrame<Quaternion>>,
}

#[derive(Debug, Default)]
pub struct ImportKeyFrame<T> {
    pub frame: usize,
    pub value: T,
}

#[derive(Debug, Default, Serialize)]
pub struct ImportPart {
    pub name: String,
    #[serde(skip_serializing)]
    pub vertices: Vec<ImportVertex>,
    #[serde(skip_serializing)]
    pub polygons: IndexMap<String, Vec<Vec<usize>>>,
    #[serde(skip_serializing)]
    pub flexes: Vec<ImportFlex>,
}

#[derive(Debug, Default)]
pub struct ImportVertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub texture_coordinate: Vector2,
    pub links: Vec<ImportLink>,
}

#[derive(Debug, Default)]
pub struct ImportLink {
    pub bone: usize,
    pub weight: f64,
}

#[derive(Debug, Default)]
pub struct ImportFlex {
    pub name: Option<String>,
    pub vertices: Vec<ImportFlexVertex>,
}

#[derive(Debug, Default)]
pub struct ImportFlexVertex {
    pub index: usize,
    pub position: Vector3,
    pub normal: Vector3,
}

#[derive(Debug, ThisError)]
pub enum ParseError {
    #[error("Failed To Open File")]
    FailedFileOpen(#[from] Error),
    #[error("File Does Not Exist")]
    FileDoesNotExist,
    #[error("File Does Not Have Extension")]
    FileDoesNotHaveExtension,
    #[error("File Format Is Not Supported")]
    UnsupportedFileFormat,
    #[error("Failed To Parse SMD File: {0}")]
    FailedSMDFileParse(#[from] ParseSMDError),
    #[error("Failed To Parse OBJ File: {0}")]
    FailedOBJFileParse(#[from] ParseOBJError),
}

#[derive(Debug, Default)]
pub struct FileManager {
    pub files: Mutex<IndexMap<PathBuf, Arc<ImportFileData>>>,
}

impl FileManager {
    pub fn load_file(&self, path: String) -> Result<Arc<ImportFileData>, ParseError> {
        let file_path = PathBuf::from(path);
        let mut files = self.files.lock().unwrap();

        if let Some(file) = files.get(&file_path) {
            return Ok(Arc::clone(file));
        }

        if !file_path.try_exists()? {
            return Err(ParseError::FileDoesNotExist);
        }

        let file_extension = file_path.extension().ok_or_else(|| ParseError::FileDoesNotHaveExtension)?;

        let imported_file = match file_extension.to_string_lossy().to_lowercase().as_str() {
            "smd" => smd::load_smd(&file_path)?,
            "obj" => obj::load_obj(&file_path)?,
            _ => return Err(ParseError::UnsupportedFileFormat),
        };

        log(
            format!(
                "Loaded {} file: {}",
                file_extension.to_string_lossy().to_uppercase(),
                file_path.as_os_str().to_string_lossy()
            ),
            LogLevel::Verbose,
        );
        let file = Arc::new(imported_file);
        files.insert(file_path, Arc::clone(&file));
        Ok(file)
    }

    pub fn unload_file(&self, path: String) {
        let file_path = PathBuf::from(path);
        let mut files = self.files.lock().unwrap();
        files.swap_remove(&file_path);
    }

    pub fn get_file(&self, path: &str) -> Option<Arc<ImportFileData>> {
        let file_path = Path::new(path);
        self.files.lock().unwrap().get(file_path).cloned()
    }
}
