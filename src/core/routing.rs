use crate::core::context::ServerContext;
use crate::core::request::Request;
use anyhow::anyhow;
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use std::hash::Hash;

pub type RouteHandler = fn(req: &Request, ctx: &ServerContext) -> Result<Vec<u8>>;

pub struct Identifiers<'a> {
    pub path_values: HashMap<&'static str, &'a str>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Segments {
    Literal(&'static str),
    Capture(&'static str),
}

impl Segments {
    pub fn new(seg: &'static str) -> Result<Segments> {
        let chars = seg.chars().collect_vec();
        let n = chars.len();
        if n < 2 {
            return Ok(Segments::Literal(seg));
        }

        if chars[0] == '{' && chars[n - 1] == '}' {
            if n == 2 {
                return Err(anyhow!("Capture group does not have associated key."));
            }
            return Ok(Segments::Capture(&seg[1..n - 1]));
        }
        return Ok(Segments::Literal(seg));
    }
}

pub trait Routeable {
    fn new() -> Self;
    fn add_route(&mut self, path: &'static str, handler: RouteHandler) -> Result<()>;
    fn match_route(&self, path: &str) -> Option<RouteHandler>;
}

#[cfg(test)]
mod tests {
    use crate::core::routing::Segments;

    #[test]
    fn segments_literal() {
        let orig = "boop";
        let segment = Segments::new(orig);
        assert!(segment.is_ok());
        let segment = segment.unwrap();
        assert_eq!(segment, Segments::Literal(orig));
    }

    #[test]
    fn segments_capture() {
        let orig = "{capture}";
        let segment = Segments::new(orig);
        assert!(segment.is_ok());
        let segment = segment.unwrap();
        assert_eq!(segment, Segments::Capture("capture"));
    }

    #[test]
    fn segment_panic() {
        let orig = "{}";
        let segment = Segments::new(orig);
        assert!(segment.is_err())
    }
}
