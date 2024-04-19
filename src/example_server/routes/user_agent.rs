use crate::core::request::HttpMethod;
use crate::core::request::Request;
use crate::core::routing::Identifiers;
use anyhow::{anyhow, Result};

use crate::example_server::context::ServerContext;

pub fn user_agent(req: &Request, path_vals: &Identifiers, _ctx: &ServerContext) -> Result<Vec<u8>> {
    if let HttpMethod::Get = req.method {
        return get_user_agent(req, path_vals, _ctx);
    }
    return Err(anyhow!("Only GET is supported by this endpoint"));
}

fn get_user_agent(
    req: &Request,
    _path_vals: &Identifiers,
    _ctx: &ServerContext,
) -> Result<Vec<u8>> {
    let Some(agent) = req.headers.get("User-Agent") else {
        return Err(anyhow!("User agent not found in headers"));
    };
    let response = format!(
        "{}\r\n{}\r\nContent-Length: {}\r\n\r\n{}\r\n",
        "HTTP/1.1 200 OK",
        "Content-Type: text/plain",
        agent.len(),
        agent
    );
    return Ok(response.into_bytes());
}
