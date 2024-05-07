use crate::core::request::HttpMethod;
use crate::core::request::Request;
use crate::core::response;
use crate::core::routing::Identifiers;
use anyhow::{anyhow, Result};

use crate::example_server::context::ServerContext;

pub fn files(req: &Request, _path_vals: &Identifiers, _ctx: &ServerContext) -> Result<Vec<u8>> {
    match req.method {
        HttpMethod::Get => return files_get(req, _path_vals, _ctx),
        HttpMethod::Post => return files_post(req, _path_vals, _ctx),
    }
}

pub fn files_get(
    _req: &Request,
    path_vals: &Identifiers,
    context: &ServerContext,
) -> Result<Vec<u8>> {
    let file_name = path_vals
        .path_values
        .get("file")
        .ok_or(anyhow!("file name not found"))?;
    match context.file_handler.search(file_name) {
        None => return Err(anyhow!("file not found on server")),
        Some(data) => {
            let data = context.file_handler.read(data);
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
    }
}

pub fn files_post(req: &Request, pv: &Identifiers, ctx: &ServerContext) -> Result<Vec<u8>> {
    let body = req
        .body
        .ok_or(anyhow!("No data to found to write to file"))?;
    let file_name = pv
        .path_values
        .get("file")
        .ok_or(anyhow!("file name not found"))?;
    let path = ctx.file_handler.get_path(file_name);
    let _written = match ctx.file_handler.write(path, body.as_bytes()) {
        Err(_) => return Err(anyhow!("Unable to write data to file")),
        Ok(b) => b,
    };
    return Ok(response::CREATED.to_vec());
}
