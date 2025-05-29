use std::{error::Error, net::SocketAddr};

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

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
    let (reader, mut writer) = socket.into_split();
    let mut buffered_reader = BufReader::new(reader);

    let mut headers = Vec::new();
    let mut content_length = 0;
    let mut line = String::new();

    // Read headers
    loop {
        line.clear();

        let bytes_read = buffered_reader.read_line(&mut line).await?;
        if bytes_read == 0 || line == "\r\n" {
            break;
        }

        if let Some(cl) = line.strip_prefix("Content-Length: ") {
            content_length = cl.trim().parse::<usize>().unwrap_or(0);
        }

        headers.push(line.trim_end().to_string());
    }

    // Read body
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        buffered_reader.read_exact(&mut body).await?;
    }

    println!(
        "Request from {}:\nHeaders: {:#?}\nBody: {:?}",
        addr,
        headers,
        String::from_utf8_lossy(&body)
    );

    let response = "HTTP/1.1 200 OK\r\n\r\n<h1>Ok thx</h1>";

    writer.write_all(response.as_bytes()).await.unwrap();
    writer.shutdown().await
}

fn handle_socket_error<T: Error>(e: T, addr: SocketAddr) {
    eprintln!("Error handling {}: {}", addr, e);
}
