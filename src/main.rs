use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossbeam::channel::{select, tick};

mod lxi;
use lxi::LxiDevice;

fn main() -> Result<()> {
    let host = "dmm1.home";

    let mut dmm = LxiDevice::connect(host)
        .with_context(|| format!("Connecting to device `{host}` failed"))?;

    let ident = dmm
        .request("*IDN?")
        .context("Requesting device identification failed")?;

    dmm.send("CONF:VOLT:DC 10")
        .context("Setting voltage measurement range failed")?;

    println!("# Identification: {ident}");
    println!("sequence,moment,latency,reading");

    let ticker = tick(Duration::from_millis(1000));
    let ticker_started = Instant::now();

    let mut sequence = 0u32;

    loop {
        select! {
                recv(ticker) -> _ => {
                    let reading_started = Instant::now();
                    let reading = dmm.read()?;
                    let reading_stopped = Instant::now();

                    let latency = (reading_stopped - reading_started).as_secs_f64();
                    let moment = (reading_started - ticker_started).as_secs_f64();

                    sequence += 1;

                    println!("{sequence},{moment},{latency},{reading}");
                }
        }
    }

    //dmm.close()?;

    //Ok(())
}
