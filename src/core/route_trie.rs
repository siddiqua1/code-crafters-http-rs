// use crate::core::routing::Route;
// use crate::core::routing::{Identifiers, RouteHandler, Routeable, Segments};
// use anyhow::anyhow;
// use anyhow::Result;
// use itertools::Itertools;
// use std::collections::HashMap;

// pub struct RouteTrie<Context> {
//     literal_routes: HashMap<&'static str, Box<RouteTrie<Context>>>,
//     wild_route: Option<(&'static str, Box<RouteTrie<Context>>)>,
//     pub handler: Option<RouteHandler<Context>>,
// }

// impl<Context> RouteTrie<Context> {
//     fn add_segments(
//         &mut self,
//         segments: &[Segments],
//         handler: RouteHandler<Context>,
//     ) -> Result<()> {
//         if segments.is_empty() {
//             if self.handler.is_some() {
//                 return Err(anyhow!("This path already has a handler!"));
//             }
//             self.handler = Some(handler);
//             return Ok(());
//         }

//         let curr = &segments[0];

//         let next: &mut RouteTrie<Context> = match curr {
//             Segments::Literal(s) => self
//                 .literal_routes
//                 .entry(s)
//                 .or_insert(Box::new(RouteTrie::new())),
//             Segments::Capture(s) => {
//                 if let Some((set, _)) = self.wild_route {
//                     if &set != s {
//                         return Err(anyhow!(
//                             "Add route has capture group with key different than expected"
//                         ));
//                     }
//                 } else {
//                     self.wild_route = Some((s, Box::new(RouteTrie::new())));
//                 }

//                 let Some((_, next)) = self.wild_route.as_mut() else {
//                     return Err(anyhow!("Failed to set link to capture group"));
//                 };
//                 next
//             }
//         };
//         next.add_segments(&segments[1..], handler)?;

//         return Ok(());
//     }

//     fn match_internal<'a>(
//         &self,
//         path: &[&'a str],
//         mut path_values: HashMap<&'static str, &'a str>,
//     ) -> Option<(RouteHandler<Context>, Identifiers<'a>)> {
//         if path.is_empty() {
//             let Some(handler) = self.handler else {
//                 return None;
//             };
//             return Some((handler, Identifiers { path_values }));
//         }

//         // check literals first
//         if let Some(next) = self.literal_routes.get(path[0]) {
//             return next.match_internal(&path[1..], path_values);
//         }

//         // try wild then
//         if let Some((key, next)) = &self.wild_route {
//             path_values.insert(key, path[0]);
//             return next.match_internal(&path[1..], path_values);
//         }

//         return None;
//     }
// }

// impl<Context> Routeable<Context> for RouteTrie<Context> {
//     fn new() -> Self {
//         return RouteTrie {
//             literal_routes: HashMap::new(),
//             wild_route: None,
//             handler: None,
//         };
//     }

//     fn add_route(&mut self, path: &'static str, handler: RouteHandler<Context>) -> Result<()> {
//         let route = Route::new(path, handler)?;
//         self.add_segments(&route.path_segments, handler)?;
//         return Ok(());
//     }

//     fn match_route<'a>(&self, path: &'a str) -> Option<(RouteHandler<Context>, Identifiers<'a>)> {
//         let path = path.split('/').collect_vec();
//         return self.match_internal(&path, HashMap::new());
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::core::request::Request;
//     use crate::core::route_trie::RouteTrie;
//     use crate::core::routing::Identifiers;
//     use crate::core::routing::Routeable;
//     use anyhow::Result;

//     struct PlaceholderContext {}

//     fn thunk(_req: &Request, _paths: &Identifiers, _ctx: &PlaceholderContext) -> Result<Vec<u8>> {
//         return Ok(vec![1, 2, 3]);
//     }

//     const RAW_REQUEST: &[u8; 18] = b"GET / HTTP/1.1\r\n\r\n";

//     fn adding_new_literal(trie: &mut RouteTrie<PlaceholderContext>, path: &'static str) {
//         let test_request: Request = Request::from(RAW_REQUEST).unwrap();
//         assert!(trie.add_route(path, thunk).is_ok());
//         let out = trie.match_route(path);
//         assert!(out.is_some());
//         let (handler, ids) = out.unwrap();
//         let response = handler(&test_request, &ids, &PlaceholderContext {});
//         assert!(response.is_ok());
//         let response = response.unwrap();
//         assert_eq!(response, vec![1, 2, 3]);
//         assert!(ids.path_values.is_empty());
//         // should not be able to overwrite the handler once set
//         assert!(trie.add_route(path, thunk).is_err());
//     }

//     fn adding_invalid(trie: &mut RouteTrie<PlaceholderContext>, path: &'static str) {
//         assert!(trie.add_route(path, thunk).is_err());
//     }

