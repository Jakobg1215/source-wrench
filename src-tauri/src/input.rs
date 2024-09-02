use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ImputedCompilationData {
    pub model_name: String,
    pub export_path: String,
    pub body_parts: Vec<ImputedBodyPart>,
    pub animations: Vec<ImputedAnimation>,
    pub sequences: Vec<ImputedSequence>,
}

#[derive(Debug, Deserialize)]
pub struct ImputedBodyPart {
    pub name: String,
    pub models: Vec<ImputedModel>,
}

#[derive(Debug, Deserialize)]
pub struct ImputedModel {
    pub name: String,
    pub is_blank: bool,
    pub file_source: String,
    pub part_names: Vec<Option<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ImputedAnimation {
    pub name: String,
    pub file_source: String,
    pub animation_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ImputedSequence {
    pub name: String,
    pub animations: Vec<String>,
}
