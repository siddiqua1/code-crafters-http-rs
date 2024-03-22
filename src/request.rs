use anyhow::Error;

pub struct InvalidRequest;
const RESPONSE_OK: &[u8; 19] = b"HTTP/1.1 200 OK\r\n\r\n";
pub enum HttpMethod {
    Get,
}
pub enum HttpPath<'a> {
    Root,
    Node(&'a str),
}
pub enum Version {
    Http1_1,
}
pub struct HeaderKey<'a>(&'a str);
pub struct HeaderValue<'a>(&'a str);
pub struct RequestHeaders<'a> {
    pairs: Vec<(HeaderKey<'a>, HeaderValue<'a>)>,
}

pub struct Request<'a> {
    method: HttpMethod,
    path: HttpPath<'a>,
    version: Version,
    headers: RequestHeaders<'a>,
}

impl<'a> Request<'a> {
    pub fn from(read_buffer: &'a [u8]) -> Result<Request<'a>, InvalidRequest> {
        let request = match std::str::from_utf8(read_buffer) {
            Err(_) => return Err(InvalidRequest),
            Ok(s) => s,
        };
        let words = request
            .split("\r\n")
            .map(|line| line.split(" ").collect::<Vec<&str>>())
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
            _ => return Err(InvalidRequest),
        };

        let path = match status_line[1] {
            "/" => HttpPath::Root,
            _s => HttpPath::Node(_s),
        };

        let req_headers = &words[1..];
        // this could probably better but not sure how to break out of map closure
        let mut headers: Vec<(HeaderKey, HeaderValue)> = Vec::new();

        for line in req_headers {
            if line.len() != 2 {
                break;
            }
            headers.push((HeaderKey(line[0]), HeaderValue(line[1])));
        }
        let headers = RequestHeaders { pairs: headers };

        return Ok(Request {
            method,
            path,
            version,
            headers,
        });
    }

    pub fn handle_request(&self) -> Result<Vec<u8>, InvalidRequest> {
        match self.method {
            // can probably use type state pattern here instead but lazy atm
            HttpMethod::Get => return self.handle_request_get(),
            _ => return Err(InvalidRequest),
        }
    }

    fn handle_request_get(&self) -> Result<Vec<u8>, InvalidRequest> {
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
                todo!()
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
    if path != "/user-agent" { return Err(RequestMismatch); }
    let mut agent = None;

    for (k, v) in &headers.pairs {
        if k.0.to_lowercase() == "user-agent:" {
            agent = Some(v.0);
            break;
        }
    }
    let agent = match agent {
        None => return Err(RequestMismatch),
        Some(a) => a
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