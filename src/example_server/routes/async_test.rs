use crate::core::request::HttpMethod;
use crate::core::request::Request;
use crate::core::response;
use crate::core::routing::Identifiers;
use anyhow::{anyhow, Result};
use async_std::task;
use core::future::Future;
use core::pin::Pin;
use std::time::Duration;

use crate::example_server::context::ServerContext;

// sadly unable to use async keyword directly rn

// pub fn async_test(
//     req: &Request,
//     _path_vals: &Identifiers,
//     _ctx: &'static ServerContext,
// ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send>> {
//     return Box::pin(async {
//         if let HttpMethod::Get = req.method {
//             return async_test_get(req, _path_vals, _ctx).await;
//         }
//         return Err(anyhow!("Only GET is supported by async"));
//     });
// }

fn thunk_async<'a>(
    _req: &Request<'a>,
    _path_vals: &Identifiers<'a>,
    _ctx: &ServerContext,
) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send>> {
    return Box::pin(async {
        task::sleep(Duration::from_secs(5)).await;
        return Ok(response::OK.to_vec());
    });
}

pub async fn async_test_get(
    _req: &Request<'_>,
    _pv: &Identifiers<'_>,
    _ctx: &ServerContext,
) -> Result<Vec<u8>> {
    task::sleep(Duration::from_secs(5)).await;
    return Ok(response::OK.to_vec());
}
