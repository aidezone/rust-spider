use std::fs::{create_dir_all, write};
use std::path::Path;

pub struct Writer;

impl Writer {
    pub fn new() -> Self {
        Writer
    }

    pub fn write(&self, url: &str, content: &str) {
        let path = format!("output/{}.md", url.replace("://", "_").replace("/", "_"));
        let dir = Path::new(&path).parent().unwrap();
        create_dir_all(dir).unwrap();
        write(path, content).unwrap();
    }
}
