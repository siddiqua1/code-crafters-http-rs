use crate::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Version {
    Http1_1,
}

impl TryFrom<&str> for Version {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self> {
        match value {
            "HTTP/1.1" => return Ok(Version::Http1_1),
            _ => return Err(Error::HttpUnsupportedVersion),
        }
    }
}
