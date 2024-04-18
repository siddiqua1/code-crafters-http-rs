use anyhow::anyhow;
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
/***
 * RFC: https://datatracker.ietf.org/doc/html/rfc9112
 */

#[derive(PartialEq, Debug)]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(PartialEq, Debug)]

pub enum Version {
    Http1_1,
}

// Request only lives as long as the TCP buffer so we tie the lifetime of the Request to that buffer
// Request is passive, should be ok to make the fields public
pub struct Request<'a> {
    pub method: HttpMethod,
    pub path: &'a str,
    pub _version: Version,
    pub headers: HashMap<&'a str, &'a str>,
    pub body: Option<&'a str>,
}

impl<'a> Request<'a> {
    pub fn from(read_buffer: &'a [u8]) -> Result<Request<'a>> {
        let Ok(request) = std::str::from_utf8(read_buffer) else {
            return Err(anyhow!("Invalid UTF-8 encoding"));
        };

        let lines = request.split("\r\n").collect::<Vec<&str>>();

        const MINIMUM_EXPECTED_LINES: usize = 2;
        if lines.len() < MINIMUM_EXPECTED_LINES {
            return Err(anyhow!("Message lines less than expected"));
        }

        let status_line = &lines[0].split(' ').collect_vec();
        let version = match status_line[2] {
            "HTTP/1.1" => Version::Http1_1,
            _ => return Err(anyhow!("Only HTTP/1.1 is supported")),
        };

        let method = match status_line[0] {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            _ => {
                return Err(anyhow!(
                    "Invalid HTTP verb, only Get and Post are supported at this time"
                ))
            }
        };

        let path = status_line[1];

        // from the spec, an empty line seperates the headers and status line from the body
        let mut empty_line = None;

        const STARTING_BODY_SEARCH_IDX: usize = 1;
        for (i, line) in lines[STARTING_BODY_SEARCH_IDX..].iter().enumerate() {
            if line.is_empty() {
                empty_line = Some(i + 1); // since we remove the status line
                break;
            }
        }

        let mut headers = HashMap::new();
        let mut body = None;

        let Some(empty_line) = empty_line else {
            return Ok(Request {
                method,
                path,
                _version: version,
                headers,
                body,
            });
        };

        let request_headers = &lines[1..empty_line];

        // this could probably better but not sure how to break out of map closure

        for header in request_headers {
            let key_val = header.split(": ").collect_vec();
            if key_val.len() != 2 {
                return Err(anyhow!("Ill-formatted headers"));
            }
            headers.insert(key_val[0], key_val[1]);
        }

        //not supporting case-insensitive strings at the moment as that would require ownership
        if let Some(n) = headers.get("Content-Length") {
            let n = n.parse::<usize>();
            let Ok(n) = n else {
                return Err(anyhow!("Unable to parse Content-Length header"));
            };
            let body_idx = empty_line + 1;
            if lines.len() <= body_idx {
                return Err(anyhow!(
                    "Content-Length specified, but no body was provided."
                ));
            }
            if n > lines[body_idx].len() {
                return Err(anyhow!("Content-Length specified is larger than body"));
            }
            body = Some(&lines[body_idx][0..n]);
        }

        return Ok(Request {
            method,
            path,
            _version: version,
            headers,
            body,
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::core::request::{HttpMethod, Request, Version};

    #[test]
    fn request_home() {
        let path = "/";
        let request = format!("GET {} HTTP/1.1\r\n\r\n", path);
        let parsed = Request::from(request.as_bytes());
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.method, HttpMethod::Get);
        assert_eq!(parsed.path, path);
        assert_eq!(parsed._version, Version::Http1_1);
        assert!(parsed.headers.is_empty());
        assert_eq!(parsed.body, None);
    }

    #[test]
    fn request_err_missing_escapes() {
        let path = "/";
        let request = format!("GET {} HTTP/1.1", path);
        let parsed = Request::from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_complex_path() {
        let path = "/ajhkgdslf/aiuay783924/h78^&*%2345";
        let request = format!("GET {} HTTP/1.1\r\n\r\n", path);
        let parsed = Request::from(request.as_bytes());
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.method, HttpMethod::Get);
        assert_eq!(parsed.path, path);
        assert_eq!(parsed._version, Version::Http1_1);
        assert!(parsed.headers.is_empty());
        assert_eq!(parsed.body, None);
    }

    #[test]
    fn request_post() {
        let path = "/ajhkgdslf/aiuay783924/h78^&*%2345";
        let request = format!("POST {} HTTP/1.1\r\n\r\n", path);
        let parsed = Request::from(request.as_bytes());
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.method, HttpMethod::Post);
        assert_eq!(parsed.path, path);
        assert_eq!(parsed._version, Version::Http1_1);
        assert!(parsed.headers.is_empty());
        assert_eq!(parsed.body, None);
    }

    #[test]
    fn request_user_agent() {
        let path = "/user-agent";
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\n",
            path
        );
        let parsed = Request::from(request.as_bytes());
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.method, HttpMethod::Get);
        assert_eq!(parsed.path, path);
        assert_eq!(parsed._version, Version::Http1_1);
        assert!(!parsed.headers.is_empty());
        assert_eq!(parsed.body, None);

        assert_eq!(parsed.headers.get("Host"), Some(&"localhost:4221"));
        assert_eq!(parsed.headers.get("User-Agent"), Some(&"curl/7.64.1"));
    }

