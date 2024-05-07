use crate::core::request::HttpMethod;
use crate::core::request::Request;
use crate::core::routing::Identifiers;
use anyhow::{anyhow, Result};

use crate::example_server::context::ServerContext;

pub fn echo(req: &Request, _path_vals: &Identifiers, _ctx: &ServerContext) -> Result<Vec<u8>> {
    if let HttpMethod::Get = req.method {
        return echo_get(req, _path_vals, _ctx);
    }
    return Err(anyhow!("Only GET is supported by echo"));
}

pub fn echo_get(_req: &Request, pv: &Identifiers, _ctx: &ServerContext) -> Result<Vec<u8>> {
    let msg = pv
        .path_values
        .get("msg")
        .ok_or(anyhow!("Message not supplied to echo"))?;
    let response = format!(
        "{}\r\n{}\r\nContent-Length: {}\r\n\r\n{}\r\n",
        "HTTP/1.1 200 OK",
        "Content-Type: text/plain",
        msg.len(),
        msg
    );
    return Ok(response.into_bytes());
}
