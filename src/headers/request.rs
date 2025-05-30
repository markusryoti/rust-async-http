use std::str::FromStr;

use tokio::{
    io::{self, AsyncBufReadExt, AsyncReadExt, BufReader},
    net::tcp::OwnedReadHalf,
};

pub async fn get_headers(
    buffered_reader: &mut BufReader<OwnedReadHalf>,
) -> Result<Headers, io::Error> {
    let mut headers = Vec::new();
    let mut content_length = 0;
    let mut line = String::new();

    loop {
        line.clear();

        let bytes_read = buffered_reader.read_line(&mut line).await?;
        if bytes_read == 0 || line == "\r\n" {
            break;
        }

        if let Some(cl) = line.strip_prefix("Content-Length: ") {
            content_length = cl.trim().parse::<usize>().unwrap_or(0);
        }

        headers.push(line.trim_end().to_string());
    }

    Ok(Headers {
        values: headers,
        content_length: content_length,
    })
}

pub struct Headers {
    pub values: Vec<String>,
    pub content_length: usize,
}

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
