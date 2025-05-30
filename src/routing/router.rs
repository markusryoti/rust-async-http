use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::tcp::OwnedWriteHalf,
};

use crate::headers::request::HttpRequest;

pub struct Router {
    routes: Vec<Route>,
}

pub struct Route {
    path: String,
    resource: String,
}

impl Router {
    pub fn new() -> Router {
        Router { routes: vec![] }
    }

    pub fn add_route(&mut self, uri_path: &str, resource: &str) {
        self.routes.push(Route {
            path: String::from(uri_path),
            resource: String::from(resource),
        });
    }

    pub async fn match_route(
        &self,
        mut writer: OwnedWriteHalf,
        request: &HttpRequest,
    ) -> Result<(), std::io::Error> {
        let res = self.routes.iter().find(|x| x.path == request.path);

        let res = match res {
            Some(r) => (r.resource.clone(), 200),
            None => (String::from("public/404.html"), 404),
        };

        let res = create_response(res.0.as_str(), res.1).await?;

        writer.write_all(&res.as_bytes()).await.unwrap();
        writer.shutdown().await?;

        Ok(())
    }
}

async fn create_response(fname: &str, status: u16) -> Result<String, io::Error> {
    let mut f = File::open(fname).await?;
    let mut buffer = String::new();

    f.read_to_string(&mut buffer).await?;

    let length = buffer.len();

    let response = format!("HTTP/1.1 {status}\r\nContent-Length: {length}\r\n\r\n{buffer}");

    Ok(response)
}
