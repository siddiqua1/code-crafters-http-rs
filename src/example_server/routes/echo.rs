use crate::core::request::HttpMethod;
use crate::core::request::Request;
use crate::core::routing::Identifiers;
use anyhow::{anyhow, Result};

use crate::example_server::context::ServerContext;

pub fn echo(req: &Request, path_vals: &Identifiers, _ctx: &ServerContext) -> Result<Vec<u8>> {
    if let HttpMethod::Get = req.method {
        return get_echo(req, path_vals, _ctx);
    }
    return Err(anyhow!("Only GET is supported by endpoint"));
}

fn get_echo(_req: &Request, path_vals: &Identifiers, _ctx: &ServerContext) -> Result<Vec<u8>> {
    let message = path_vals.path_values.get("msg");

    let Some(message) = message else {
        return Err(anyhow!("Could not find message in path"));
    };
    let response = format!(
        "{}\r\n{}\r\nContent-Length: {}\r\n\r\n{}\r\n",
        "HTTP/1.1 200 OK",
        "Content-Type: text/plain",
        message.len(),
        message
    );
    return Ok(response.into_bytes());
}
