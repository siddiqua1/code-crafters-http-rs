// use crate::core::routing::{Identifiers, Route, RouteHandler, Routeable};
// use anyhow::anyhow;
// use anyhow::Result;
// use itertools::Itertools;
// use std::collections::HashSet;

// pub struct RouteTable<Context> {
//     routes: HashSet<Route<Context>>,
// }

// impl<Context> Routeable<Context> for RouteTable<Context> {
//     fn new() -> Self {
//         return RouteTable {
//             routes: HashSet::new(),
//         };
//     }

//     fn add_route(&mut self, path: &'static str, handler: RouteHandler<Context>) -> Result<()> {
//         if !self.routes.insert(Route::new(path, handler)?) {
//             return Err(anyhow!(format!("Handler for {} already set!", path)));
//         }
//         return Ok(());
//     }

//     fn match_route<'a>(&self, path: &'a str) -> Option<(RouteHandler<Context>, Identifiers<'a>)> {
//         // need to sort routes so that we try literals first then captures
//         let sorted_routes = self.routes.iter().sorted().collect_vec();

//         for route in sorted_routes {
//             if let Some(scope) = route.matches(path) {
//                 return Some((route.handler, scope));
//             }
//         }
//         return None;
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::core::request::Request;
//     use crate::core::route_table::RouteTable;
//     use crate::core::routing::Identifiers;
//     use crate::core::routing::Routeable;
//     use anyhow::Result;

//     struct PlaceholderContext {}

//     fn thunk(_req: &Request, _paths: &Identifiers, _ctx: &PlaceholderContext) -> Result<Vec<u8>> {
//         return Ok(vec![1, 2, 3]);
//     }

//     const RAW_REQUEST: &[u8; 18] = b"GET / HTTP/1.1\r\n\r\n";

//     fn adding_new_literal(table: &mut RouteTable<PlaceholderContext>, path: &'static str) {
//         let test_request: Request = Request::from(RAW_REQUEST).unwrap();
//         assert!(table.add_route(path, thunk).is_ok());
//         let out = table.match_route(path);
//         assert!(out.is_some());
//         let (handler, ids) = out.unwrap();
//         let response = handler(&test_request, &ids, &PlaceholderContext {});
//         assert!(response.is_ok());
//         let response = response.unwrap();
//         assert_eq!(response, vec![1, 2, 3]);
//         assert!(ids.path_values.is_empty());
//         // should not be able to overwrite the handler once set
//         assert!(table.add_route(path, thunk).is_err());
//     }

//     fn adding_invalid(table: &mut RouteTable<PlaceholderContext>, path: &'static str) {
//         assert!(table.add_route(path, thunk).is_err());
//     }

//     fn adding_new_wild(table: &mut RouteTable<PlaceholderContext>, path: &'static str) {
//         let test_request: Request = Request::from(RAW_REQUEST).unwrap();
//         assert!(table.add_route(path, thunk).is_ok());
//         let out = table.match_route(path);
//         assert!(out.is_some());
//         let (handler, ids) = out.unwrap();
//         let response = handler(&test_request, &ids, &PlaceholderContext {});
//         assert!(response.is_ok());
//         let response = response.unwrap();
//         assert_eq!(response, vec![1, 2, 3]);
//         assert!(!ids.path_values.is_empty());
//         // should not be able to overwrite the handler once set
//         assert!(table.add_route(path, thunk).is_err());
//     }

//     #[test]
//     fn table_matches_literals() {
//         let mut table = RouteTable::<PlaceholderContext>::new();

//         adding_new_literal(&mut table, "/");
//         adding_new_literal(&mut table, "/user");
//         adding_new_literal(&mut table, "/user/"); // checking distinction
//     }

//     #[test]
//     fn table_no_match_literals_subpath() {
//         let mut table = RouteTable::<PlaceholderContext>::new();

//         adding_new_literal(&mut table, "/");
//         adding_new_literal(&mut table, "/user/dashboard");
//         assert!(table.match_route("/user").is_none());
//         assert!(table.match_route("/user/not_dashboard").is_none());
//         assert!(table.match_route("/user/dashboard").is_some());
//     }

//     #[test]
//     fn table_no_match_wild_subpath() {
//         let mut table = RouteTable::<PlaceholderContext>::new();

//         adding_new_literal(&mut table, "/");
//         adding_new_wild(&mut table, "/user/{id}/dashboard");
//         assert!(table.match_route("/user").is_none());
//         assert!(table.match_route("/user/1234").is_none());
//         assert!(table.match_route("/user/1234/dashboard").is_some());
//     }

//     #[test]
//     fn table_matches_wilds() {
//         let mut table = RouteTable::<PlaceholderContext>::new();

//         adding_new_literal(&mut table, "/");
//         adding_new_wild(&mut table, "/user/{id}");
//         adding_new_wild(&mut table, "/user/{id}/");

//         // should not be able to rebind the wild match
//         adding_invalid(&mut table, "/user/{msg}");

//         let path = "/user/1234";
//         let Some((_handler, ids)) = table.match_route(path) else {
//             panic!("Should be valid route");
//         };
//         assert_eq!(ids.path_values.get("id"), Some(&"1234"));

//         let path = "/user/1234/";
//         let Some((_handler, ids)) = table.match_route(path) else {
//             panic!("Should be valid route");
//         };
//         assert_eq!(ids.path_values.get("id"), Some(&"1234"));
//     }

//     #[test]
//     fn table_matches_literal_over_wild() {
//         let mut table = RouteTable::<PlaceholderContext>::new();

//         adding_new_literal(&mut table, "/");
//         adding_new_literal(&mut table, "/user/dashboard");
//         adding_new_wild(&mut table, "/user/{id}");

//         // should not be able to rebind the wild match
//         adding_invalid(&mut table, "/user/{msg}");

//         let path = "/user/1234";
//         let Some((_handler, ids)) = table.match_route(path) else {
//             panic!("Should be valid route");
//         };
//         assert_eq!(ids.path_values.get("id"), Some(&"1234"));

//         let path = "/user/dashboard";
//         let Some((_handler, ids)) = table.match_route(path) else {
//             panic!("Should be valid route");
//         };
//         assert!(ids.path_values.is_empty());
//     }

//     #[test]
//     fn table_matches_literal_over_wild_opposite_order() {
//         let mut table = RouteTable::<PlaceholderContext>::new();

//         adding_new_literal(&mut table, "/");
//         adding_new_wild(&mut table, "/user/{id}");
//         adding_new_literal(&mut table, "/user/dashboard");

//         // should not be able to rebind the wild match
//         adding_invalid(&mut table, "/user/{msg}");

//         let path = "/user/1234";
//         let Some((_handler, ids)) = table.match_route(path) else {
//             panic!("Should be valid route");
//         };
//         assert_eq!(ids.path_values.get("id"), Some(&"1234"));

//         let path = "/user/dashboard";
//         let Some((_handler, ids)) = table.match_route(path) else {
//             panic!("Should be valid route");
//         };
//         assert!(ids.path_values.is_empty());
//     }

//     #[test]
//     fn table_does_not_accept_empty_capture() {
//         let mut table = RouteTable::<PlaceholderContext>::new();

//         adding_invalid(&mut table, "/user/{}");
//     }
// }
