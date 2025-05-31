use super::headers::{HttpHeaderName, HttpHeaders};

#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: usize,
    pub headers: HttpHeaders,
    pub body: String,
}

impl HttpResponse {
    pub fn new() -> HttpResponse {
        HttpResponse {
            status_code: 200,
            headers: HttpHeaders::new(),
            body: String::new(),
        }
    }

    pub fn add_header(&mut self, header: HttpHeaderName, value: &str) {
        self.headers.add(header, value);
    }

    pub fn content_length(&self) -> usize {
        self.body.len()
    }
}
