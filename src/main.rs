use routing::router::Router;
use server::server::Server;

pub mod http;
pub mod routing;
pub mod server;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let mut router = Router::new();

    router.add_route("/", "public/index.html");
    router.add_route("/kitty", "public/kitty.html");

    let server = Server::new(router, "127.0.0.1", 7878);

    server.start().await?;

    Ok(())
}
