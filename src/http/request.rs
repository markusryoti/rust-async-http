use std::str::FromStr;

use tokio::{
    io::{self, AsyncBufReadExt, AsyncReadExt, BufReader},
    net::tcp::OwnedReadHalf,
};

use super::{
    headers::{HttpHeaderName, HttpHeaders},
    method::HttpMethod,
};

#[derive(Debug)]
pub struct HttpFirstRow {
    pub method: String,
    pub protocol: String,
    pub resource: String,
}

impl FromStr for HttpFirstRow {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.trim().split_whitespace().collect();

        if parts.len() != 3 {
            return Err("Invalid request line format");
        }

        Ok(HttpFirstRow {
            method: parts[0].to_string(),
            resource: parts[1].to_string(),
            protocol: parts[2].to_string(),
        })
    }
}

pub async fn get_body(
    buffered_reader: &mut BufReader<OwnedReadHalf>,
    content_length: usize,
) -> Result<Vec<u8>, io::Error> {
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        buffered_reader.read_exact(&mut body).await?;
    }

    Ok(body)
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub http_version: String, // e.g., "HTTP/1.1"
    pub headers: HttpHeaders,
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    pub async fn parse(
        buffered_reader: &mut BufReader<OwnedReadHalf>,
    ) -> Result<HttpRequest, io::Error> {
        let mut headers = HttpHeaders::new();

        let mut line = String::new();

        let bytes_read = buffered_reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, ""));
        }

        let first_row = line.parse::<HttpFirstRow>().unwrap();
        let path = first_row.resource;
        let http_version = first_row.protocol;
        let method = HttpMethod::from(first_row.method.as_str());

        loop {
            line.clear();

            let bytes_read = buffered_reader.read_line(&mut line).await?;
            if bytes_read == 0 || line == "\r\n" {
                break;
            }

            let mut parts = line.splitn(2, ':');
            let key = parts.next().unwrap().trim();
            let value = parts.next().unwrap().trim();

            let header = HttpHeaderName::from(key);

            headers.add(header, value);
        }

        let content_length = headers.content_length().unwrap_or(0);
        let body = get_body(buffered_reader, content_length).await?;

        Ok(HttpRequest {
            method: method,
            path: path,
            http_version: http_version,
            headers: headers,
            body: Some(body),
        })
    }
}
