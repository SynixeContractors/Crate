use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use synixe_events::serde_json;

pub struct Store {
    path: PathBuf,
    items: Vec<String>,
}

impl Store {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let path = PathBuf::from(path);
        if !path.exists() {
            return Ok(Self {
                path,
                items: Vec::new(),
            });
        }
        let mut file = File::open(&path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        Ok(Self {
            path,
            items: serde_json::from_str(&buffer)?,
        })
    }

    pub fn add(&mut self, item: String) {
        self.items.push(item);
    }

    pub fn contains(&self, item: &str) -> bool {
        self.items.contains(&item.to_string())
    }

    // Save and load from disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let mut file = File::create(&self.path)?;
        file.write_all(serde_json::to_string(&self.items)?.as_bytes())?;
        Ok(())
    }
}
