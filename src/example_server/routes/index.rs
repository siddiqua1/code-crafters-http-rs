use crate::core::request::HttpMethod;
use crate::core::request::Request;
use crate::core::response;
use crate::core::routing::Identifiers;
use anyhow::{anyhow, Result};

use crate::example_server::context::ServerContext;

pub fn index(req: &Request, _path_vals: &Identifiers, _ctx: &ServerContext) -> Result<Vec<u8>> {
    if let HttpMethod::Get = req.method {
        return Ok(response::OK.to_vec());
    }
    return Err(anyhow!("Only GET is supported by index"));
}
