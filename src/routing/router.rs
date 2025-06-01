use log::info;
use std::{collections::HashMap, net::SocketAddr, pin::Pin};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::OwnedWriteHalf,
};

use crate::http::{headers::HttpHeaderName, request::HttpRequest, response::HttpResponse};

pub struct Router {
    routes: HashMap<String, HandlerFn>,
}

impl Router {
    pub fn new() -> Router {
        Router {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, path: &str, handler: HandlerFn) {
        self.routes.insert(path.to_string(), handler);
    }

    pub async fn match_route(
        &self,
        mut writer: OwnedWriteHalf,
        addr: SocketAddr,
        request: &HttpRequest,
    ) -> Result<(), std::io::Error> {
        info!("Handling request to path: {}", request.path);

        let route = self.routes.get(&request.path);

        let mut res = HttpResponse::new();

        if let Some(r) = route {
            r(request, &mut res).await;
        } else {
            not_found(&mut res).await;
        }

        // TODO:
        // Make sure all "needed" headers are included
        res.add_header(
            HttpHeaderName::ContentLength,
            &res.content_length().to_string(),
        );
        res.add_header(HttpHeaderName::Connection, "close");

        let response = format!(
            "HTTP/1.1 {}\r\n{}\r\n\r\n{}",
            res.status_code,
            res.headers.as_str(),
            res.body
        );

        info!(
            "Sending response to peer: {} with status: {}, Content-Length: {}, Content-Type: {}",
            addr,
            res.status_code,
            res.content_length(),
            res.headers.content_type().unwrap_or("text/html")
        );

        writer.write_all(response.as_bytes()).await?;
        writer.flush().await?;

        info!("Response sent to peer: {}", addr);

        Ok(())
    }
}

async fn not_found(res: &mut HttpResponse) {
    let mut f = File::open("public/404.html").await.unwrap();
    let mut buffer = String::new();

    f.read_to_string(&mut buffer).await.unwrap();

    res.body = buffer;
    res.status_code = 404;
}

pub type HandlerFn = Box<
    dyn for<'a> Fn(
            &'a HttpRequest,
            &'a mut HttpResponse,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>
        + Send
        + Sync,
>;

#[macro_export]
macro_rules! async_handler {
    (|$req:ident, $res:ident| $body:block) => {{
        use std::{future::Future, pin::Pin};

        fn handler<'a>(
            $req: &'a $crate::http::request::HttpRequest,
            $res: &'a mut $crate::http::response::HttpResponse,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
            Box::pin(async move $body)
        }

        Box::new(handler) as $crate::routing::router::HandlerFn
    }};
}

#[macro_export]
macro_rules! async_fn_handler {
    ($func:path) => {{ $crate::async_handler!(|req, res| { $func(req, res).await }) }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_works() {
        let mut router = Router::new();

        async fn _handler(_req: &HttpRequest, res: &mut HttpResponse) {
            res.status_code = 200;
            res.body = String::from("hello");
        }

        router.add_route("/", async_fn_handler!(_handler));
    }

    #[test]
    fn closure_works() {
        let mut router = Router::new();

        let handler: HandlerFn = async_handler!(|_req, res| {
            res.status_code = 200;
            res.body = String::from("hello");
        });

        router.add_route("/", handler);
    }
}
