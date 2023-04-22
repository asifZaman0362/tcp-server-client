use tcp_connection::Connection;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:2000").await?;
    let mut conn = Connection::new(stream);
    let id = "god".to_string();
    conn.write_frame(tcp_connection::Frame::Connect { id })
        .await?;
    conn.read_frame().await.unwrap();
    conn.write_frame(tcp_connection::Frame::Text {
        content: "Hello, server!".to_string(),
    })
    .await?;
    conn.read_frame().await.unwrap();
    conn.write_frame(tcp_connection::Frame::Disconnect).await?;
    Ok(())
}
