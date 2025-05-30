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
