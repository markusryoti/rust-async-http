use std::error::Error;
use std::sync::Arc;
use std::{net::SocketAddr, usize};

use tokio::net::TcpListener;
use tokio::{io::BufReader, net::TcpStream};

use crate::{
    headers::request::{get_body, get_headers},
    routing::router,
};

pub struct Server {
    port: usize,
    router: router::Router,
}

impl Server {
    pub fn new(router: router::Router, port: usize) -> Server {
        Server { router, port }
    }

    pub async fn start(self) -> std::io::Result<()> {
        let srv_addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&srv_addr).await?;

        let server = Arc::new(self);
        let listener = Arc::new(listener);

        println!("Async server listening on {srv_addr}");

        loop {
            let listener = Arc::clone(&listener);
            let server = Arc::clone(&server);

            let (socket, addr) = listener.accept().await?;
            println!("New connection from {addr}");

            tokio::spawn(async move {
                if let Err(e) = server.handle_connection(socket, addr).await {
                    handle_socket_error(e, addr);
                }
            });
        }
    }

    pub async fn handle_connection(
        &self,
        socket: TcpStream,
        addr: SocketAddr,
    ) -> tokio::io::Result<()> {
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

        self.router.match_route(writer, headers, body).await
    }
}

fn handle_socket_error<T: Error>(e: T, addr: SocketAddr) {
    eprintln!("Error handling {}: {}", addr, e);
}
