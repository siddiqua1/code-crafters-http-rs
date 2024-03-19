// Uncomment this block to pass the first stage
use std::{io::{Read, Write}, net::{TcpListener, TcpStream}};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                handle_valid_connection(&mut _stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

const RESPONSE_OK: &[u8; 19] = b"HTTP/1.1 200 OK\r\n\r\n";
const RESPONSE_404: &[u8; 26] = b"HTTP/1.1 404 Not Found\r\n\r\n";

fn handle_valid_connection(stream: &mut TcpStream) {
    println!("accepted new connection");
    let response = parse_request(stream);
    match stream.write(response) {
        Ok(_bytes_written) => {
            println!("{} bytes were written to the connection", _bytes_written);
        }
        Err(_e) => {
            println!("error: {}", _e);
        }
    }
}

fn parse_request(stream: &mut TcpStream) -> &[u8] {
    const MAX_HTTP_REQUEST_SIZE: usize = 8192;
    let mut read_buffer: [u8; MAX_HTTP_REQUEST_SIZE] = [0; MAX_HTTP_REQUEST_SIZE];
    // doing this really terribly on the first run
    if let Err(_e) = stream.read(&mut read_buffer) {
        println!("error: {}", _e);
    }
    const VALID_REQUEST: &[u8; 16] = b"GET / HTTP/1.1\r\n";
    if &read_buffer[0..16] == VALID_REQUEST {
        return RESPONSE_OK;
    } else {
        return RESPONSE_404;
    }
}