    #[test]
    fn request_with_body() {
        let path = "/files/potato";
        let data_write = "garbage data to write";
        let request = format!(
            "POST {} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}\r\n",
            path,
            data_write.len(),
            data_write
        );
        let parsed = Request::from(request.as_bytes());
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.method, HttpMethod::Post);
        assert_eq!(parsed.path, path);
        assert_eq!(parsed._version, Version::Http1_1);
        assert!(!parsed.headers.is_empty());
        assert_eq!(parsed.body, Some(data_write));
        let len_str = data_write.len().to_string();
        assert_eq!(
            parsed.headers.get("Content-Length"),
            Some(&len_str.as_str())
        );
    }

    #[test]
    fn request_err_invalid_verb() {
        let path = "/ajhkgdslf/aiuay783924/h78^&*%2345";
        let request = format!("SUPER_VALID_METHOD {} HTTP/1.1\r\n\r\n", path);
        let parsed = Request::from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_invalid_status() {
        let request = "This is a valid HTTP request I swear.";
        let parsed = Request::from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_invalid_version() {
        let path = "/ajhkgdslf/aiuay783924/h78^&*%2345";
        let request = format!("GET {} HTTP/9001\r\n\r\n", path);
        let parsed = Request::from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_invalid_header_no_split() {
        let path = "/host";
        let request = format!("GET {} HTTP/1.1\r\nHost localhost:4221\r\n", path);
        let parsed = Request::from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_invalid_header_multi_split() {
        let path = "/host";
        let request = format!("GET {} HTTP/1.1\r\nHost: localhost: 4221\r\n", path);
        let parsed = Request::from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_content_len_bad() {
        let path = "/files/potato";
        let data_write = "garbage data to write";
        let request = format!(
            "POST {} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}\r\n",
            path, "8912.123", data_write
        );
        let parsed = Request::from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_content_len_no_content() {
        let path = "/files/potato";
        let data_write = "garbage data to write";
        let request = format!(
            "POST {} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}\r\n",
            path,
            data_write.len(),
            ""
        );
        let parsed = Request::from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_content_len_no_body() {
        let path = "/files/potato";
        let data_write = "garbage data to write";
        let request = format!(
            "POST {} HTTP/1.1\r\nContent-Length: {}\r\n",
            path,
            data_write.len()
        );
        let parsed = Request::from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_invalid_path_with_space() {
        let path = "super awesome path";
        let request = format!("GET {} HTTP/1.1\r\n\r\n", path);
        let parsed = Request::from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_buffer_invalid_utf8() {
        // from https://stackoverflow.com/questions/1301402/example-invalid-utf8-string
        const OCTET_2: &[u8; 2] = b"\xc3\x28";
        const OCTET_3: &[u8; 3] = b"\xe2\x28\xa1";
        const OCTET_4: &[u8; 4] = b"\xe2\x28\xa1\xbc";
        const OCTET_5: &[u8; 5] = b"\xf8\xa1\xa1\xa1\xa1"; //not unicode

        assert!(Request::from(OCTET_2).is_err());
        assert!(Request::from(OCTET_3).is_err());
        assert!(Request::from(OCTET_4).is_err());
        assert!(Request::from(OCTET_5).is_err());
    }
}
