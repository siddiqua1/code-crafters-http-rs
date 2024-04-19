use crate::core::request::HttpMethod;
use crate::core::request::Request;
use crate::core::response;
use crate::core::routing::Identifiers;
use anyhow::{anyhow, Result};

use crate::example_server::context::ServerContext;

pub fn files(req: &Request, path_vals: &Identifiers, ctx: &ServerContext) -> Result<Vec<u8>> {
    match req.method {
        HttpMethod::Get => {return get_files(req, path_vals, ctx); },
        HttpMethod::Post => {return post_files(req, path_vals, ctx)}
    }
}

fn get_files(_req: &Request, path_vals: &Identifiers, ctx: &ServerContext) -> Result<Vec<u8>> { 
    let Some(file) = path_vals.path_values.get("file") else {
        return Err(anyhow!("File not specified"));
    };

    let Some(content) = ctx.file_handler.search(file) else {
        return Err(anyhow!(format!("Unable to get content from {}", file)));
    };

    let data = ctx.file_handler.read(content);
    let response = format!(
        "{}\r\n{}\r\nContent-Length: {}\r\n\r\n",
        "HTTP/1.1 200 OK",
        "Content-Type: application/octet-stream",
        data.len()
    );
    let mut response = response.into_bytes();
    response.extend(data);
    response.extend(b"\r\n".to_vec());
    return Ok(response);
}

fn post_files(req: &Request, path_vals: &Identifiers, ctx: &ServerContext) -> Result<Vec<u8>> {
    let Some(body) = req.body else {
        return Err(anyhow!("No body found with request"))
    };
    // request parsing already ensures that content length exists and truncates body accordingly
    let Some(file) = path_vals.path_values.get("file") else {
        return Err(anyhow!("File not specified"));
    };
    let path = ctx.file_handler.get_path(file);
    let _written =  ctx
        .file_handler
        .write(path, &body.as_bytes())?;
    return Ok(response::CREATED.to_vec());
}