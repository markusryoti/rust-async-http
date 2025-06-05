use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use log::{error, info};
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio::{io::BufReader, net::TcpStream};

use crate::http::headers::{ConnectionHeaderValue, HttpHeaderName, HttpHeaderValue, HttpHeaders};
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
        let (reader, mut writer) = socket.into_split();
        let mut buffered_reader = BufReader::new(reader);

        loop {
            let result = timeout(
                Duration::from_secs(30),
                HttpRequest::parse(&mut buffered_reader),
            )
            .await?;

            let req = match result {
                Ok(r) => r,
                Err(e) => return Err(e),
            };

            let body = match &req.body.as_ref() {
                Some(b) => String::from_utf8_lossy(b).to_string(),
                None => "no body".to_string(),
            };

            let keep_alive = should_keep_alive(&req.headers);

            info!(
                "Handling request from {}, NumHeaders: {}, LenBody: {}",
                addr,
                req.headers.values.len(),
                body.len(),
            );

            self.router
                .match_route(&mut writer, addr, &req, keep_alive)
                .await?;

            if !keep_alive {
                info!("No keep-alive, exiting");
                break;
            }

            info!("Keep-alive, continuing");
        }

        Ok(())
    }
}

fn should_keep_alive(headers: &HttpHeaders) -> bool {
    let connection_header = headers.get(&HttpHeaderName::Connection);
    if let Some(ch) = connection_header {
        let res = ch.get(0);
        let header_value = match res {
            None => {
                error!("Found connection header but now value. Don't use keep-alive");
                return false;
            }
            Some(hv) => hv,
        };

        if let HttpHeaderValue::Connection(cv) = header_value {
            if let ConnectionHeaderValue::Close = cv {
                info!("Client requested connection close");
                return false;
            }
            if let ConnectionHeaderValue::KeepAlive = cv {
                info!("Client requested keep alive");
                return true;
            }
        }
    }
    info!("No connection header, defaulting to keep-alive (HTTP/1.1)");
    return true;
}

fn handle_socket_error<T: Error>(e: T, addr: SocketAddr) {
    error!("Error handling {}: {}", addr, e);
}
