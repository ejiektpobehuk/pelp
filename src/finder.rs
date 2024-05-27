use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use crate::file_access_logs::FileAccessLog;

// Temporary house for search functions until they find a better home.

pub fn global_search() {
    println!("Project file: {:?}\n", look_for_project_file());
    println!(
        "Markdown files in current directory: {:?}\n",
        look_for_md_files()
    );
    println!(
        "Recent files {:?}\n",
        FileAccessLog::load_from_file("/home/ejiek/.local/share/pelp/recent.db").entries
    );
    println!("A list of registered seried will be here some day\n");
}

pub fn look_for_project_file() -> Option<PathBuf> {
    let path = PathBuf::from("./pelp.toml");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

// Return all files in a Current Working Directory
pub fn look_for_md_files() -> Vec<PathBuf> {
    fs::read_dir(".")
        .expect("Unable to read current directory to look for .md file")
        .filter(|file| {
            match file
                .as_ref()
                .unwrap()
                .path()
                .extension()
                .and_then(OsStr::to_str)
            {
                Some("md") => true,
                _ => false,
            }
        })
        .map(|entrie| entrie.unwrap().path())
        .collect()
}

// Query entries from recent database
fn list_recent_files() {}
