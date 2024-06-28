use std::{
    collections::HashMap,
    io::Error,
    path::Path,
    sync::{Arc, Mutex},
};

use smd::ParseSMDError;
use thiserror::Error as ThisError;

use crate::utilities::{
    logging::{log, LogLevel},
    mathematics::{Quaternion, Vector2, Vector3},
};

mod smd;

#[derive(Debug, Default)]
pub struct ImportFileData {
    pub skeleton: Vec<ImportBone>,
    pub animations: Vec<ImportAnimation>,
    pub parts: Vec<ImportPart>,
}

#[derive(Debug, Default)]
pub struct ImportBone {
    pub name: String,
    pub parent: Option<usize>,
    pub position: Vector3,
    pub orientation: Quaternion,
}

#[derive(Debug, Default)]
pub struct ImportAnimation {
    pub name: String,
    pub frame_count: usize,
    pub channels: Vec<ImportChannel>,
}

#[derive(Debug, Default)]
pub struct ImportChannel {
    pub bone: usize,
    pub position: Vec<ImportKeyFrame<Vector3>>,
    pub orientation: Vec<ImportKeyFrame<Quaternion>>,
}

#[derive(Debug, Default)]
pub struct ImportKeyFrame<T> {
    pub frame: usize,
    pub value: T,
}

#[derive(Debug, Default)]
pub struct ImportPart {
    pub name: String,
    pub vertices: Vec<ImportVertex>,
    pub polygons: HashMap<String, Vec<Vec<usize>>>,
    pub flexes: Vec<ImportFlex>,
}

#[derive(Clone, Debug, Default)]
pub struct ImportVertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub texture_coordinate: Vector2,
    pub links: Vec<ImportLink>,
}

#[derive(Clone, Debug, Default)]
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
    #[error("File Does Not Exist")]
    FileDoesNotExist,
    #[error("Failed To Open File")]
    FailedFileOpen(#[from] Error),
    #[error("File Does Not Have Extension")]
    FileDoesNotHaveExtension,
    #[error("File Format Is Not Supported")]
    UnsupportedFileFormat,
    #[error("Failed To Parse SMD File")]
    FailedSMDFileParse(#[from] ParseSMDError),
}

#[derive(Debug, Default)]
pub struct FileManager {
    pub files: Mutex<HashMap<String, (usize, Arc<ImportFileData>)>>,
}

impl FileManager {
    pub fn load_file(&self, path: String) -> Result<(), ParseError> {
        let file_path = Path::new(&path);
        let mut files = self.files.lock().unwrap();

        if files.contains_key(&path) {
            files.get_mut(&path).unwrap().0 += 1;
            return Ok(());
        }

        let exists = file_path.try_exists()?;

        if !exists {
            return Err(ParseError::FileDoesNotExist);
        }

        let file_extension = match file_path.extension() {
            Some(extension) => extension,
            None => return Err(ParseError::FileDoesNotHaveExtension),
        };

        let imported_file = match file_extension.to_str().expect("Failed To Convert File Extension To String!") {
            "smd" => smd::load_smd(file_path)?,
            "vta" => todo!("Support VTA Files!"),
            _ => return Err(ParseError::UnsupportedFileFormat),
        };

        log(format!("Loaded {:?} file: {}", file_extension.to_ascii_uppercase(), path), LogLevel::Verbose);
        files.insert(path, (1, Arc::new(imported_file)));
        Ok(())
    }

    pub fn unload_file(&self, path: String) {
        let mut files = self.files.lock().unwrap();

        if let Some((ref mut count, _)) = files.get_mut(&path) {
            *count -= 1;
            if *count == 0 {
                files.remove(&path);
            }
        }
    }

    pub fn get_file(&self, path: &str) -> Option<Arc<ImportFileData>> {
        match self.files.lock().unwrap().get(path) {
            Some((_, file_data)) => Some(Arc::clone(file_data)),
            None => None,
        }
    }
}
