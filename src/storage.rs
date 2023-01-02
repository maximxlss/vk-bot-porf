use std::fs::File;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Storage {
    #[serde(default = "default_chance")]
    chance: u32,
    #[serde(default = "default_length")]
    length: u32
}

fn default_chance() -> u32 {
    10
}

fn default_length() -> u32 {
    30
}

impl Storage {
    pub fn new() -> Self {
        Self {
            chance: 10,
            length: 30
        }
    }

    pub fn load() -> Self {
        let file = File::open("storage.json").unwrap_or_else(|_| {
            Self::new().save();
            File::open("storage.json").unwrap()
        });
        serde_json::from_reader(file).unwrap()
    }

    pub fn save(&mut self) {
        let file = File::options().write(true).truncate(true).open("storage.json").unwrap_or_else(|_| File::create("storage.json").unwrap());
        serde_json::to_writer(file, self).unwrap();
    }

    pub fn get_chance(&self) -> u32 {
        self.chance
    }

    pub fn set_chance(&mut self, value: u32) {
        self.chance = value;
        self.save();
    }

    pub fn get_length(&self) -> u32 {
        self.chance
    }

    pub fn set_length(&mut self, value: u32) {
        self.length = value;
        self.save();
    }
}