//     fn adding_new_wild(trie: &mut RouteTrie<PlaceholderContext>, path: &'static str) {
//         let test_request: Request = Request::from(RAW_REQUEST).unwrap();
//         assert!(trie.add_route(path, thunk).is_ok());
//         let out = trie.match_route(path);
//         assert!(out.is_some());
//         let (handler, ids) = out.unwrap();
//         let response = handler(&test_request, &ids, &PlaceholderContext {});
//         assert!(response.is_ok());
//         let response = response.unwrap();
//         assert_eq!(response, vec![1, 2, 3]);
//         assert!(!ids.path_values.is_empty());
//         // should not be able to overwrite the handler once set
//         assert!(trie.add_route(path, thunk).is_err());
//     }

//     #[test]
//     fn trie_matches_literals() {
//         let mut trie = RouteTrie::<PlaceholderContext>::new();

//         adding_new_literal(&mut trie, "/");
//         adding_new_literal(&mut trie, "/user");
//         adding_new_literal(&mut trie, "/user/"); // checking distinction
//     }

//     #[test]
//     fn trie_no_match_literals_subpath() {
//         let mut trie = RouteTrie::<PlaceholderContext>::new();

//         adding_new_literal(&mut trie, "/");
//         adding_new_literal(&mut trie, "/user/dashboard");
//         assert!(trie.match_route("/user").is_none());
//         assert!(trie.match_route("/user/not_dashboard").is_none());
//         assert!(trie.match_route("/user/dashboard").is_some());
//     }

//     #[test]
//     fn trie_no_match_wild_subpath() {
//         let mut trie = RouteTrie::<PlaceholderContext>::new();

//         adding_new_literal(&mut trie, "/");
//         adding_new_wild(&mut trie, "/user/{id}/dashboard");
//         assert!(trie.match_route("/user").is_none());
//         assert!(trie.match_route("/user/1234").is_none());
//         assert!(trie.match_route("/user/1234/dashboard").is_some());
//     }

//     #[test]
//     fn trie_matches_wilds() {
//         let mut trie = RouteTrie::<PlaceholderContext>::new();

//         adding_new_literal(&mut trie, "/");
//         adding_new_wild(&mut trie, "/user/{id}");
//         adding_new_wild(&mut trie, "/user/{id}/");

//         // should not be able to rebind the wild match
//         adding_invalid(&mut trie, "/user/{msg}");

//         let path = "/user/1234";
//         let Some((_handler, ids)) = trie.match_route(path) else {
//             panic!("Should be valid route");
//         };
//         assert_eq!(ids.path_values.get("id"), Some(&"1234"));

//         let path = "/user/1234/";
//         let Some((_handler, ids)) = trie.match_route(path) else {
//             panic!("Should be valid route");
//         };
//         assert_eq!(ids.path_values.get("id"), Some(&"1234"));
//     }

//     #[test]
//     fn trie_matches_literal_over_wild() {
//         let mut trie = RouteTrie::<PlaceholderContext>::new();

//         adding_new_literal(&mut trie, "/");
//         adding_new_literal(&mut trie, "/user/dashboard");
//         adding_new_wild(&mut trie, "/user/{id}");

//         // should not be able to rebind the wild match
//         adding_invalid(&mut trie, "/user/{msg}");

//         let path = "/user/1234";
//         let Some((_handler, ids)) = trie.match_route(path) else {
//             panic!("Should be valid route");
//         };
//         assert_eq!(ids.path_values.get("id"), Some(&"1234"));

//         let path = "/user/dashboard";
//         let Some((_handler, ids)) = trie.match_route(path) else {
//             panic!("Should be valid route");
//         };
//         assert!(ids.path_values.is_empty());
//     }

//     #[test]
//     fn trie_matches_literal_over_wild_opposite_order() {
//         let mut trie = RouteTrie::<PlaceholderContext>::new();

//         adding_new_literal(&mut trie, "/");
//         adding_new_wild(&mut trie, "/user/{id}");
//         adding_new_literal(&mut trie, "/user/dashboard");

//         // should not be able to rebind the wild match
//         adding_invalid(&mut trie, "/user/{msg}");

//         let path = "/user/1234";
//         let Some((_handler, ids)) = trie.match_route(path) else {
//             panic!("Should be valid route");
//         };
//         assert_eq!(ids.path_values.get("id"), Some(&"1234"));

//         let path = "/user/dashboard";
//         let Some((_handler, ids)) = trie.match_route(path) else {
//             panic!("Should be valid route");
//         };
//         assert!(ids.path_values.is_empty());
//     }

//     #[test]
//     fn trie_does_not_accept_empty_capture() {
//         let mut trie = RouteTrie::<PlaceholderContext>::new();

//         adding_invalid(&mut trie, "/user/{}");
//     }
// }
