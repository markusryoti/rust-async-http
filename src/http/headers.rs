use std::collections::HashMap;

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
