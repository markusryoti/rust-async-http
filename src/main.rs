use std::{error::Error, net::SocketAddr};

use headers::request::{get_body, get_headers};
use routing::router::router;
use tokio::{
    io::BufReader,
    net::{TcpListener, TcpStream},
};

pub mod headers;
pub mod routing;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let srv_addr = "127.0.0.1:7878";

    let listener = TcpListener::bind(srv_addr).await?;

    println!("Async server listening on {srv_addr}");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, addr).await {
                handle_socket_error(e, addr);
            }
        });
    }
}

async fn handle_connection(socket: TcpStream, addr: SocketAddr) -> tokio::io::Result<()> {
    let (reader, writer) = socket.into_split();
    let mut buffered_reader = BufReader::new(reader);

    let headers = get_headers(&mut buffered_reader).await?;
    let body = get_body(&mut buffered_reader, headers.content_length).await?;

    println!(
        "Request from {}:\nHeaders: {:#?}\nBody: {:?}",
        addr,
        headers.values,
        String::from_utf8_lossy(&body)
    );

    router(writer, headers, body).await
}

fn handle_socket_error<T: Error>(e: T, addr: SocketAddr) {
    eprintln!("Error handling {}: {}", addr, e);
}
