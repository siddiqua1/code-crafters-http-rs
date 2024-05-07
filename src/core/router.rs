use crate::core::request::Request;
use crate::core::response;
use crate::core::routing::AsyncRouteHandler;
use crate::core::routing::RouteHandler;
use crate::core::routing::Routeable;
use crate::core::routing::SyncRouteHandler;
use anyhow::Result;
use async_std::net::TcpListener;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::task::spawn;
use futures::stream::StreamExt;
// use std::sync::Arc;
// use std::io::{Read, Write};
// use std::thread;

pub struct Router<Context: Clone + 'static, T: Routeable<Context> + Sync + Send> {
    pub listener: TcpListener,
    routes: T,
    context: Context,
}

unsafe impl<Context: Clone + 'static, T: Routeable<Context> + Sync + Send> Send
    for Router<Context, T>
{
}

impl<Context: Clone + 'static, T: Routeable<Context> + Sync + Send> Router<Context, T> {
    pub async fn new(addr: &str, context: Context) -> Result<Router<Context, T>> {
        let listener = TcpListener::bind(addr).await?;
        return Ok(Router {
            listener,
            routes: T::new(),
            context,
        });
    }

    pub fn handle(&mut self, path: &'static str, handler: RouteHandler<Context>) -> Result<()> {
        return self.routes.add_route(path, handler);
    }

    pub fn handle_sync(
        &mut self,
        path: &'static str,
        handler: SyncRouteHandler<Context>,
    ) -> Result<()> {
        return self.routes.add_route_sync(path, handler);
    }

    pub fn handle_async(
        &mut self,
        path: &'static str,
        handler: AsyncRouteHandler<Context>,
    ) -> Result<()> {
        return self.routes.add_route_async(path, handler);
    }

    pub async fn handle_connection(&'static self, mut stream: TcpStream) {
        // println!("Incoming connection!");
        const MAX_HTTP_REQUEST_SIZE: usize = 8192;
        let mut read_buffer: [u8; MAX_HTTP_REQUEST_SIZE] = [0; MAX_HTTP_REQUEST_SIZE];
        if let Err(_e) = stream.read(&mut read_buffer).await {
            println!("Error reading from the connection: {}", _e);
            return;
        }
        // let response = parse_request(&read_buffer, context);
        let request = Request::from(&read_buffer);

        let response = match request {
            Err(_e) => {
                println!("{}", _e);
                response::NOT_FOUND.to_vec()
            }
            Ok(req) => self.handle_request(&req).await,
        };
        // println!("Responding: {:?}", response);
        if let Err(_e) = stream.write(&response).await {
            println!("Error writing to the connection: {}", _e);
            return;
        }
    }

    pub async fn handle_request(&'static self, request: &Request<'_>) -> Vec<u8> {
        let Some((handler, scope)) = self.routes.match_route(request.path) else {
            return response::NOT_FOUND.to_vec();
        };
        let result = match handler {
            RouteHandler::Sync(f) => f(request, &scope, &self.context),
            RouteHandler::Async(f) => f(request, &scope, &self.context).await,
        };
        match result {
            Ok(buf) => {
                return buf;
            }
            Err(_e) => {
                println!("{}", _e);
                return response::NOT_FOUND.to_vec();
            }
        }
    }
}

pub async fn run<Context, T>(app: &'static Router<Context, T>)
where
    Context: Clone + Sync + Send + 'static,
    T: Routeable<Context> + Sync + Send + 'static,
{
    // seems like clone isn't needed if we make app a lazy static, which is fine to do
    // as we do not support "runtime" support for additional routing
    // let app = Arc::new(app);
    app.listener
        .incoming()
        .for_each_concurrent(/* limit */ None, |tcpstream| {
            // let app = app.clone();
            return async move {
                let tcpstream = tcpstream.unwrap();
                spawn(app.handle_connection(tcpstream));
            };
        })
        .await;
}
#[cfg(test)]
mod tests {
    use crate::core::route_table::RouteTable;
    use crate::core::router::Router;

    struct PlaceholderContext {}

    #[async_std::test]
    async fn router_new_ok() {
        let app = Router::<_, RouteTable<_>>::new("127.0.0.1:4221", &PlaceholderContext {}).await;
        assert!(app.is_ok());
    }

    #[async_std::test]

    async fn router_new_err() {
        let app = Router::<_, RouteTable<_>>::new("not valid ip", &PlaceholderContext {}).await;
        assert!(app.is_err());
    }
}
