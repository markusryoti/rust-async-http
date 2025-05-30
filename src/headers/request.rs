use std::{collections::HashMap, str::FromStr};

use tokio::{
    io::{self, AsyncBufReadExt, AsyncReadExt, BufReader},
    net::tcp::OwnedReadHalf,
};

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

            headers.add(key, value);
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

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
    Custom(String),
}

impl From<&str> for HttpMethod {
    fn from(s: &str) -> Self {
        match s.to_ascii_uppercase().as_str() {
            "GET" => HttpMethod::GET,
            "HEAD" => HttpMethod::HEAD,
            // ...
            _ => HttpMethod::Custom(s.to_string()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum HttpHeaderName {
    // ... (All your HttpHeaderName variants)
    CacheControl,
    Connection,
    Date,
    Pragma,
    Trailer,
    TransferEncoding,
    Upgrade,
    Via,
    Warning,
    Accept,
    AcceptCharset,
    AcceptEncoding,
    AcceptLanguage,
    Authorization,
    Cookie,
    ContentLength,
    ContentType,
    Expect,
    From,
    Host,
    IfMatch,
    IfModifiedSince,
    IfNoneMatch,
    IfRange,
    IfUnmodifiedSince,
    MaxForwards,
    ProxyAuthorization,
    Range,
    Referer,
    TE,
    UserAgent,
    Age,
    Allow,
    ContentEncoding,
    ContentLanguage,
    ContentLocation,
    ContentMD5,
    ContentRange,
    Expires,
    LastModified,
    Location,
    ProxyAuthenticate,
    RetryAfter,
    Server,
    Vary,
    WWWAuthenticate,
    Custom(String),
}

impl From<&str> for HttpHeaderName {
    fn from(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "cache-control" => HttpHeaderName::CacheControl,
            "connection" => HttpHeaderName::Connection,
            "date" => HttpHeaderName::Date,
            "pragma" => HttpHeaderName::Pragma,
            "trailer" => HttpHeaderName::Trailer,
            "transfer-encoding" => HttpHeaderName::TransferEncoding,
            "upgrade" => HttpHeaderName::Upgrade,
            "via" => HttpHeaderName::Via,
            "warning" => HttpHeaderName::Warning,
            "accept" => HttpHeaderName::Accept,
            "accept-charset" => HttpHeaderName::AcceptCharset,
            "accept-encoding" => HttpHeaderName::AcceptEncoding,
            "accept-language" => HttpHeaderName::AcceptLanguage,
            "authorization" => HttpHeaderName::Authorization,
            "cookie" => HttpHeaderName::Cookie,
            "content-length" => HttpHeaderName::ContentLength,
            "content-type" => HttpHeaderName::ContentType,
            "expect" => HttpHeaderName::Expect,
            "from" => HttpHeaderName::From,
            "host" => HttpHeaderName::Host,
            "if-match" => HttpHeaderName::IfMatch,
            "if-modified-since" => HttpHeaderName::IfModifiedSince,
            "if-none-match" => HttpHeaderName::IfNoneMatch,
            "if-range" => HttpHeaderName::IfRange,
            "if-unmodified-since" => HttpHeaderName::IfUnmodifiedSince,
            "max-forwards" => HttpHeaderName::MaxForwards,
            "proxy-authorization" => HttpHeaderName::ProxyAuthorization,
            "range" => HttpHeaderName::Range,
            "referer" => HttpHeaderName::Referer,
            "te" => HttpHeaderName::TE,
            "user-agent" => HttpHeaderName::UserAgent,
            "age" => HttpHeaderName::Age,
            "allow" => HttpHeaderName::Allow,
            "content-encoding" => HttpHeaderName::ContentEncoding,
            "content-language" => HttpHeaderName::ContentLanguage,
            "content-location" => HttpHeaderName::ContentLocation,
            "content-md5" => HttpHeaderName::ContentMD5,
            "content-range" => HttpHeaderName::ContentRange,
            "expires" => HttpHeaderName::Expires,
            "last-modified" => HttpHeaderName::LastModified,
            "location" => HttpHeaderName::Location,
            "proxy-authenticate" => HttpHeaderName::ProxyAuthenticate,
            "retry-after" => HttpHeaderName::RetryAfter,
            "server" => HttpHeaderName::Server,
            "vary" => HttpHeaderName::Vary,
            "www-authenticate" => HttpHeaderName::WWWAuthenticate,
            _ => HttpHeaderName::Custom(s.to_string()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum HttpHeaderValue {
    // ... (All your HttpHeaderValue variants)
    ContentLength(usize),
    ContentType(String),
    Host(String, Option<u16>),
    Connection(ConnectionHeaderValue),
    Raw(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionHeaderValue {
    // ... (All your ConnectionHeaderValue variants)
    Close,
    KeepAlive,
    Upgrade,
    Custom(String),
}

impl HttpHeaderValue {
    pub fn parse(name: &HttpHeaderName, value: &str) -> Self {
        match name {
            HttpHeaderName::ContentLength => {
                HttpHeaderValue::ContentLength(value.parse::<usize>().unwrap_or(0))
            }
            HttpHeaderName::ContentType => HttpHeaderValue::ContentType(value.to_string()),
            HttpHeaderName::Host => {
                let parts: Vec<&str> = value.split(':').collect();
                let host = parts[0].to_string();
                let port = if parts.len() > 1 {
                    parts[1].parse::<u16>().ok()
                } else {
                    None
                };
                HttpHeaderValue::Host(host, port)
            }
            HttpHeaderName::Connection => match value.to_ascii_lowercase().as_str() {
                "close" => HttpHeaderValue::Connection(ConnectionHeaderValue::Close),
                "keep-alive" => HttpHeaderValue::Connection(ConnectionHeaderValue::KeepAlive),
                "upgrade" => HttpHeaderValue::Connection(ConnectionHeaderValue::Upgrade),
                _ => HttpHeaderValue::Connection(ConnectionHeaderValue::Custom(value.to_string())),
            },
            _ => HttpHeaderValue::Raw(value.to_string()),
        }
    }
}

#[derive(Debug, Default)]
pub struct HttpHeaders {
    pub headers: HashMap<HttpHeaderName, Vec<HttpHeaderValue>>,
}

impl HttpHeaders {
    pub fn new() -> Self {
        HttpHeaders {
            headers: HashMap::new(),
        }
    }

    pub fn add(&mut self, name_str: &str, value_str: &str) {
        let name: HttpHeaderName = name_str.into();
        let value = HttpHeaderValue::parse(&name, value_str);
        self.headers
            .entry(name)
            .or_insert_with(Vec::new)
            .push(value);
    }

    pub fn get(&self, name: &HttpHeaderName) -> Option<&Vec<HttpHeaderValue>> {
        self.headers.get(name)
    }

    pub fn get_one_raw(&self, name: &HttpHeaderName) -> Option<&str> {
        self.get(name)
            .and_then(|values| values.first())
            .and_then(|value| match value {
                HttpHeaderValue::Raw(s) => Some(s.as_str()),
                _ => None,
            })
    }

    pub fn content_length(&self) -> Option<usize> {
        self.get(&HttpHeaderName::ContentLength)
            .and_then(|values| values.first())
            .and_then(|value| {
                if let HttpHeaderValue::ContentLength(len) = value {
                    Some(*len)
                } else {
                    None
                }
            })
    }

    pub fn host(&self) -> Option<(&str, Option<u16>)> {
        self.get(&HttpHeaderName::Host)
            .and_then(|values| values.first())
            .and_then(|value| {
                if let HttpHeaderValue::Host(host, port) = value {
                    Some((host.as_str(), *port))
                } else {
                    None
                }
            })
    }
}
