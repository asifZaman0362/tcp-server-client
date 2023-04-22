use tcp_connection::{Connection, Frame};
use tokio::{
    net::{TcpListener, TcpStream},
    spawn,
};

async fn process(stream: TcpStream) {
    let mut connection = Connection::new(stream);
    loop {
        match connection.read_frame().await {
            Ok(Some(Frame::Ping)) => {
                println!("got ping!");
                let _ = connection.write_frame(Frame::Ping).await;
            }
            Ok(Some(Frame::Disconnect)) => {
                println!("connection closed!");
                break;
            }
            Ok(Some(Frame::Binary { content })) => {
                println!("frame read binary: {content:?}");
                let _ = connection.write_frame(Frame::Ping).await;
            }
            Ok(Some(Frame::Text { content })) => {
                println!("frame read: {content}");
                let _ = connection.write_frame(Frame::Ping).await;
            }
            Ok(Some(Frame::Connect { id })) => {
                println!("{id} connected!");
                let _ = connection.write_frame(Frame::Ping).await;
            }
            Ok(None) => {}
            Err(err) => {
                println!("error {err:?}");
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let server = TcpListener::bind("127.0.0.1:2000").await?;
    loop {
        let (stream, _) = server.accept().await?;
        spawn(async move { process(stream).await });
    }
}
