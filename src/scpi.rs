use std::io::prelude::*;
use std::net::{Shutdown, TcpStream};
use std::time::Duration;

use anyhow::{bail, Result};

pub const DEFAULT_PORT: u16 = 5025;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Device {
    stream: TcpStream,
    debug: bool,
}

impl Device {
    #[allow(dead_code)]
    pub fn connect(host: &str) -> Result<Device> {
        Self::connect_with_port(host, DEFAULT_PORT)
    }

    pub fn connect_with_port(host: &str, port: u16) -> Result<Device> {
        let stream = TcpStream::connect((host, port))?;

        stream.set_read_timeout(Some(DEFAULT_TIMEOUT))?;
        stream.set_write_timeout(Some(DEFAULT_TIMEOUT))?;
        stream.set_nodelay(true)?;

        Ok(Device {
            stream,
            debug: false,
        })
    }

    pub fn close(self) -> Result<()> {
        self.stream.shutdown(Shutdown::Both)?;
        Ok(())
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    pub fn send(&mut self, msg: &str) -> Result<()> {
        if self.debug {
            eprintln!("> {msg}");
        }

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

        let msg = std::str::from_utf8(data)?.into();

        if self.debug {
            eprintln!("< {msg}");
        }

        Ok(msg)
    }

    pub fn request(&mut self, msg: &str) -> Result<String> {
        self.send(msg)?;
        self.receive()
    }

    pub fn read(&mut self) -> Result<f64> {
        Ok(self.request("READ?")?.parse()?)
    }

    pub fn identification(&mut self) -> Result<Identification> {
        let response = self.request("*IDN?")?;

        let fields = response.split(',').collect::<Vec<_>>();

        if fields.len() == 4 {
            let manufacturer = fields[0].trim().into();
            let model = fields[1].trim().into();
            let serial = fields[2].trim().into();
            let firmware = fields[3].trim().into();
            Ok(Identification {
                manufacturer,
                model,
                serial,
                firmware,
            })
        } else {
            let model = response.trim().into();

            Ok(Identification {
                manufacturer: "?".into(),
                model,
                serial: "?".into(),
                firmware: "?".into(),
            })
        }
    }

    pub fn fetch_error(&mut self) -> Result<Option<ScpiError>> {
        use regex::Regex;

        let response = self.request("SYST:ERR?")?;
        let re = Regex::new(r#"^([-+]?\d+\s*)\s*,\s*"(.+)"\s*$"#)?;

        if let Some(caps) = re.captures(&response) {
            let code = caps[1].parse()?;
            if code != 0 {
                let text = caps[2].to_string();
                Ok(Some(ScpiError { code, text }))
            } else {
                Ok(None)
            }
        } else {
            bail!("Could not parse error response from instrument");
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScpiError {
    pub code: i32,
    pub text: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Identification {
    pub manufacturer: String,
    pub model: String,
    pub serial: String,
    pub firmware: String,
}
