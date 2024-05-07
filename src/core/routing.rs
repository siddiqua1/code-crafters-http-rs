use crate::core::request::Request;
use anyhow::anyhow;
use anyhow::Result;
use core::future::Future;
use core::pin::Pin;
use itertools::Itertools;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::hash::Hasher;

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T>>>;
pub struct Identifiers<'a> {
    pub path_values: HashMap<&'static str, &'a str>,
}

pub type SyncRouteHandler<Context> =
    fn(req: &Request, path_vals: &Identifiers, ctx: &Context) -> Result<Vec<u8>>;

pub type AsyncRouteHandler<Context> =
    fn(req: &Request, path_vals: &Identifiers, ctx: &Context) -> BoxFuture<Result<Vec<u8>>>;

#[derive(Debug)]
pub enum RouteHandler<Context> {
    Sync(SyncRouteHandler<Context>),
    Async(AsyncRouteHandler<Context>),
}

pub trait MyFrom<T>: Sized {
    fn my_from(value: T) -> Self;
}

impl<Context> From<SyncRouteHandler<Context>> for RouteHandler<Context> {
    fn from(f: SyncRouteHandler<Context>) -> Self {
        return Self::Sync(f);
    }
}
impl<Context> From<AsyncRouteHandler<Context>> for RouteHandler<Context> {
    fn from(f: AsyncRouteHandler<Context>) -> Self {
        return Self::Async(f);
    }
}

#[derive(Debug)]
pub enum Segments {
    Literal(&'static str),
    Capture(&'static str),
}

impl PartialEq for Segments {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Segments::Capture(_), Segments::Capture(_)) => return true,
            (Segments::Literal(a), Segments::Literal(b)) => return a == b,
            _ => return false,
        }
    }
}

impl Eq for Segments {}

impl Hash for Segments {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        match self {
            Segments::Literal(s) => s.hash(hasher),
            _ => 0.hash(hasher),
        }
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Segments {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(other));
    }
}

impl Ord for Segments {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Segments::Capture(_), Segments::Capture(_)) => return Ordering::Equal,
            (Segments::Literal(a), Segments::Literal(b)) => return a.cmp(b),
            (Segments::Capture(_), Segments::Literal(_)) => return Ordering::Greater,
            (Segments::Literal(_), Segments::Capture(_)) => return Ordering::Less,
        }
    }
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

pub trait Routeable<Context: Sized> {
    fn new() -> Self;
    fn add_route(&mut self, path: &'static str, handler: RouteHandler<Context>) -> Result<()>;

    fn match_route<'a>(
        &self,
        path: &'a str,
    ) -> Option<(AsyncRouteHandler<Context>, Identifiers<'a>)>;
}

#[derive(Debug)]
pub struct Route<Context> {
    pub path_segments: Vec<Segments>,
    pub handler: RouteHandler<Context>, // handler should not matter when comparing routes
}

impl<Context> Hash for Route<Context> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.path_segments.hash(hasher);
    }
}

fn has_unique_elements<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    return iter.into_iter().all(move |x| return uniq.insert(x));
}

fn make_segments(path: &'static str) -> Result<Vec<Segments>> {
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
    return Ok(path_segments);
}

impl<Context> Route<Context> {
    pub fn matches<'b>(&self, path: &'b str) -> Option<Identifiers<'b>> {
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

impl<Context> Route<Context> {
    pub fn new(
        path: &'static str,
        handler: impl Into<RouteHandler<Context>>,
    ) -> Result<Route<Context>> {
        let path_segments = make_segments(path)?;
        return Ok(Route {
            path_segments,
            handler: handler.into(),
        });
    }
}

impl<Context> PartialEq for Route<Context> {
    fn eq(&self, other: &Self) -> bool {
        return self.path_segments.eq(&other.path_segments);
    }
}

impl<Context> Eq for Route<Context> {}

impl<Context> Ord for Route<Context> {
    fn cmp(&self, other: &Self) -> Ordering {
        return self.path_segments.cmp(&other.path_segments);
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl<Context> PartialOrd for Route<Context> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(other));
    }
}

#[cfg(test)]
mod tests {
    use crate::core::request::Request;
    use crate::core::routing::Route;
    use crate::core::routing::Segments;
    use anyhow::anyhow;
    use anyhow::Result;
    use core::pin::Pin;
    use std::collections::HashMap;
    use std::future::Future;

    #[derive(Debug)]
    struct PlaceholderContext {}

