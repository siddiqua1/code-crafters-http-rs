use itertools::Itertools;

use crate::core::context::ServerContext;
use crate::core::request::Request;
use anyhow::Result;
use std::collections::HashMap;

pub type RouteHandler = fn(req: &Request, ctx: &ServerContext) -> Result<Vec<u8>>;

pub struct Identifiers<'a> {
    pub path_value: HashMap<&'a str, &'a str>,
}

// owning should be ok here since they are defined once
#[derive(PartialEq, Eq, Hash)]
pub struct Route {
    path_segments: Vec<String>,
    handler: RouteHandler,
}

impl Route {
    pub fn new(path: &str, handler: RouteHandler) -> Route {
        return Route {
            path_segments: path.split('/').map(|s| return s.to_owned()).collect_vec(),
            handler,
        };
    }

    pub fn matches<'a>(&self, path: &'a str) -> Result<Identifiers<'a>> {
        todo!()
    }
}

// impl std::cmp::PartialEq for Route {
//     fn eq(&self, other: &Self) -> bool {
//         return self.path_segments == other.path_segments;
//     }
// }
