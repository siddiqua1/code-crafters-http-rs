#[cfg(test)]
mod tests {
    use http_routing_rust::http::*;

    #[test]
    fn request_home() {
        let path = "/";
        let request = format!("GET {} HTTP/1.1\r\n\r\n", path);
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.method, Method::Get);
        assert_eq!(parsed.path, path);
        assert_eq!(parsed.version, Version::Http1_1);
        assert!(parsed.headers.is_empty());
        assert_eq!(parsed.body, None);
    }
    #[test]
    fn request_err_missing_one_escapes() {
        let path = "/";
        let request = format!("GET {} HTTP/1.1\r\n", path);
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }
    #[test]
    fn request_err_missing_both_escapes() {
        let path = "/";
        let request = format!("GET {} HTTP/1.1", path);
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_path_missing_slash() {
        let path = "home";
        let request = format!("GET {} HTTP/1.1", path);
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn path_err_empty() {
        let path = "";
        let parsed = Path::try_from(path);
        assert!(parsed.is_err());
    }

    #[test]
    fn request_complex_path() {
        let path = "/ajhkgdslf/aiuay783924/h78^&*%2345";
        let request = format!("GET {} HTTP/1.1\r\n\r\n", path);
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.method, Method::Get);
        assert_eq!(parsed.path, path);
        assert_eq!(parsed.version, Version::Http1_1);
        assert!(parsed.headers.is_empty());
        assert_eq!(parsed.body, None);
    }

    #[test]
    fn request_post() {
        let path = "/ajhkgdslf/aiuay783924/h78^&*%2345";
        let request = format!("POST {} HTTP/1.1\r\n\r\n", path);
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.method, Method::Post);
        assert_eq!(parsed.path, path);
        assert_eq!(parsed.version, Version::Http1_1);
        assert!(parsed.headers.is_empty());
        assert_eq!(parsed.body, None);
    }

    #[test]
    fn request_user_agent() {
        let path = "/user-agent";
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\n\r\n",
            path
        );
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.method, Method::Get);
        assert_eq!(parsed.path, path);
        assert_eq!(parsed.version, Version::Http1_1);
        assert!(!parsed.headers.is_empty());
        assert_eq!(parsed.body, None);

        assert_eq!(parsed.headers.get("Host"), Some(&"localhost:4221"));
        assert_eq!(parsed.headers.get("User-Agent"), Some(&"curl/7.64.1"));
    }

    #[test]
    fn request_user_agent_leading_and_trailing_space() {
        let path = "/user-agent";
        let request = format!(
            "GET {} HTTP/1.1\r\nHost:localhost:4221\r\nUser-Agent: curl/7.64.1 \r\n\r\n",
            path
        );
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.method, Method::Get);
        assert_eq!(parsed.path, path);
        assert_eq!(parsed.version, Version::Http1_1);
        assert!(!parsed.headers.is_empty());
        assert_eq!(parsed.body, None);

        assert_eq!(parsed.headers.get("Host"), Some(&"localhost:4221"));
        assert_eq!(parsed.headers.get("User-Agent"), Some(&"curl/7.64.1"));
    }
    #[test]
    fn request_err_header_no_key() {
        let path = "/user-agent";
        let request = format!(
            "GET {} HTTP/1.1\r\n:localhost:4221\r\nUser-Agent: curl/7.64.1 \r\n\r\n",
            path
        );
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }
    #[test]
    fn request_err_header_key_has_whitespace() {
        let path = "/user-agent";
        let request = format!(
            "GET {} HTTP/1.1\r\nHost :localhost:4221\r\nUser-Agent: curl/7.64.1 \r\n\r\n",
            path
        );
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_header_no_value() {
        let path = "/user-agent";
        let request = format!(
            "GET {} HTTP/1.1\r\nHost:\r\nUser-Agent: curl/7.64.1 \r\n\r\n",
            path
        );
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());

        let request = format!(
            "GET {} HTTP/1.1\r\nHost: \r\nUser-Agent: curl/7.64.1 \r\n\r\n",
            path
        );
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());

        //leading and trailing whitespace => key is empty string
        let request = format!(
            "GET {} HTTP/1.1\r\nHost:  \r\nUser-Agent: curl/7.64.1 \r\n\r\n",
            path
        );
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_ok());
    }

    #[test]
    fn request_with_body() {
        let path = "/files/potato";
        let data_write = "garbage data to write";
        let request = format!(
            "POST {} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
            path,
            data_write.len(),
            data_write
        );
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.method, Method::Post);
        assert_eq!(parsed.path, path);
        assert_eq!(parsed.version, Version::Http1_1);
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
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_invalid_status() {
        let request = "This is a valid HTTP request I swear.";
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_invalid_version() {
        let path = "/ajhkgdslf/aiuay783924/h78^&*%2345";
        let request = format!("GET {} HTTP/9001\r\n\r\n", path);
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_invalid_header_no_split() {
        let path = "/host";
        let request = format!("GET {} HTTP/1.1\r\nHost localhost 4221\r\n\r\n", path);
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_expected_empty_line_found_header() {
        let path = "/host";
        let request = format!("GET {} HTTP/1.1\r\nHost: localhost: 4221\r\n", path);
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_content_len_bad() {
        let path = "/files/potato";
        let data_write = "garbage data to write";
        let request = format!(
            "POST {} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
            path, "8912.123", data_write
        );
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_content_len_no_parse() {
        let path = "/files/potato";
        let data_write = "garbage data to write";
        let request = format!(
            "POST {} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
            path, "this_is_not_parsable_as_usize", data_write
        );
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_content_len_bigger_than_usize() {
        const BIGGER_THAN_USIZE: &str = "184467440737095516150";

        let path = "/files/potato";
        let data_write = "garbage data to write";
        let request = format!(
            "POST {} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
            path, BIGGER_THAN_USIZE, data_write
        );
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_content_len_bigger_than_body() {
        let path = "/files/potato";
        let data_write = "garbage data to write";
        let request = format!(
            "POST {} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
            path,
            data_write.len() + 1,
            data_write
        );
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_content_len_no_content() {
        let path = "/files/potato";
        let data_write = "garbage data to write";
        let request = format!(
            "POST {} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
            path,
            data_write.len(),
            ""
        );
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_content_len_no_body() {
        let path = "/files/potato";
        let data_write = "garbage data to write";
        let request = format!(
            "POST {} HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            path,
            data_write.len()
        );
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_body_no_header() {
        let path = "/files/potato";
        let data_write = "garbage data to write";
        let request = format!("POST {} HTTP/1.1\r\n\r\n{}\r\n", path, data_write);
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_body_no_header() {
        let path = "/files/potato";
        let data_write = "garbage data to write";
        let request = format!("POST {} HTTP/1.1\r\n\r\n{}", path, data_write);
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert!(parsed.body.is_none());
    }

    #[test]
    fn request_err_body_with_no_empty_line() {
        let path = "/files/potato";
        let data_write = "garbage data to write";
        let request = format!("POST {} HTTP/1.1\r\n{}", path, data_write);
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_body_no_header_no_space_empty_line_no_ending_rn() {
        let path = "/files/potato";
        let data_write = "garbage data to write";
        let request = format!("POST {} HTTP/1.1\r\n{}", path, data_write);
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_err_invalid_path_with_space() {
        let path = "super awesome path";
        let request = format!("GET {} HTTP/1.1\r\n\r\n", path);
        let parsed = Request::try_from(request.as_bytes());
        assert!(parsed.is_err());
    }

    #[test]
    fn request_buffer_invalid_utf8() {
        // from https://stackoverflow.com/questions/1301402/example-invalid-utf8-string
        const OCTET_2: &[u8; 2] = b"\xc3\x28";
        const OCTET_3: &[u8; 3] = b"\xe2\x28\xa1";
        const OCTET_4: &[u8; 4] = b"\xe2\x28\xa1\xbc";
        const OCTET_5: &[u8; 5] = b"\xf8\xa1\xa1\xa1\xa1"; //not unicode

        assert!(Request::try_from(&OCTET_2[..]).is_err());
        assert!(Request::try_from(&OCTET_3[..]).is_err());
        assert!(Request::try_from(&OCTET_4[..]).is_err());
        assert!(Request::try_from(&OCTET_5[..]).is_err());
    }
}
