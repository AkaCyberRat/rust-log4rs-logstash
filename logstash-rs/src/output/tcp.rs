use crate::prelude::*;
use std::fmt::Write as FMTWrite;
use std::io::Write as IOWrite;
use std::net::TcpStream;

#[derive(Debug)]
pub struct TcpSender {
    hostname: String,
    port: u16,
    stream: Option<TcpStream>,
}

impl TcpSender {
    pub fn new(hostname: String, port: u16) -> Self {
        Self {
            hostname,
            port,
            stream: None,
        }
    }

    fn send_raw_data(&mut self, data: &[u8]) -> Result<()> {
        let stream = self.get_stream()?;
        stream.write_all(data)?;
        Ok(())
    }

    fn get_stream(&mut self) -> Result<&mut TcpStream> {
        if self.stream.is_none() {
            let stream = TcpStream::connect((self.hostname.as_str(), self.port))?;
            self.stream = Some(stream);
        }
        self.stream.as_mut().ok_or_else(|| Error::Unknown.into())
    }
}

impl Sender for TcpSender {
    fn send(&mut self, event: &Event) -> Result<()> {
        let mut event = serde_json::to_string(event)?;
        event.write_char('\n')?;
        self.send_raw_data(event.as_bytes())?;
        Ok(())
    }

    fn send_batch(&mut self, events: &[Event]) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }
        let mut buf = vec![];
        for event in events {
            serde_json::to_writer(&mut buf, event)?;
            buf.push('\n' as u8);
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        if let Some(stream) = self.stream.as_mut() {
            stream.flush()?;
        }
        Ok(())
    }
}
