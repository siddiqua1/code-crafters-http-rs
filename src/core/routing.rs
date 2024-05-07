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

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
pub struct Identifiers<'a> {
    pub path_values: HashMap<&'static str, &'a str>,
}

pub type SyncRouteHandler<Context> =
    fn(req: &Request, path_vals: &Identifiers, ctx: &'static Context) -> Result<Vec<u8>>;

pub type AsyncRouteHandler<Context> =
    fn(req: &Request, path_vals: &Identifiers, ctx: &'static Context) -> BoxFuture<Result<Vec<u8>>>;

// TODO: once the following issue is stablized, switch to using impl trait in type alias
// https://github.com/rust-lang/rust/issues/63063
// pub type OtherAsycn<Context> = fn(
//     req: &Request,
//     path_vals: &Identifiers,
//     ctx: &Context,
// ) -> impl Future<Output = Result<Vec<u8>>>;

#[derive(Debug, Clone)]
pub enum RouteHandler<Context: Clone + 'static> {
    Sync(SyncRouteHandler<Context>),
    Async(AsyncRouteHandler<Context>),
}

// fn foo<Context>(u: impl Fn(&Request, &Identifiers, &Context) -> BoxFuture<Result<Vec<u8>>>) {}

impl<Context: Clone> From<SyncRouteHandler<Context>> for RouteHandler<Context> {
    fn from(f: SyncRouteHandler<Context>) -> Self {
        return Self::Sync(f);
    }
}
impl<Context: Clone> From<AsyncRouteHandler<Context>> for RouteHandler<Context> {
    fn from(f: AsyncRouteHandler<Context>) -> Self {
        return Self::Async(f);
    }
}

#[macro_export]
macro_rules! form_handler {
    ($l:tt, SyncRouteHandler) => {{
        use $crate::core::routing::RouteHandler;
        let handle: SyncRouteHandler<_> = $l;
        let handle: RouteHandler<_> = handle.into();
        handle
    }};
    ($l:tt, AsyncRouteHandler) => {{
        // use $crate::core::request::Request;
        use $crate::core::routing::AsyncRouteHandler;
        use $crate::core::routing::RouteHandler;

        let handle: AsyncRouteHandler<_> = $l;
        // let handle: AsyncRouteHandler<_> = move |r, p, c| Box::pin($l(r, p, c));
        let handle: RouteHandler<_> = handle.into();
        handle
    }};
}
#[allow(unused_imports)]
pub(crate) use form_handler;

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

pub trait Routeable<Context: Sized + Clone> {
    fn new() -> Self;
    fn add_route(&mut self, path: &'static str, handler: RouteHandler<Context>) -> Result<()>;
    fn add_route_async(
        &mut self,
        path: &'static str,
        handler: AsyncRouteHandler<Context>,
    ) -> Result<()>;
    fn add_route_sync(
        &mut self,
        path: &'static str,
        handler: SyncRouteHandler<Context>,
    ) -> Result<()>;

    fn match_route<'a>(&self, path: &'a str) -> Option<(RouteHandler<Context>, Identifiers<'a>)>;
}

#[derive(Debug)]
pub struct Route<Context: Clone + 'static> {
    pub path_segments: Vec<Segments>,
    pub handler: RouteHandler<Context>, // handler should not matter when comparing routes
}

impl<Context: Clone> Hash for Route<Context> {
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

impl<Context: Clone> Route<Context> {
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

impl<Context: Clone> Route<Context> {
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

impl<Context: Clone> PartialEq for Route<Context> {
    fn eq(&self, other: &Self) -> bool {
        return self.path_segments.eq(&other.path_segments);
    }
}

impl<Context: Clone> Eq for Route<Context> {}

impl<Context: Clone> Ord for Route<Context> {
    fn cmp(&self, other: &Self) -> Ordering {
        return self.path_segments.cmp(&other.path_segments);
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl<Context: Clone> PartialOrd for Route<Context> {
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

    #[derive(Debug, Clone, Copy)]
    struct PlaceholderContext {}

    use super::AsyncRouteHandler;
    use super::Identifiers;
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
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send>> {
        return Box::pin(async {
            return Err(anyhow!("thunk2"));
        });
    }

    // async fn thunk2_async(
    //     _req: &Request<'_>,
    //     _path_vals: &Identifiers<'_>,
    //     _ctx: &PlaceholderContext,
    // ) -> Result<Vec<u8>> {
    //     return Ok(Vec::new());
    // }

    // fn force_boxed<T>(f: fn(u32) -> T) -> Incrementer
    // where
    //     T: Future<Output = u32> + 'static,
    // {
    //     Box::new(move |n| Box::pin(f(n)))
    // }
    // checking that async syntax works in tandem with impl Future
    async fn _test() -> Result<()> {
        let req = Request::from(&[1])?;
        let path_vals = Identifiers {
            path_values: HashMap::new(),
        };
        let ctx = PlaceholderContext {};
        thunk_async(&req, &path_vals, &ctx).await?;
        return Ok(());
    }

    #[test]
    fn route_same_if_path_same() {
        let orig = "/some/path/potato";
        let handle: SyncRouteHandler<PlaceholderContext> = thunk;

        let route = Route::new(orig, handle);
        assert!(route.is_ok());
        let route = route.unwrap();

        let route2 = Route::new(orig, form_handler!(thunk2, SyncRouteHandler));
        assert!(route2.is_ok());
        let route2 = route2.unwrap();

        // let handle: AsyncRouteHandler<_> = thunk_async;
        let route3 = Route::new(orig, form_handler!(thunk_async, AsyncRouteHandler));
        assert!(route3.is_ok());
        let route3 = route3.unwrap();

        // fn wrapped_thunk2_async(
        //     r: &Request<'_>,
        //     p: &Identifiers<'_>,
        //     c: &PlaceholderContext,
        // ) -> BoxFuture<Result<Vec<u8>>> {
        //     return Box::pin(async { thunk2_async(r, p, c).await });
        // };

        // let async_handler: RouteHandler<PlaceholderContext> = wrapped_thunk2_async.into();

        // TODO: need to make work with regular async functions

        // let route4 = Route::new(orig, handler!(thunk2_async, AsyncRouteHandler));
        // assert!(route4.is_ok());
        // let route4 = route4.unwrap();

        assert_eq!(route, route2);
        assert_eq!(route, route3);
    }

    #[test]
    fn route_matches() {
        let orig = "/some/path/potato";
        let route = Route::new(orig, form_handler!(thunk, SyncRouteHandler));
        assert!(route.is_ok());
        let route = route.unwrap();
        assert!(route.matches(orig).is_some());
    }

    #[test]
    fn route_matches_captures() {
        let orig = "/some/{id}/potato";
        let route = Route::new(orig, form_handler!(thunk, SyncRouteHandler));
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
        let route = Route::new(orig, form_handler!(thunk, SyncRouteHandler));
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
        let route = Route::new(orig, form_handler!(thunk, SyncRouteHandler));
        assert!(route.is_ok());
        let route = route.unwrap();
        let id = 1234;
        let bad_path = format!("/some/potato/{}", id);
        let matches = route.matches(&bad_path);
        assert!(matches.is_none());
    }

    #[test]
    fn route_keys_unique() {
        let orig = "/some/{id}/potato/{id}";
        let route = Route::new(orig, form_handler!(thunk, SyncRouteHandler));
        assert!(route.is_err());
    }

    #[test]
    fn route_matches_same_var_twice() {
        let orig = "/some/{id}/potato/{id}";
        let route = Route::new(orig, form_handler!(thunk, SyncRouteHandler));
        assert!(route.is_err());
    }
}
