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
}
