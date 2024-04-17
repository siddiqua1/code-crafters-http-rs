use crate::core::file_handler::FileHandler;
use std::env;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ServerContext {
    pub file_handler: FileHandler,
}

pub fn get_context() -> ServerContext {
    let args: Vec<String> = env::args().collect();
    const DIR_FLAG: &str = "--directory";

    let mut idx = None;
    for (i, arg) in args.iter().enumerate() {
        if DIR_FLAG == arg {
            idx = Some(i);
            break;
        }
    }
    let mut base = env::current_dir().unwrap();
    if let Some(i) = idx {
        if i + 1 < args.len() {
            base = PathBuf::from(args[i + 1].clone());
        }
    }

    return ServerContext {
        file_handler: FileHandler::new(base).unwrap(),
    };
}
