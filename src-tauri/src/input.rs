use serde::Deserialize;

#[derive(Deserialize)]
pub struct CompilationDataInput {
    pub model_name: String,
    pub body_groups: Vec<BodyGroupInput>,
    pub animations: Vec<AnimationInput>,
    pub sequences: Vec<SequenceInput>,
    pub export_path: String,
}

#[derive(Deserialize)]
pub struct BodyGroupInput {
    pub name: String,
    pub parts: Vec<BodyPartInput>,
}

#[derive(Deserialize)]
pub struct BodyPartInput {
    pub name: String,
    pub is_blank: bool,
    pub model_source: String,
}

#[derive(Deserialize)]
pub struct AnimationInput {
    pub name: String,
    pub source_file: String,
}

#[derive(Deserialize)]
pub struct SequenceInput {
    pub name: String,
    pub animation: String,
}
