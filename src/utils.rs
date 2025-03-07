use csv::ReaderBuilder;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

#[derive(Debug, Deserialize)]
pub struct State {
    pub pc: usize,
    pub ap: usize,
    pub fp: usize,
}

pub fn load_trace(path: impl AsRef<Path>) -> Result<Vec<State>, csv::Error> {
    let file = File::open(path)?;
    let mut reader = ReaderBuilder::new().from_reader(BufReader::new(file));
    let mut states: Vec<State> = Vec::new();

    for result in reader.deserialize() {
        let state: State = result?;
        states.push(state);
    }

    Ok(states)
}

pub fn load_program(path: impl AsRef<Path>) -> std::io::Result<HashMap<usize, Vec<String>>> {
    let json = fs::read_to_string(path)?;
    let program: Value = serde_json::from_str(&json).expect("Failed to parse program.");
    let mut final_instructions: HashMap<usize, Vec<String>> = HashMap::new();
    if let Some(instructions) = program["debug_info"]["instruction_locations"].as_object() {
        instructions
            .iter()
            .filter_map(|(s, v)| v["accessible_scopes"].as_array().map(|value| (s, value)))
            .for_each(|(s, v)| {
                final_instructions.insert(
                    s.parse::<usize>()
                        .expect("Failed to parse instruction to integer."),
                    v.iter()
                        .filter_map(|value| match value {
                            Value::String(s) => Some(s.clone()),
                            _ => None,
                        })
                        .collect::<Vec<String>>(),
                );
            });
    }
    Ok(final_instructions)
}
