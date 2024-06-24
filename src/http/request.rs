//! Based on RFC: https://datatracker.ietf.org/doc/html/rfc9112

use crate::prelude::*;
use itertools::Itertools;
use std::collections::HashMap;

// Request only lives as long as the TCP buffer so we tie the lifetime of the Request to that buffer
// Request is passive, should be ok to make the fields public
#[derive(Debug, PartialEq, Clone)]
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
        /*
        From the RFC the message format is the following
        start-line CRLF
        *( field-line CRLF )
        CRLF
        [ message-body ]

        which translates to
        STATUS \r\n
        *(HEADER: KEY \r\n)
        \r\n
        BODY?
        */

        let lines = request.split("\r\n").collect::<Vec<&str>>();

        const MINIMUM_CRLF: usize = 2;
        const MINIMUM_LINES: usize = MINIMUM_CRLF + 1;
        if lines.len() < MINIMUM_LINES {
            return Err(Error::HttpMalformedRequest);
        }

        let status_line = &lines[0].split(' ').collect_vec();
        if status_line.len() != 3 {
            return Err(Error::HttpMalformedRequest);
        }
        let method = Method::try_from(status_line[0])?;
        let path = Path::try_from(status_line[1])?;
        let version = Version::try_from(status_line[2])?;

        // there must a sequence \r\n\r\n from the specification
        // that is the second to last line
        let empty_line = lines.len() - 2;
        if !lines[empty_line].is_empty() {
            return Err(Error::HttpMalformedRequest);
        }

        if lines.len() == MINIMUM_LINES {
            return Ok(Request {
                method,
                path,
                version,
                headers: Headers::default(),
                // even if a body was supplied, because no header are present we don't parse them
                body: None,
            });
        }

        let mut body = None;

        let request_headers = &lines[1..empty_line];
        let headers = Headers::try_from(request_headers)?;

        //not supporting case-insensitive strings at the moment as that would require ownership
        if let Some(n) = headers.get("Content-Length") {
            let Ok(n) = n.parse::<usize>() else {
                return Err(Error::Generic(
                    "Unable to parse Content-Length header".to_string(),
                ));
            };
            let body_idx = empty_line + 1; //always in bound
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
