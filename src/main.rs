use clap::{Parser, command};
use http::{headers::HttpHeaderName, request::HttpRequest, response::HttpResponse};
use routing::router::Router;
use serde::{Deserialize, Serialize};
use server::server::Server;
use tokio::{fs::File, io::AsyncReadExt};

pub mod http;
pub mod routing;
pub mod server;

use log::{Level, info};

/// Rust async TCP/HTTP server
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    // Use keep alive or force the connection to close
    #[arg(short, long)]
    use_keep_alive: bool,
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    simple_logger::init_with_level(Level::Info).unwrap();

    let features = Args::parse();

    println!("Using features: {:?}", features);

    let mut router = Router::new();

    let index_handler = async_handler!(|_req, res| {
        let mut f = File::open("public/index.html").await.unwrap();
        let mut buffer = String::new();

        f.read_to_string(&mut buffer).await.unwrap();

        res.add_header(HttpHeaderName::from("Content-Type"), "text/html");
        res.body = buffer;
    });

    router.add_route("/", index_handler);
    router.add_route("/kitty", async_fn_handler!(kitty_handler));
    router.add_route("/json", async_fn_handler!(json_handler));

    let server = Server::new(router, "127.0.0.1", 7878, features);

    info!("Starting server");

    server.start().await?;

    info!("Server shutting down");

    Ok(())
}

async fn kitty_handler(_req: &HttpRequest, res: &mut HttpResponse) {
    let mut f = File::open("public/kitty.html").await.unwrap();
    let mut buffer = String::new();

    f.read_to_string(&mut buffer).await.unwrap();

    res.add_header(HttpHeaderName::from("Content-Type"), "text/html");

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
