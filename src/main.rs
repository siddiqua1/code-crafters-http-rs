// Uncomment this block to pass the first stage
use std::{io::{Read, Write}, net::{TcpListener, TcpStream}};
use anyhow::Error;
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

fn parse_request<'a>(read_buffer: &[u8]) -> Vec<u8> {
    let request_words = request_to_words(read_buffer);
    if let Err(_e) = request_words { return RESPONSE_404.to_vec(); }
    let request_words = request_words.unwrap();
    let status_line = &request_words[0];

    if status_line[2] != "HTTP/1.1" { return RESPONSE_404.to_vec(); }
    if status_line[0] != "GET" { return RESPONSE_404.to_vec(); }

    let path = status_line[1];
    let paths = path.split("/").collect::<Vec<&str>>();
    println!("{:?}", paths);
    if paths.len() == 2 && paths[1] == "" { return RESPONSE_OK.to_vec(); }
    if paths.len() == 3 && paths[1] == "echo" {
        let echo = paths[2];
        let response = format!("{}\r\n{}\r\nContent-Length: {}\r\n\r\n{}\r\n",
            "HTTP/1.1 200 OK",
            "Content-Type: text/plain",
            echo.len(),
            echo
        );
        return response.into_bytes();
    }

    return RESPONSE_404.to_vec();
}

fn request_to_words(read_buffer: &[u8]) -> Result<Vec<Vec<&str>>, Error>{
    let request = std::str::from_utf8(read_buffer)?;
    let words = request
        .split("\r\n")
        .map(|line| {
            line.split(" ")
            .collect::<Vec<&str>>()
        })
        .collect::<Vec<Vec<&str>>>();
    return Ok(words);
}

#[test]
fn request_to_words_test() {
    let example_request = "GET /echo/abc HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\n";
    if let Ok(words) = request_to_words(example_request.as_bytes()) {
        println!("{:?}", words);
    }
}

#[test]
fn request_base() {
    let request = "GET / HTTP/1.1\r\n\r\n";
    let response = parse_request(request.as_bytes());
    assert!(response == RESPONSE_OK.to_vec())
}
#[test]
fn request_echo() {
    let request = "GET /echo/abc HTTP/1.1\r\n\r\n";
    let response = parse_request(request.as_bytes());
    let expected = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 3\r\n\r\nabc\r\n";
    assert!(response == expected.as_bytes().to_vec(), "Got: {:?}", String::from_utf8(response).unwrap());
}