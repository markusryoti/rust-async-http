use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use log::{error, info};
use tokio::net::TcpListener;
use tokio::{io::BufReader, net::TcpStream};

use crate::http::request::HttpRequest;
use crate::routing::router;

pub struct Server {
    host: String,
    port: u16,
    router: router::Router,
}

impl Server {
    pub fn new(router: router::Router, host: &str, port: u16) -> Server {
        Server {
            host: host.to_string(),
            router,
            port,
        }
    }

    pub async fn start(self) -> std::io::Result<()> {
        let srv_addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&srv_addr).await?;

        let server = Arc::new(self);

        info!("Async server listening on {srv_addr}");

        loop {
            let server = Arc::clone(&server);

            let (socket, addr) = listener.accept().await?;

            info!("New connection from {addr}");

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

        let req = HttpRequest::parse(&mut buffered_reader).await?;

        let body = match &req.body.as_ref() {
            Some(b) => String::from_utf8_lossy(b).to_string(),
            None => "no body".to_string(),
        };

        info!(
            "Handling request from {}, NumHeaders: {}, Body: {:#?}",
            addr,
            req.headers.values.len(),
            body,
        );

        self.router.match_route(writer, addr, &req).await
    }
}

fn handle_socket_error<T: Error>(e: T, addr: SocketAddr) {
    error!("Error handling {}: {}", addr, e);
}
