use std::fs;

pub fn create() {
    let template = include_str!("template.md");
    fs::write("presentation.md", template).unwrap();
}
