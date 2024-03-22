use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct BaseDirectoryNotFound;

pub struct FileHandler {
    base_dir: PathBuf,
}

impl FileHandler {
    pub fn from(path: &str) -> Result<FileHandler, BaseDirectoryNotFound> {
        let path = Path::new("path");
        if !path.is_absolute() {
            return Err(BaseDirectoryNotFound);
        }
        return Ok(FileHandler {
            base_dir: path.into(),
        });
    }

    pub fn new(path: PathBuf) -> Result<FileHandler, BaseDirectoryNotFound> {
        if !path.is_absolute() {
            return Err(BaseDirectoryNotFound);
        }
        return Ok(FileHandler {
            base_dir: path.into(),
        });
    }

    pub fn search(&self, file: &str) -> Option<Vec<u8>> {
        let mut bfs: Vec<PathBuf> = vec![self.base_dir.clone().into()];

        while let Some(curr) = bfs.pop() {
            if let Ok(entries) = curr.read_dir() {
                for entry in entries.flatten() {
                    let next = entry.path();
                    if next.is_dir() {
                        bfs.push(next);
                        continue;
                    }
                    if next.ends_with(file) {
                        let mut file = match File::open(next) {
                            Err(_) => return None,
                            Ok(f) => f,
                        };
                        let mut file_buffer: Vec<u8> = Vec::new();
                        if file.read_to_end(&mut file_buffer).is_err() {
                            return None;
                        };
                        return Some(file_buffer);
                    }
                }
            }
        }

        return None;
    }
}
