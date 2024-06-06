use crate::prelude::*;

#[derive(Debug, PartialEq)]
pub enum Method {
    Get,
    Post,
}

impl TryFrom<&str> for Method {
    type Error = Error;
    fn try_from(method: &str) -> Result<Self> {
        match method {
            "GET" => return Ok(Self::Get),
            "POST" => return Ok(Self::Post),
            _ => return Err(Error::HttpUnsupportedMethod),
        }
    }
}
