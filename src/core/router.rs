use crate::core::context::ServerContext;
use crate::core::routing::{Route, RouteHandler};
use anyhow::anyhow;
use anyhow::Result;
use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::core::request::Request;

pub struct Router<'a> {
    listener: TcpListener,
    routes: HashSet<Route<'a>>,
    context: ServerContext,
}

impl<'a> Router<'a> {
    fn new(addr: &str, context: ServerContext) -> Result<Router> {
        let listener = TcpListener::bind(addr)?;
        return Ok(Router {
            listener,
            routes: HashSet::new(),
            context,
        });
    }

    fn handle(&mut self, path: &'a str, handler: RouteHandler) -> Result<()> {
        if !self.routes.insert(Route::new(path, handler)?) {
            return Err(anyhow!(format!("Handler for {} already set!", path)));
        }
        return Ok(());
    }

    fn run(&'static self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(mut _stream) => {
                    // let context = Arc::clone(&context);
                    thread::spawn(move || {
                        self.handle_connection(&mut _stream);
                    });
                }
                Err(e) => {
                    println!("error: {}", e);
                }
            }
        }
    }

    fn handle_connection(&self, stream: &mut TcpStream) {
        const MAX_HTTP_REQUEST_SIZE: usize = 8192;
        let mut read_buffer: [u8; MAX_HTTP_REQUEST_SIZE] = [0; MAX_HTTP_REQUEST_SIZE];
        if let Err(_e) = stream.read(&mut read_buffer) {
            println!("Error reading from the connection: {}", _e);
            return;
        }
        // let response = parse_request(&read_buffer, context);
        let request = Request::from(&read_buffer);
        let response = [0];
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
