pub struct InvalidRequest;
pub const RESPONSE_OK: &[u8; 19] = b"HTTP/1.1 200 OK\r\n\r\n";
pub const RESPONSE_CREATED: &[u8; 24] = b"HTTP/1.1 201 CREATED\r\n\r\n";

pub enum HttpMethod {
    Get,
    Post,
}
pub enum HttpPath<'a> {
    Root,
    Node(&'a str),
}
pub enum Version {
    Http1_1,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct HeaderKey<'a>(&'a str);
pub struct HeaderValue<'a>(&'a str);
pub struct RequestHeaders<'a> {
    pairs: Vec<(HeaderKey<'a>, HeaderValue<'a>)>,
}

pub struct Request<'a> {
    pub method: HttpMethod,
    pub path: HttpPath<'a>,
    pub _version: Version,
    pub headers: RequestHeaders<'a>,
    pub body: Option<String>,
}

use crate::core::context::ServerContext;

impl<'a> Request<'a> {
    pub fn from(read_buffer: &'a [u8]) -> Result<Request<'a>, InvalidRequest> {
        let request = match std::str::from_utf8(read_buffer) {
            Err(_) => return Err(InvalidRequest),
            Ok(s) => s,
        };
        let words = request
            .split("\r\n")
            .map(|line| return line.split(' ').collect::<Vec<&str>>())
            .collect::<Vec<Vec<&str>>>();

        if words.len() < 2 {
            return Err(InvalidRequest);
        }

        let status_line = &words[0];
        let version = match status_line[2] {
            "HTTP/1.1" => Version::Http1_1,
            _ => return Err(InvalidRequest),
        };

        let method = match status_line[0] {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            _ => return Err(InvalidRequest),
        };

        let path = match status_line[1] {
            "/" => HttpPath::Root,
            _s => HttpPath::Node(_s),
        };

        let req_headers = &words[1..];
        // this could probably better but not sure how to break out of map closure
        let mut headers: Vec<(HeaderKey, HeaderValue)> = Vec::new();
        let mut body = None;
        let mut end_of_headers = false;

        for line in req_headers {
            if line.len() == 2 {
                headers.push((HeaderKey(line[0]), HeaderValue(line[1])));
            } else if line.len() == 1 {
                end_of_headers = true;
            } else if end_of_headers {
                body = Some(line.join(" "));
            }
        }
        let headers = RequestHeaders { pairs: headers };

        return Ok(Request {
            method,
            path,
            _version: version,
            headers,
            body,
        });
    }

    pub fn handle_request(&self, context: &ServerContext) -> Result<Vec<u8>, InvalidRequest> {
        match self.method {
            // can probably use type state pattern here instead but lazy atm
            HttpMethod::Get => return self.handle_request_get(context),
            HttpMethod::Post => return self.handle_request_post(context), // _ => return Err(InvalidRequest),
        }
    }

    fn handle_request_get(&self, context: &ServerContext) -> Result<Vec<u8>, InvalidRequest> {
        match self.path {
            HttpPath::Root => return Ok(RESPONSE_OK.to_vec()),
            HttpPath::Node(s) => {
                // this is def bad but most ergonomic solution i can think would involve proc macro magic which i have a skil issue with
                if let Ok(v) = try_echo(s) {
                    return Ok(v);
                }
                if let Ok(v) = try_user_agent(s, &self.headers) {
                    return Ok(v);
                }
                if let Ok(v) = try_get_file(s, context) {
                    return Ok(v);
                }

                return Err(InvalidRequest);
            }
        }
    }

    fn handle_request_post(&self, context: &ServerContext) -> Result<Vec<u8>, InvalidRequest> {
        match self.path {
            HttpPath::Root => return Err(InvalidRequest),
            HttpPath::Node(s) => {
                if let Ok(v) = try_post_file(s, context, &self.body, &self.headers) {
                    return Ok(v);
                }

                return Err(InvalidRequest);
            }
        }
    }
}

struct RequestMismatch;

fn try_echo(s: &str) -> Result<Vec<u8>, RequestMismatch> {
    if "/echo/" != &s[0..6] {
        return Err(RequestMismatch);
    }
    let random_str = &s[6..];
    let response = format!(
        "{}\r\n{}\r\nContent-Length: {}\r\n\r\n{}\r\n",
        "HTTP/1.1 200 OK",
        "Content-Type: text/plain",
        random_str.len(),
        random_str
    );
    return Ok(response.into_bytes());
}

fn try_user_agent(path: &str, headers: &RequestHeaders) -> Result<Vec<u8>, RequestMismatch> {
    if path != "/user-agent" {
        return Err(RequestMismatch);
    }
    let mut agent = None;

    for (k, v) in &headers.pairs {
        if k.0.to_lowercase() == "user-agent:" {
            agent = Some(v.0);
            break;
        }
    }
    let agent = match agent {
        None => return Err(RequestMismatch),
        Some(a) => a,
    };
    let response = format!(
        "{}\r\n{}\r\nContent-Length: {}\r\n\r\n{}\r\n",
        "HTTP/1.1 200 OK",
        "Content-Type: text/plain",
        agent.len(),
        agent
    );
    return Ok(response.into_bytes());
}

fn try_get_file(path: &str, context: &ServerContext) -> Result<Vec<u8>, RequestMismatch> {
    if &path[0..7] != "/files/" {
        return Err(RequestMismatch);
    }
    let file_name = &path[7..];
    match context.file_handler.search(file_name) {
        None => return Err(RequestMismatch),
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

fn try_post_file(
    path: &str,
    context: &ServerContext,
    body: &Option<String>,
    headers: &RequestHeaders,
) -> Result<Vec<u8>, RequestMismatch> {
    if &path[0..7] != "/files/" {
        return Err(RequestMismatch);
    }

    let body = match body {
        None => return Err(RequestMismatch),
        Some(b) => b,
    };
    const DESIRED_KEY: HeaderKey<'_> = HeaderKey("Content-Length:");

    let mut body_len = None;
    for (k, v) in &headers.pairs {
        if k == &DESIRED_KEY {
            body_len = Some(v.0.parse::<usize>())
        }
    }
    let body_len = match body_len {
        None => return Err(RequestMismatch),
        Some(u) => match u {
            Err(_) => return Err(RequestMismatch),
            Ok(l) => l,
        },
    };
    if body.len() < body_len {
        return Err(RequestMismatch);
    }

    let file_name = &path[7..];
    let path = context.file_handler.get_path(file_name);
    let _written = match context
        .file_handler
        .write(path, &body.as_bytes()[0..body_len])
    {
        Err(_) => return Err(RequestMismatch),
        Ok(b) => b,
    };
    return Ok(RESPONSE_CREATED.to_vec());
}
