use std::{collections::HashMap, pin::Pin};

use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::tcp::OwnedWriteHalf,
};

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

async fn create_response(fname: &str, status: u16) -> Result<String, io::Error> {
    let mut f = File::open(fname).await?;
    let mut buffer = String::new();

    f.read_to_string(&mut buffer).await?;

    let length = buffer.len();

    let response = format!("HTTP/1.1 {status}\r\nContent-Length: {length}\r\n\r\n{buffer}");

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_works() {
        let mut router = Router::new();

        // let home_handler: HandlerFn = Box::new(|req, res| {
        //     Box::pin(async move {
        //         println!("Home handler received request for path: {}", req.path);
        //         res.status = 200;
        //     })
        // });

        let home_handler: HandlerFn = Box::new(|req, res| {
            Box::pin(async move {
                println!("Home handler received request for path: {}", req.path);
                res.status = 200;
            })
        });

        // async fn home_handler(req: &HttpRequest, res: &mut HttpResponse) {
        //     println!("Home handler received request for path: {}", req.path);
        //     res.status = 200;
        // }

        router.add_route("/", home_handler);
    }

    #[test]
    fn closure_works() {
        // let mut router = Router::new();

        // router.add_route("/about", |req, res| {
        //     Box::pin(async move {
        //         println!("About handler received request for path: {}", req.path);
        //         res.status = 200;
        //     })
        // });
    }
}
