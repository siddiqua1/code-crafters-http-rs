use crate::core::routing::{Identifiers, RouteHandler, Routeable, Segments};
use anyhow::anyhow;
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Eq, Debug)]
struct Route {
    path_segments: Vec<Segments>,
    pub handler: RouteHandler, // handler should not matter when comparing routes
}

fn has_unique_elements<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    return iter.into_iter().all(move |x| return uniq.insert(x));
}

impl Route {
    fn new(path: &'static str, handler: RouteHandler) -> Result<Route> {
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

    fn matches<'a>(&self, path: &'a str) -> Option<Identifiers<'a>> {
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

impl std::cmp::PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        return self.path_segments.eq(&other.path_segments);
    }
}

impl Hash for Route {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.path_segments.hash(hasher);
    }
}

pub struct RouteTable {
    routes: HashSet<Route>,
}

impl Routeable for RouteTable {
    fn new() -> Self {
        return RouteTable {
            routes: HashSet::new(),
        };
    }

    fn add_route(&mut self, path: &'static str, handler: RouteHandler) -> Result<()> {
        if !self.routes.insert(Route::new(path, handler)?) {
            return Err(anyhow!(format!("Handler for {} already set!", path)));
        }
        return Ok(());
    }

    fn match_route(&self, path: &str) -> Option<RouteHandler> {
        for route in &self.routes {
            if let Some(scope) = route.matches(path) {
                return Some(route.handler);
            }
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use crate::core::context::ServerContext;
    use crate::core::request::Request;
    use crate::core::route_table::Route;
    use anyhow::anyhow;
    use anyhow::Result;

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