    use super::AsyncRouteHandler;
    use super::Identifiers;
    use super::RouteHandler;
    use super::SyncRouteHandler;

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

    fn thunk(
        _req: &Request,
        _path_vals: &Identifiers,
        _ctx: &PlaceholderContext,
    ) -> Result<Vec<u8>> {
        return Ok(Vec::new());
    }

    fn thunk2(
        _req: &Request,
        _path_vals: &Identifiers,
        _ctx: &PlaceholderContext,
    ) -> Result<Vec<u8>> {
        return Err(anyhow!("thunk2"));
    }

    fn thunk_async(
        _req: &Request<'_>,
        _path_vals: &Identifiers<'_>,
        _ctx: &PlaceholderContext,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>>>> {
        return Box::pin(async {
            return Err(anyhow!("thunk2"));
        });
    }

    macro_rules! handler {
        ($l:tt, SyncRouteHandler) => {{
            let handle: SyncRouteHandler<_> = $l;
            let handle: RouteHandler<_> = handle.into();
            handle
        }};
        ($l:tt, AsyncRouteHandler) => {{
            let handle: AsyncRouteHandler<_> = $l;
            let handle: RouteHandler<_> = handle.into();
            handle
        }};
    }

    #[test]
    fn route_same_if_path_same() {
        let orig = "/some/path/potato";
        let handle: SyncRouteHandler<PlaceholderContext> = thunk;

        let route = Route::new(orig, handle);
        assert!(route.is_ok());
        let route = route.unwrap();

        let route2 = Route::new(orig, handler!(thunk2, SyncRouteHandler));
        assert!(route2.is_ok());
        let route2 = route2.unwrap();

        let handle: AsyncRouteHandler<_> = thunk_async;

        let route3 = Route::new(orig, handler!(thunk_async, AsyncRouteHandler));
        assert!(route3.is_ok());
        let route3 = route3.unwrap();

        assert_eq!(route, route2);
        assert_eq!(route, route3);
    }

    // #[test]
    // fn route_matches() {
    //     let orig = "/some/path/potato";
    //     let route = Route::new(orig, thunk.into());
    //     assert!(route.is_ok());
    //     let route = route.unwrap();
    //     assert!(route.matches(orig).is_some());
    // }

    // #[test]
    // fn route_matches_captures() {
    //     let orig = "/some/{id}/potato";
    //     let route = Route::new(orig, thunk);
    //     assert!(route.is_ok());
    //     let route = route.unwrap();
    //     for id in 0..10 {
    //         let path = format!("/some/{}/potato", id);
    //         let matches = route.matches(&path);
    //         assert!(matches.is_some());
    //         let vals = matches.unwrap();
    //         let id = id.to_string();
    //         assert_eq!(vals.path_values.get("id").unwrap(), &&id);
    //     }
    // }

    // #[test]
    // fn route_matches_captures_multiple() {
    //     let orig = "/some/{id}/potato/{msg}";
    //     let route = Route::new(orig, thunk);
    //     assert!(route.is_ok());
    //     let route = route.unwrap();
    //     let id = 1234;
    //     let msg = "elegant_message";
    //     let path = format!("/some/{}/potato/{}", id, msg);
    //     let matches = route.matches(&path);
    //     assert!(matches.is_some());
    //     let vals = matches.unwrap();
    //     let id = id.to_string();
    //     assert!(vals.path_values.get("id").is_some());
    //     assert!(vals.path_values.get("msg").is_some());
    //     assert_eq!(vals.path_values.get("id").unwrap(), &&id);
    //     assert_eq!(vals.path_values.get("msg").unwrap(), &msg);
    // }

    // #[test]
    // fn route_matches_fails() {
    //     let orig = "/some/{id}/potato";
    //     let route = Route::new(orig, thunk);
    //     assert!(route.is_ok());
    //     let route = route.unwrap();
    //     let id = 1234;
    //     let bad_path = format!("/some/potato/{}", id);
    //     let matches = route.matches(&bad_path);
    //     assert!(matches.is_none());
    // }

    // #[test]
    // fn route_keys_unique() {
    //     let orig = "/some/{id}/potato/{id}";
    //     let route = Route::new(orig, thunk);
    //     assert!(route.is_err());
    // }

    // #[test]
    // fn route_matches_same_var_twice() {
    //     let orig = "/some/{id}/potato/{id}";
    //     let route = Route::new(orig, thunk);
    //     assert!(route.is_err());
    // }
}
