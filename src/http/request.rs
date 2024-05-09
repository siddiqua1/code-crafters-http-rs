//! Based on RFC: https://datatracker.ietf.org/doc/html/rfc9112

use crate::{http::headers, prelude::*};
use itertools::Itertools;
use std::collections::HashMap;

// Request only lives as long as the TCP buffer so we tie the lifetime of the Request to that buffer
// Request is passive, should be ok to make the fields public
pub struct Request<'a> {
    pub method: Method,
    pub path: Path<'a>,
    pub version: Version,
    pub headers: Headers<'a>,
    pub body: Option<&'a str>,
}

impl<'a> TryFrom<&'a [u8]> for Request<'a> {
    type Error = Error;
    fn try_from(read_buffer: &'a [u8]) -> Result<Self> {
        let Ok(request) = std::str::from_utf8(read_buffer) else {
            return Err(Error::HttpUnableToParse);
        };

        let lines = request.split("\r\n").collect::<Vec<&str>>();

        const MINIMUM_EXPECTED_LINES: usize = 2;
        if lines.len() < MINIMUM_EXPECTED_LINES {
            return Err(Error::HttpMalformedRequest);
        }

        let status_line = &lines[0].split(' ').collect_vec();
        if status_line.len() != 3 {
            return Err(Error::HttpMalformedRequest);
        }
        let method = Method::try_from(status_line[0])?;
        let path = Path::try_from(status_line[1])?;
        let version = Version::try_from(status_line[2])?;

        // from the spec, an empty line seperates the headers and status line from the body
        let mut empty_line = None;
        const STARTING_BODY_SEARCH_IDX: usize = 1;
        for (i, line) in lines[STARTING_BODY_SEARCH_IDX..].iter().enumerate() {
            if line.is_empty() {
                empty_line = Some(i + 1); // since we remove the status line
                break;
            }
        }

        let mut body = None;

        let Some(empty_line) = empty_line else {
            return Ok(Request {
                method,
                path,
                version,
                headers: Headers::default(),
                body,
            });
        };

        let request_headers = &lines[1..empty_line];
        let headers = Headers::try_from(request_headers)?;

        //not supporting case-insensitive strings at the moment as that would require ownership
        if let Some(n) = headers.get("Content-Length") {
            let n = n.parse::<usize>();
            let Ok(n) = n else {
                return Err(Error::Generic(
                    "Unable to parse Content-Length header".to_string(),
                ));
            };
            let body_idx = empty_line + 1;
            if lines.len() <= body_idx {
                return Err(Error::Generic(
                    "Content-Length specified, but no body was provided.".to_string(),
                ));
            }
            if n > lines[body_idx].len() {
                return Err(Error::Generic(
                    "Content-Length specified is larger than body".to_string(),
                ));
            }
            body = Some(&lines[body_idx][0..n]);
        }

        return Ok(Request {
            method,
            path,
            version,
            headers,
            body,
        });
    }
}
