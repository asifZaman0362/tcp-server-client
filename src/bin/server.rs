use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    spawn,
};

struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>,
}

#[derive(Debug)]
enum FrameError {
    ParseError,
    ReadError,
}

#[derive(Serialize, Deserialize, Debug)]
enum Frame {
    Text(String),
    Binary(Vec<u8>),
    Ping,
    Ok,
    Disconnect,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        let buffer = vec![];
        Connection { stream, buffer }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>, FrameError> {
        if let Some(pos) = self.buffer.iter().position(|&ch| ch == b'\n') {
            let string = String::from_utf8_lossy(&self.buffer[..pos]).to_string();
            self.buffer = self.buffer[pos + 1..].to_vec();
            match from_str::<Frame>(&string) {
                Ok(frame) => Ok(Some(frame)),
                Err(_) => Err(FrameError::ParseError),
            }
        } else {
            Ok(None)
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>, FrameError> {
        let mut buf = [0; 1024];
        if let Some(frame) = self.parse_frame()? {
            return Ok(Some(frame));
        }
        match self.stream.read(&mut buf).await {
            Ok(0) => {
                if self.buffer.is_empty() {
                    Ok(None)
                } else {
                    Err(FrameError::ReadError)
                }
            }
            Ok(n) => {
                println!("got {n} bytes!");
                self.buffer.extend_from_slice(&buf[..n]);
                self.parse_frame()
            }
            Err(_) => match self.buffer.len() {
                0 => Err(FrameError::ReadError),
                _ => Ok(None),
            },
        }
    }

    pub async fn write_frame(&mut self, frame: Frame) {
        let buf = to_string(&frame).unwrap() + "\n";
        let _ = self.stream.write_all(buf.as_bytes()).await;
    }
}

async fn process_client(mut client: Connection) {
    loop {
        match client.read_frame().await {
            Ok(Some(frame)) => {
                if let Frame::Disconnect = frame {
                    println!("connection closed!");
                    break;
                }
                println!("frame got from client: {frame:?}");
                client.write_frame(Frame::Ok).await;
            }
            Ok(None) => {
                println!("connection closed!");
                break;
            }
            Err(err) => {
                println!("err {err:?}");
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:2000").await?;
    loop {
        let (stream, _) = listener.accept().await?;
        spawn(async move {
            let conn = Connection::new(stream);
            process_client(conn).await;
        });
    }
}
