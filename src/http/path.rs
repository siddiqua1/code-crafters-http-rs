use crate::prelude::*;

pub struct Path<'a> {
    /// Exposing internal for ease of use when segmenting
    pub path: &'a str,
}

impl<'a> TryFrom<&'a str> for Path<'a> {
    type Error = Error;
    fn try_from(value: &'a str) -> Result<Self> {
        if value.is_empty() || value.chars().nth(0) != Some('/') {
            return Err(Error::HttpPathParsing);
        }
        return Ok(Path { path: value });
    }
}
