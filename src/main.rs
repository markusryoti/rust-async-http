use routing::router::Router;
use server::server::Server;

pub mod headers;
pub mod routing;
pub mod server;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let mut router = Router::new();

    router.add_route("/", "public/index.html");

    let server = Server::new(router, 7878);

    server.start().await?;

    Ok(())
}
