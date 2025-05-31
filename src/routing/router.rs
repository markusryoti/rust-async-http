use std::{collections::HashMap, pin::Pin};
use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf};

use crate::http::{request::HttpRequest, response::HttpResponse};

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
        request: &HttpRequest,
    ) -> Result<(), std::io::Error> {
        let route = self.routes.get(&request.path);

        let mut res = HttpResponse::new();

        if let Some(r) = route {
            r(request, &mut res).await;
        }

        let response = format!(
            "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
            res.status,
            res.body.len(),
            res.body
        );

        writer.write_all(response.as_bytes()).await.unwrap();
        writer.shutdown().await?;

        Ok(())
    }
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
macro_rules! wrap_async_handler {
    ($func:path) => {{ $crate::async_handler!(|req, res| { $func(req, res).await }) }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_works() {
        let mut router = Router::new();

        async fn _handler(_req: &HttpRequest, res: &mut HttpResponse) {
            res.status = 200;
            res.body = String::from("hello");
        }

        router.add_route("/", wrap_async_handler!(_handler));
    }

    #[test]
    fn closure_works() {
        let mut router = Router::new();

        let handler: HandlerFn = async_handler!(|_req, res| {
            res.status = 200;
            res.body = String::from("hello");
        });

        router.add_route("/", handler);
    }
}
