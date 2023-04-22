use bytes::BytesMut;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut,
}

#[derive(Serialize, Deserialize)]
pub enum Frame {
    Connect { id: String },
    Disconnect,
    Text { content: String },
    Binary { content: Vec<u8> },
    Ping,
}

#[derive(Debug)]
pub enum FrameError {
    ParseError,
    ReadError,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        let buffer = BytesMut::new();
        Connection { stream, buffer }
    }

    pub fn parse_frame(&mut self) -> Result<Option<Frame>, FrameError> {
        if let Some(pos) = self.buffer.iter().position(|&ch| ch == b'\n') {
            let frame_raw = self.buffer.split_to(pos + 1);
            let str = String::from_utf8_lossy(&frame_raw);
            match from_str::<Frame>(&str) {
                Ok(frame) => Ok(Some(frame)),
                Err(_) => Err(FrameError::ParseError),
            }
        } else {
            Ok(None)
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>, FrameError> {
        if let Some(frame) = self.parse_frame()? {
            return Ok(Some(frame));
        }
        let mut buf = [0; 1024];
        match self.stream.read(&mut buf).await {
            Ok(0) => {
                if self.buffer.is_empty() {
                    Ok(None)
                } else {
                    Err(FrameError::ReadError)
                }
            }
            Ok(n) => {
                self.buffer.extend(&buf[..n]);
                self.parse_frame()
            }
            Err(_) => match self.buffer.is_empty() {
                true => Ok(None),
                false => Err(FrameError::ReadError),
            },
        }
    }

    pub async fn write_frame(&mut self, frame: Frame) -> std::io::Result<usize> {
        let str = to_string(&frame).unwrap() + "\n";
        self.stream.write(str.as_bytes()).await
    }
}
