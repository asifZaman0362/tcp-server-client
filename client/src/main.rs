use tcp_connection::Connection;
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    net::TcpStream,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut stdin = stdin();
    let stream = TcpStream::connect("127.0.0.1:2000").await?;
    let mut conn = Connection::new(stream);
    let id = "god".to_string();
    conn.write_frame(tcp_connection::Frame::Connect { id })
        .await?;
    conn.read_frame().await.unwrap();
    let mut buf = String::new();
    let mut reader = BufReader::new(&mut stdin);
    loop {
        buf.clear();
        println!("Enter a string (type 'disconnect' to stop): ");
        reader.read_line(&mut buf).await?;
        if buf.starts_with("disconnect") {
            break;
        }
        conn.write_frame(tcp_connection::Frame::Text {
            content: buf.clone(),
        })
        .await?;
        conn.read_frame().await.unwrap();
    }
    conn.write_frame(tcp_connection::Frame::Disconnect).await?;
    Ok(())
}
