use crate::prelude::*;

#[derive(Debug, PartialEq)]
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

/// Allow for quick checking of `Path` struct against string literals
impl<'a> PartialEq<&str> for Path<'a> {
    fn eq(&self, other: &&str) -> bool {
        return self.path.eq(*other);
    }
}
