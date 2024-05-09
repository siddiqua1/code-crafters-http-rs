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
use std::sync::Arc;

pub struct Identifiers<'a> {
    pub path_values: HashMap<&'static str, &'a str>,
}

pub type SyncRouteHandler<'a, Context> = fn(
    req: &'a Request<'a>,
    path_vals: &'a Identifiers<'a>,
    ctx: &'static Context,
) -> Result<Vec<u8>>;

pub type BoxedAsync<'a, Context> = Arc<
    dyn Fn(
        &'a Request<'a>,
        &'a Identifiers<'a>,
        &'static Context,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>>>>,
>;

// impl<'a, Context: Clone + Copy + 'static> Clone for BoxedAsync<'a, Context> {}
#[derive(Clone)]
pub struct AsyncRouteHandler<'a, Context: Clone + Copy + 'static> {
    pub handler: BoxedAsync<'a, Context>,
}

#[derive(Clone)]
pub enum RouteHandler<'a, Context: Clone + Copy + 'static> {
    Sync(SyncRouteHandler<'a, Context>),
    Async(AsyncRouteHandler<'a, Context>),
}

pub fn force_boxed<'a, Context: Clone + Copy + 'static, F, T>(
    f: F,
) -> AsyncRouteHandler<'a, Context>
where
    F: 'static + Fn(&'a Request<'a>, &'a Identifiers<'a>, &'static Context) -> T,
    T: Future<Output = Result<Vec<u8>>> + 'static,
{
    return AsyncRouteHandler {
        handler: Arc::new(move |r, p, c| {
            return Box::pin(f(r, p, c));
        }),
    };
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

pub trait Routeable<'a, Context: Clone + Copy + 'static + Sized> {
    fn new() -> Self;
    fn add_route_async<'b, F, T>(&mut self, path: &'static str, handler: F) -> Result<()>
    where
        F: 'static + Fn(&'b Request<'b>, &'b Identifiers<'b>, &'static Context) -> T + Clone,
        T: Future<Output = Result<Vec<u8>>> + 'static,
        'a: 'b,
        'b: 'a;

    fn add_route_async2(
        &mut self,
        path: &'static str,
        handler: AsyncRouteHandler<'a, Context>,
    ) -> Result<()>;
    fn add_route_sync(
        &mut self,
        path: &'static str,
        handler: SyncRouteHandler<'_, Context>,
    ) -> Result<()>;
    // 'b: 'a;

    fn match_route<'b>(
        &self,
        path: &'b str,
    ) -> Option<(RouteHandler<'b, Context>, Identifiers<'b>)>
    where
        // 'a: 'b,
        'b: 'a;
}

pub struct Route<'a, Context: Clone + Copy + 'static> {
    pub path_segments: Vec<Segments>,
    pub handler: RouteHandler<'a, Context>, // handler should not matter when comparing routes
}

impl<'a, Context: Clone + Copy + 'static> Hash for Route<'a, Context> {
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

impl<'a, Context: Clone + Copy + 'static> Route<'a, Context> {
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

impl<'a, Context: Clone + Copy + 'static> Route<'a, Context> {
    pub fn new(
        path: &'static str,
        handler: impl Into<RouteHandler<'a, Context>>,
    ) -> Result<Route<'a, Context>> {
        let path_segments = make_segments(path)?;
        return Ok(Route {
            path_segments,
            handler: handler.into(),
        });
    }

    pub fn new_sync(
        path: &'static str,
        handler: SyncRouteHandler<'a, Context>,
    ) -> Result<Route<'a, Context>> {
        let path_segments = make_segments(path)?;
        return Ok(Route {
            path_segments,
            handler: RouteHandler::Sync(handler),
        });
    }

    pub fn new_async<F, T>(path: &'static str, func: F) -> Result<Route<'a, Context>>
    where
        F: 'static + Fn(&'a Request<'a>, &'a Identifiers<'a>, &'static Context) -> T,
        T: Future<Output = Result<Vec<u8>>> + 'static,
    {
        let path_segments = make_segments(path)?;
        return Ok(Route {
            path_segments,
            handler: RouteHandler::Async(force_boxed(func)),
        });
    }

    pub fn new_async2(
        path: &'static str,
        func: AsyncRouteHandler<'a, Context>,
    ) -> Result<Route<'a, Context>> {
        let path_segments = make_segments(path)?;
        return Ok(Route {
            path_segments,
            handler: RouteHandler::Async(func),
        });
    }
}

impl<'a, Context: Clone + Copy + 'static> PartialEq for Route<'a, Context> {
    fn eq(&self, other: &Self) -> bool {
        return self.path_segments.eq(&other.path_segments);
    }
}

impl<'a, Context: Clone + Copy + 'static> Eq for Route<'a, Context> {}

impl<Context: Clone + Copy + 'static> Ord for Route<'_, Context> {
    fn cmp(&self, other: &Self) -> Ordering {
        return self.path_segments.cmp(&other.path_segments);
    }
}

impl<Context: Clone + Copy + 'static> PartialOrd for Route<'_, Context> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(other));
    }
}

impl<Context: Clone + Copy + 'static> std::fmt::Debug for Route<'_, Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Route: [{:?}]", self.path_segments);
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

    async fn thunk2_async(
        _req: &Request<'_>,
        _path_vals: &Identifiers<'_>,
        _ctx: &PlaceholderContext,
    ) -> Result<Vec<u8>> {
        return Ok(Vec::new());
    }

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

        let route = Route::new_sync(orig, handle);
        assert!(route.is_ok());
        let route = route.unwrap();

        let route2 = Route::new_sync(orig, thunk2);
        assert!(route2.is_ok());
        let route2 = route2.unwrap();

        // let handle: AsyncRouteHandler<_> = thunk_async;
        let route3 = Route::new_async(orig, thunk_async);
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

        let route4 = Route::new_async(orig, thunk2_async);
        assert!(route4.is_ok());
        let route4 = route4.unwrap();

        assert_eq!(route, route2);
        assert_eq!(route, route3);
        assert_eq!(route, route4);
    }

    #[test]
    fn route_matches() {
        let orig = "/some/path/potato";
        let route = Route::new_sync(orig, thunk);
        assert!(route.is_ok());
        let route = route.unwrap();
        assert!(route.matches(orig).is_some());
    }

    #[test]
    fn route_matches_captures() {
        let orig = "/some/{id}/potato";
        let route = Route::new_sync(orig, thunk);
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
        let route = Route::new_sync(orig, thunk);
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
        let route = Route::new_sync(orig, thunk);
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
        let route = Route::new_sync(orig, thunk);
        assert!(route.is_err());
    }

    #[test]
    fn route_matches_same_var_twice() {
        let orig = "/some/{id}/potato/{id}";
        let route = Route::new_sync(orig, thunk);
        assert!(route.is_err());
    }
}
