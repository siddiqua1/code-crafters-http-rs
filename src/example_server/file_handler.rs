#![allow(unused)]

use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug)]
pub struct BaseDirectoryNotFound;

#[derive(Debug)]
pub struct FileHandler {
    base_dir: PathBuf,
}

impl FileHandler {
    pub fn new(path: PathBuf) -> Result<FileHandler, BaseDirectoryNotFound> {
        if !path.is_absolute() {
            return Err(BaseDirectoryNotFound);
        }
        return Ok(FileHandler { base_dir: path });
    }

    pub fn search(&self, file: &str) -> Option<PathBuf> {
        let mut bfs: Vec<PathBuf> = vec![self.base_dir.clone()];

        while let Some(curr) = bfs.pop() {
            if let Ok(entries) = curr.read_dir() {
                for entry in entries.flatten() {
                    let next = entry.path();
                    if next.is_dir() {
                        bfs.push(next);
                        continue;
                    }
                    if next.ends_with(file) {
                        return Some(next);
                    }
                }
            }
        }

        return None;
    }

    pub fn read(&self, file: PathBuf) -> Vec<u8> {
        let mut file_buffer: Vec<u8> = Vec::new();
        if let Ok(mut file) = File::open(file) {
            let _ = file.read_to_end(&mut file_buffer);
        }
        return file_buffer;
    }

    pub fn write(&self, file: PathBuf, data: &[u8]) -> Result<usize, BaseDirectoryNotFound> {
        let mut file = match File::create(file) {
            Err(_) => return Err(BaseDirectoryNotFound),
            Ok(f) => f,
        };
        match file.write_all(data) {
            Ok(()) => return Ok(data.len()),
            Err(_e) => return Err(BaseDirectoryNotFound),
        }
    }

    pub fn get_path(&self, name: &str) -> PathBuf {
        return self.base_dir.join(name);
    }
}
