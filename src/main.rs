use http::{headers::HttpHeaderName, request::HttpRequest, response::HttpResponse};
use routing::router::Router;
use serde::{Deserialize, Serialize};
use server::server::Server;
use tokio::{fs::File, io::AsyncReadExt};

pub mod http;
pub mod routing;
pub mod server;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let mut router = Router::new();

    let index_handler = async_handler!(|_req, res| {
        let mut f = File::open("public/index.html").await.unwrap();
        let mut buffer = String::new();

        f.read_to_string(&mut buffer).await.unwrap();

        res.body = buffer;
    });

    router.add_route("/", index_handler);
    router.add_route("/kitty", async_fn_handler!(kitty_handler));
    router.add_route("/json", async_fn_handler!(json_handler));

    let server = Server::new(router, "127.0.0.1", 7878);

    server.start().await?;

    Ok(())
}

async fn kitty_handler(_req: &HttpRequest, res: &mut HttpResponse) {
    let mut f = File::open("public/kitty.html").await.unwrap();
    let mut buffer = String::new();

    f.read_to_string(&mut buffer).await.unwrap();

    res.body = buffer;
}

#[derive(Serialize, Deserialize, Debug)]
struct Greeting {
    hello: String,
}

async fn json_handler(_req: &HttpRequest, res: &mut HttpResponse) {
    let greeting = Greeting {
        hello: "world".to_string(),
    };

    let serialized = serde_json::to_string(&greeting).unwrap();

    res.add_header(HttpHeaderName::from("Content-Type"), "application/json");

    res.body = serialized;
    res.status_code = 200;
}
