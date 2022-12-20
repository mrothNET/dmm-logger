use std::io::prelude::*;
use std::net::{Shutdown, TcpStream};
use std::time::{Duration, Instant};

use anyhow::Result;

pub const DEFAULT_PORT: u16 = 5025;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

pub struct LxiDevice {
    stream: TcpStream,
}

impl LxiDevice {
    #[allow(dead_code)]
    pub fn connect(host: &str) -> Result<LxiDevice> {
        Self::connect_with_port(host, DEFAULT_PORT)
    }

    pub fn connect_with_port(host: &str, port: u16) -> Result<LxiDevice> {
        let stream = TcpStream::connect((host, port))?;

        stream.set_read_timeout(Some(DEFAULT_TIMEOUT))?;
        stream.set_write_timeout(Some(DEFAULT_TIMEOUT))?;
        stream.set_nodelay(true)?;

        Ok(LxiDevice { stream })
    }

    pub fn close(self) -> Result<()> {
        self.stream.shutdown(Shutdown::Both)?;
        Ok(())
    }

    pub fn send(&mut self, msg: &str) -> Result<()> {
        let mut writer = std::io::BufWriter::with_capacity(msg.len() + 2, &self.stream);
        writer.write_all(msg.as_bytes())?;
        writer.write_all("\r\n".as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    pub fn receive(&mut self) -> Result<String> {
        let mut buffer = [0u8; 2048];
        let bytes_read = self.stream.read(&mut buffer)?;

        let data = &buffer[0..bytes_read];

        let data = if data.ends_with(b"\r\n") {
            &data[0..data.len() - 2]
        } else if data.ends_with(b"\n") {
            &data[0..data.len() - 1]
        } else {
            data
        };

        Ok(std::str::from_utf8(data)?.into())
    }

    pub fn request(&mut self, msg: &str) -> Result<String> {
        self.send(msg)?;
        self.receive()
    }

    pub fn read(&mut self) -> Result<f64> {
        Ok(self.request("READ?")?.parse()?)
    }

    pub fn timed_read(&mut self) -> Result<(Instant, Duration, f64)> {
        let started = Instant::now();
        let reading = self.read()?;
        let latency = started.elapsed();
        Ok((started, latency, reading))
    }
}
