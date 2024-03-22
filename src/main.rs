#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

mod request;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                thread::spawn(move || {
                    handle_valid_connection(&mut _stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

#[allow(dead_code)]
const RESPONSE_OK: &[u8; 19] = b"HTTP/1.1 200 OK\r\n\r\n";
const RESPONSE_404: &[u8; 26] = b"HTTP/1.1 404 Not Found\r\n\r\n";

fn handle_valid_connection(stream: &mut TcpStream) {
    const MAX_HTTP_REQUEST_SIZE: usize = 8192;
    let mut read_buffer: [u8; MAX_HTTP_REQUEST_SIZE] = [0; MAX_HTTP_REQUEST_SIZE];
    // doing this really terribly on the first run
    if let Err(_e) = stream.read(&mut read_buffer) {
        println!("error: {}", _e);
    }
    let response = parse_request(&read_buffer);
    match stream.write(&response) {
        Ok(_bytes_written) => {
            println!("{} bytes were written to the connection", _bytes_written);
        }
        Err(_e) => {
            println!("error: {}", _e);
        }
    }
}

fn parse_request(read_buffer: &[u8]) -> Vec<u8> {
    let request = match request::Request::from(read_buffer) {
        Err(_e) => return RESPONSE_404.to_vec(),
        Ok(r) => r,
    };

    match request.handle_request() {
        Err(_e) => return RESPONSE_404.to_vec(),
        Ok(resp) => return resp,
    }
}

#[test]
fn request_base() {
    let request = "GET / HTTP/1.1\r\n\r\n";
    let response = parse_request(request.as_bytes());
    assert!(
        response == RESPONSE_OK.to_vec(),
        "{}",
        String::from_utf8(response).unwrap()
    )
}
#[test]
fn request_echo() {
    let request = "GET /echo/abc HTTP/1.1\r\n\r\n";
    let response = parse_request(request.as_bytes());
    let expected =
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 3\r\n\r\nabc\r\n";
    assert!(
        response == expected.as_bytes().to_vec(),
        "Got: {:?}",
        String::from_utf8(response).unwrap()
    );
}
#[test]
fn request_echo_2() {
    let request = "GET /echo/237/yikes-monkey HTTP/1.1\r\n\r\n";
    let response = parse_request(request.as_bytes());
    println!("Got: {:?}", String::from_utf8(response).unwrap());
}

#[test]
fn request_parse_headers() {
    let request = "GET /user-agent HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\n";
    let expected =
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 11\r\n\r\ncurl/7.64.1\r\n";
    let response = parse_request(request.as_bytes());
    assert!(
        response == expected.as_bytes().to_vec(),
        "Got: {:?}",
        String::from_utf8(response).unwrap()
    );
}

#[test]
fn request_parse_headers_bad() {
    let request =
        "GET /bad-user-agent HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\n";
    let expected = RESPONSE_404;
    let response = parse_request(request.as_bytes());
    assert!(
        response == expected.to_vec(),
        "Got: {:?}",
        String::from_utf8(response).unwrap()
    );
}
