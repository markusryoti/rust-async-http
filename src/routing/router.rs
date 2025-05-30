use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::tcp::OwnedWriteHalf,
};

use crate::headers::request::{Headers, HttpFirstRow};

pub async fn router(
    mut writer: OwnedWriteHalf,
    headers: Headers,
    _body: Vec<u8>,
) -> Result<(), std::io::Error> {
    let resource_row = headers.values.get(0).unwrap();
    let rr = resource_row.parse::<HttpFirstRow>().unwrap();

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
