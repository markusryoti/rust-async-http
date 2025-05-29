use std::{error::Error, net::SocketAddr, str::FromStr};

use tokio::{
    fs::File,
    io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{
        TcpListener, TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
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

    let resource_row = headers.values.get(0).unwrap();
    let rr = resource_row.parse::<ResourceRow>().unwrap();

    router(writer, rr).await
}

async fn get_headers(buffered_reader: &mut BufReader<OwnedReadHalf>) -> Result<Headers, io::Error> {
    let mut headers = Vec::new();
    let mut content_length = 0;
    let mut line = String::new();

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

    Ok(Headers {
        values: headers,
        content_length: content_length,
    })
}

struct Headers {
    values: Vec<String>,
    content_length: usize,
}

async fn get_body(
    buffered_reader: &mut BufReader<OwnedReadHalf>,
    content_length: usize,
) -> Result<Vec<u8>, io::Error> {
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        buffered_reader.read_exact(&mut body).await?;
    }

    Ok(body)
}

#[derive(Debug)]
struct ResourceRow {
    _method: String,
    _protocol: String,
    resource: String,
}

impl FromStr for ResourceRow {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.trim().split_whitespace().collect();

        if parts.len() != 3 {
            return Err("Invalid request line format");
        }

        Ok(ResourceRow {
            _method: parts[0].to_string(),
            resource: parts[1].to_string(),
            _protocol: parts[2].to_string(),
        })
    }
}

async fn router(mut writer: OwnedWriteHalf, rr: ResourceRow) -> Result<(), std::io::Error> {
    let res = match rr.resource.as_str() {
        "/" => ("public/index.html", 200),
        _ => ("public/404.html", 404),
    };

    let res = get_response(res.0, res.1).await?;

    writer.write_all(&res.as_bytes()).await.unwrap();
    writer.shutdown().await?;

    Ok(())
}

async fn get_response(fname: &str, status: u16) -> Result<String, io::Error> {
    let mut f = File::open(fname).await?;
    let mut buffer = String::new();

    f.read_to_string(&mut buffer).await?;

    let length = buffer.len();

    let response = format!("HTTP/1.1 {status}\r\nContent-Length: {length}\r\n\r\n{buffer}");

    Ok(response)
}

fn handle_socket_error<T: Error>(e: T, addr: SocketAddr) {
    eprintln!("Error handling {}: {}", addr, e);
}
