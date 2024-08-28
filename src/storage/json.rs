use std::{fs::File, io::Write, path::PathBuf};

use anyhow::{Ok, Result};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

use crate::types::position::{self, Position};

use super::storage::Storage;

#[derive(Serialize, Default)]
struct DataAsJson {
    last_block_indexed: u64,
    positions: HashMap<u64, Position>,
}

impl DataAsJson {
    pub fn new(last_block_indexed: u64, positions: HashMap<u64, Position>) -> Self {
        DataAsJson {
            last_block_indexed,
            positions,
        }
    }
    pub fn as_tuple(&self) -> (u64, HashMap<u64, Position>) {
        (self.last_block_indexed, self.positions.clone())
    }
}

pub struct JsonStorage {
    file_path: PathBuf,
    last_saved_data: DataAsJson,
}

impl JsonStorage {
    pub fn new(path: &str) -> Self {
        JsonStorage {
            file_path: PathBuf::from(path),
            last_saved_data: DataAsJson::default(),
        }
    }
}

impl Storage for JsonStorage {
    fn load_state(&mut self) -> Result<(u64, HashMap<u64, Position>)> {
        if !self.file_path.exists() {
            self.last_saved_data = DataAsJson::new(0, HashMap::new());
            return Ok(self.last_saved_data.as_tuple());
        }
        let json_value: Value = serde_json::from_reader(File::open(self.file_path.clone())?)?;
        let last_block_indexed: u64 = match json_value.get("last_block_indexed") {
            Some(Value::Number(lbi)) => {
                if lbi.is_u64() {
                    //safe unwrap we check everything before
                    lbi.as_u64().unwrap()
                } else {
                    0 as u64
                }
            }
            _ => 0 as u64,
        };
        // no need to go further if last block indexed is genesis
        if last_block_indexed == 0 {
            self.last_saved_data = DataAsJson::new(0, HashMap::new());
            return Ok(self.last_saved_data.as_tuple());
        }
        let positions: HashMap<u64, Position> = match json_value.get("positions") {
            Some(Value::Object(map)) => map
                .iter()
                .filter_map(|(key, value)| {
                    let key = key.parse::<u64>().ok()?;
                    let position: Position = serde_json::from_value(value.clone()).ok()?;
                    Some((key, position))
                })
                .collect(),
            _ => HashMap::new(),
        };
        self.last_saved_data = DataAsJson::new(last_block_indexed, positions);
        Ok(self.last_saved_data.as_tuple())
    }

    async fn save_state(
        &mut self,
        positions: HashMap<u64, position::Position>,
        last_block_indexed: u64,
    ) -> anyhow::Result<()> {
        let map = DataAsJson {
            last_block_indexed,
            positions: positions,
        };
        // Serialize the HashMap to a JSON string
        let json = serde_json::to_string_pretty(&map)?;

        // Write the JSON string to a file
        let mut file = File::create(self.file_path.clone())?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }

    fn get_last_saved_positions_map(&self) -> HashMap<u64, Position> {
        self.last_saved_data.positions.clone()
    }
}
