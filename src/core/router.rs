use crate::core::request::Request;
use crate::core::response;
use crate::core::routing::RouteHandler;
use crate::core::routing::Routeable;
use anyhow::Result;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

pub struct Router<Context, T: Routeable<Context> + Sync + Send> {
    pub listener: TcpListener,
    routes: T,
    context: Context,
}

impl<Context, T: Routeable<Context> + Sync + Send> Router<Context, T> {
    pub fn new(addr: &str, context: Context) -> Result<Router<Context, T>> {
        let listener = TcpListener::bind(addr)?;
        return Ok(Router {
            listener,
            routes: T::new(),
            context,
        });
    }

    pub fn handle(&mut self, path: &'static str, handler: RouteHandler<Context>) -> Result<()> {
        return self.routes.add_route(path, handler);
    }

    pub fn handle_connection(&self, mut stream: TcpStream) {
        const MAX_HTTP_REQUEST_SIZE: usize = 8192;
        let mut read_buffer: [u8; MAX_HTTP_REQUEST_SIZE] = [0; MAX_HTTP_REQUEST_SIZE];
        if let Err(_e) = stream.read(&mut read_buffer) {
            println!("Error reading from the connection: {}", _e);
            return;
        }
        // let response = parse_request(&read_buffer, context);
        let request = Request::from(&read_buffer);

        let response = match request {
            Err(_e) => response::NOT_FOUND.to_vec(),
            Ok(req) => self.handle_request(&req),
        };
        if let Err(_e) = stream.write(&response) {
            println!("Error writing to the connection: {}", _e);
            return;
        }
    }

    pub fn handle_request(&self, request: &Request) -> Vec<u8> {
        let Some((handler, scope)) = self.routes.match_route(request.path) else {
            return response::NOT_FOUND.to_vec();
        };
        if let Ok(buf) = handler(request, &scope, &self.context) {
            return buf;
        }
        return response::NOT_FOUND.to_vec();
    }
}

// should only be able to run the server when it is references static data in order to be thread-safe
impl<Context: Sync + Send + 'static, T: Routeable<Context> + Sync + Send + 'static>
    Router<Context, T>
{
    pub fn run(self) {
        let app = Arc::new(self);
        for stream in app.listener.incoming() {
            match stream {
                Ok(mut _stream) => {
                    let app = app.clone();
                    thread::spawn(move || {
                        app.handle_connection(_stream);
                    });
                }
                Err(e) => {
                    println!("error: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::context::get_context;
    use crate::core::route_table::RouteTable;
    use crate::core::router::Router;

    #[test]
    fn router_new_ok() {
        let app = Router::<_, RouteTable<_>>::new("127.0.0.1:4221", get_context());
        assert!(app.is_ok());
    }

    #[test]
    fn router_new_err() {
        let app = Router::<_, RouteTable<_>>::new("not valid ip", get_context());
        assert!(app.is_err());
    }
}
