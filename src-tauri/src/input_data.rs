use serde::Deserialize;

#[derive(Deserialize)]
pub struct CompilationDataInput {
    pub model_name: String,
    body_groups: Vec<BodyGroupInput>,
    animations: Vec<AnimationInput>,
    sequences: Vec<SequenceInput>,
}

#[derive(Deserialize)]
struct BodyGroupInput {
    name: String,
    parts: Vec<BodyPartInput>,
}

#[derive(Deserialize)]
struct BodyPartInput {
    name: String,
    is_blank: bool,
    model_source: String,
}

#[derive(Deserialize)]
struct AnimationInput {
    name: String,
    source_file: String,
}

#[derive(Deserialize)]
struct SequenceInput {
    name: String,
    animation: String,
}
