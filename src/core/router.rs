use crate::core::context::ServerContext;
use crate::core::response;
use crate::core::routing::{Route, RouteHandler};
use anyhow::anyhow;
use anyhow::Result;
use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use crate::core::request::Request;

pub struct Router<'a> {
    pub listener: TcpListener,
    routes: HashSet<Route<'a>>,
    context: ServerContext,
}

pub fn run(router: Router<'static>) {
    let router = Arc::new(router);
    for stream in router.listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let router = router.clone();

                thread::spawn(move || {
                    router.handle_connection(_stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

impl<'a> Router<'a> {
    pub fn new(addr: &str, context: ServerContext) -> Result<Router> {
        let listener = TcpListener::bind(addr)?;
        return Ok(Router {
            listener,
            routes: HashSet::new(),
            context,
        });
    }

    pub fn handle(&mut self, path: &'a str, handler: RouteHandler) -> Result<()> {
        if !self.routes.insert(Route::new(path, handler)?) {
            return Err(anyhow!(format!("Handler for {} already set!", path)));
        }
        return Ok(());
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
        let response = response::OK.to_owned();
        if let Err(_e) = stream.write(&response) {
            println!("Error writing to the connection: {}", _e);
            return;
        }
    }

    pub fn handle_request(&self, request: &Request) -> Vec<u8> {
        for route in &self.routes {
            // if route.matches(request.path) {}
        }

        todo!();
    }
}

#[cfg(test)]
mod tests {
    use crate::core::context::get_context;
    use crate::core::router::Router;
    #[test]
    fn router_new_ok() {
        let app = Router::new("127.0.0.1:4221", get_context());
        assert!(app.is_ok());
    }

    #[test]
    fn router_new_err() {
        let app = Router::new("not valid ip", get_context());
        assert!(app.is_err());
    }
}
