use thiserror::Error as ThisError;

use crate::input::ImputedCompilationData;

use super::ProcessedSequence;

#[derive(Debug, ThisError)]
pub enum ProcessingSequenceError {}

pub fn process_sequences(input: &ImputedCompilationData, remapped_animations: &[usize]) -> Result<Vec<ProcessedSequence>, ProcessingSequenceError> {
    let mut processed_sequences = Vec::with_capacity(input.sequences.len());

    for (_, input_sequence) in &input.sequences {
        let mut processed_sequence = ProcessedSequence {
            name: input_sequence.name.clone(),
            animations: vec![vec![0; input_sequence.animations.as_ref().unwrap()[0].len()]; input_sequence.animations.as_ref().unwrap().len()],
        };

        for (row_index, row_value) in input_sequence.animations.as_ref().unwrap().iter().enumerate() {
            for (column_index, column_value) in row_value.iter().enumerate() {
                let mapped_animation_index = remapped_animations[*column_value];

                processed_sequence.animations[row_index][column_index] = mapped_animation_index.try_into().unwrap();
            }
        }

        processed_sequences.push(processed_sequence);
    }

    Ok(processed_sequences)
}
