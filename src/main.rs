use routing::router::{HandlerFn, Router};
use server::server::Server;

pub mod http;
pub mod routing;
pub mod server;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let mut router = Router::new();

    // router.add_route("/", "public/index.html");
    // router.add_route("/kitty", "public/kitty.html");

    let home_handler: HandlerFn = Box::new(|req, res| {
        Box::pin(async move {
            println!("Home handler received request for path: {}", req.path);
            res.body = String::from("<h1>hello :)</h1>");
            res.status = 200;
        })
    });

    router.add_route("/", home_handler);

    let server = Server::new(router, "127.0.0.1", 7878);

    server.start().await?;

    Ok(())
}
