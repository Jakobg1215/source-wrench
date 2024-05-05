use std::collections::HashMap;

use crate::{import::ImportedFileData, input::CompilationDataInput};

use super::{bones::BoneTable, structures::ProcessedModelData, ProcessingDataError};

pub fn process_mesh_data(
    input: &CompilationDataInput,
    import: &HashMap<String, ImportedFileData>,
    bone_table: &BoneTable,
) -> Result<ProcessedModelData, ProcessingDataError> {
    todo!()
}
