use itertools::Itertools;

use crate::core::context::ServerContext;
use crate::core::request::Request;
use anyhow::anyhow;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub type RouteHandler = fn(req: &Request, ctx: &ServerContext) -> Result<Vec<u8>>;

pub struct Identifiers<'a, 'b> {
    pub path_values: HashMap<&'a str, &'b str>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Segments<'a> {
    Literal(&'a str),
    Capture(&'a str),
}

impl<'a> Segments<'a> {
    pub fn new(seg: &'a str) -> Result<Segments> {
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

#[derive(Eq, Hash, Debug)]
pub struct Route<'a> {
    path_segments: Vec<Segments<'a>>,
    handler: RouteHandler, // handler should not matter when comparing routes
}

fn has_unique_elements<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    return iter.into_iter().all(move |x| return uniq.insert(x));
}

impl<'a> Route<'a> {
    pub fn new(path: &'a str, handler: RouteHandler) -> Result<Route> {
        let path_segments = path
            .split('/')
            .map(|s| return Segments::new(s))
            .collect_vec();
        if path_segments.iter().any(|e| return e.is_err()) {
            return Err(anyhow!(
                "Capture group(s) do not have associated key: {path}"
            ));
        }
        let captured = path_segments.iter().filter_map(|s| match s {
            Ok(Segments::Capture(x)) => return Some(x),
            _ => return None,
        });
        if !has_unique_elements(captured) {
            return Err(anyhow!(
                "Same key used with multiple capture groups: {}",
                path
            ));
        }
        let path_segments = path_segments
            .into_iter()
            .filter_map(|res| match res {
                Ok(x) => return Some(x),
                _ => return None,
            })
            .collect_vec();
        return Ok(Route {
            path_segments,
            handler,
        });
    }

    pub fn matches<'b>(&self, path: &'b str) -> Option<Identifiers<'a, 'b>> {
        let mut path_values = HashMap::new();

        let path = path.split('/').collect_vec();
        if path.len() != self.path_segments.len() {
            return None;
        }
        for (segment, p) in self.path_segments.iter().zip(path.iter()) {
            match segment {
                Segments::Capture(key) => {
                    path_values.insert(*key, *p);
                }
                Segments::Literal(route) => {
                    if route != p {
                        return None;
                    }
                }
            }
        }

        return Some(Identifiers { path_values });
    }
}

impl<'a> std::cmp::PartialEq for Route<'a> {
    fn eq(&self, other: &Self) -> bool {
        return self.path_segments.eq(&other.path_segments);
    }
}

#[cfg(test)]
mod tests {
    use crate::core::context::ServerContext;
    use crate::core::request::Request;
    use crate::core::routing::Route;
    use crate::core::routing::Segments;
    use anyhow::anyhow;
    use anyhow::Result;

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

    fn thunk(_req: &Request, _ctx: &ServerContext) -> Result<Vec<u8>> {
        return Ok(Vec::new());
    }

    fn thunk2(_req: &Request, _ctx: &ServerContext) -> Result<Vec<u8>> {
        return Err(anyhow!("thunk2"));
    }

    #[test]
    fn route_same_if_path_same() {
        let orig = "/some/path/potato";
        let route = Route::new(orig, thunk);
        assert!(route.is_ok());
        let route = route.unwrap();

        let route2 = Route::new(orig, thunk2);
        assert!(route2.is_ok());
        let route2 = route2.unwrap();

        assert_eq!(route, route2)
    }

    #[test]
    fn route_matches() {
        let orig = "/some/path/potato";
        let route = Route::new(orig, thunk);
        assert!(route.is_ok());
        let route = route.unwrap();
        assert!(route.matches(orig).is_some());
    }

    #[test]
    fn route_matches_captures() {
        let orig = "/some/{id}/potato";
        let route = Route::new(orig, thunk);
        assert!(route.is_ok());
        let route = route.unwrap();
        for id in 0..10 {
            let path = format!("/some/{}/potato", id);
            let matches = route.matches(&path);
            assert!(matches.is_some());
            let vals = matches.unwrap();
            let id = id.to_string();
            assert_eq!(vals.path_values.get("id").unwrap(), &&id);
        }
    }

    #[test]
    fn route_matches_captures_multiple() {
        let orig = "/some/{id}/potato/{msg}";
        let route = Route::new(orig, thunk);
        assert!(route.is_ok());
        let route = route.unwrap();
        let id = 1234;
        let msg = "elegant_message";
        let path = format!("/some/{}/potato/{}", id, msg);
        let matches = route.matches(&path);
        assert!(matches.is_some());
        let vals = matches.unwrap();
        let id = id.to_string();
        assert!(vals.path_values.get("id").is_some());
        assert!(vals.path_values.get("msg").is_some());
        assert_eq!(vals.path_values.get("id").unwrap(), &&id);
        assert_eq!(vals.path_values.get("msg").unwrap(), &msg);
    }

    #[test]
    fn route_matches_fails() {
        let orig = "/some/{id}/potato";
        let route = Route::new(orig, thunk);
        assert!(route.is_ok());
        let route = route.unwrap();
        let id = 1234;
        let bad_path = format!("/some/potato/{}", id);
        let matches = route.matches(&bad_path);
        assert!(matches.is_none());
    }

    #[test]
    fn route_matches_same_var_twice() {
        let orig = "/some/{id}/potato/{id}";
        let route = Route::new(orig, thunk);
        assert!(route.is_err());
    }
}
