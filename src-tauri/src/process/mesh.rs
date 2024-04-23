use std::collections::HashMap;

use crate::{import::ImportedFileData, input::CompilationDataInput};

use super::{bones::BoneTable, structures::ProcessedModelData, ProcessingDataError};

pub fn process_mesh_data(
    _input: &CompilationDataInput,
    _import: &HashMap<String, ImportedFileData>,
    _bone_table: &BoneTable,
) -> Result<ProcessedModelData, ProcessingDataError> {
    todo!()
}
