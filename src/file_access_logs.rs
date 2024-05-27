use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FileAccess {
    path: String,
    accessed: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileAccessLog {
    pub entries: Vec<FileAccess>,
}

impl FileAccessLog {
    pub fn add(&mut self, path: PathBuf) {
        let canonical_path = path.canonicalize().unwrap();
        let entry = FileAccess {
            path: canonical_path.into_os_string().into_string().unwrap(),
            accessed: chrono::Utc::now(),
        };
        self.entries.push(entry);
    }

    pub fn write_to_file(&self, path: &str) {
        let serialized = serde_json::to_string(&self).unwrap();
        std::fs::write(path, serialized.as_bytes()).unwrap();
    }

    pub fn load_from_file(path: &str) -> FileAccessLog {
        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap(),
            // TODO: Create empty only when file is missing
            Err(_) => FileAccessLog { entries: vec![] },
        }
    }
}
