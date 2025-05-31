use routing::router::Router;
use server::server::Server;

pub mod http;
pub mod routing;
pub mod server;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let mut router = Router::new();

    let home_handler = async_handler!(|req, res| {
        println!("Handling path: {}", req.path);
        res.status = 200;
        res.body = String::from("<h1>hello after fixing the macro <3</h1>");
    });

    router.add_route("/", home_handler);

    let server = Server::new(router, "127.0.0.1", 7878);

    server.start().await?;

    Ok(())
}
