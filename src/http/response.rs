use super::headers::HttpHeaders;

#[derive(Debug)]
pub struct HttpResponse {
    pub status: usize,
    pub headers: HttpHeaders,
    pub body: String,
}

impl HttpResponse {
    pub fn new() -> HttpResponse {
        HttpResponse {
            status: 200,
            headers: HttpHeaders::new(),
            body: String::new(),
        }
    }
}
