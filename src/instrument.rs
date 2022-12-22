use chrono::prelude::*;
use std::time::{Duration, Instant};

use crate::scpi::{self, Identification};
use anyhow::{bail, Context, Result};

pub fn connect(host: &str, port: u16) -> Result<scpi::Device> {
    scpi::Device::connect_with_port(host, port)
        .with_context(|| format!("Connecting to instrument `{host}` (port {port}) failed"))
}

pub fn disconnect(dmm: scpi::Device) -> Result<()> {
    dmm.close().context("Disconnecting from instrument failed")
}

pub fn beep(dmm: &mut scpi::Device) -> Result<()> {
    dmm.send("SYST:BEEP").context("Beeping instrument failed")
}

pub fn identification(dmm: &mut scpi::Device) -> Result<Identification> {
    dmm.identification()
        .context("Requesting instrument identification failed")
}

pub fn configure(dmm: &mut scpi::Device, configs: Vec<String>, reset: bool) -> Result<()> {
    if reset {
        dmm.send("*RST")?;
    } else {
        dmm.send("*CLS")?;
    }

    if let Some(error) = dmm.fetch_error()? {
        bail!(
            "Clearing error state failed, instrument returned error code {}: {}",
            error.code,
            error.text
        );
    }

    if !configs.is_empty() {
        for config in configs.iter() {
            dmm.send(config).context("Configuring instrument failed")?;
        }

        if let Some(error) = dmm.fetch_error()? {
            bail!(
                "Configuring instrument failed, instrument returned error code {}: {}",
                error.code,
                error.text
            );
        }
    }

    Ok(())
}

pub fn read(
    dmm: &mut scpi::Device,
    sequence: u32,
) -> Result<(DateTime<Local>, Instant, Duration, f64)> {
    let datetime = Local::now();
    let moment = Instant::now();

    let reading = dmm
        .read()
        .with_context(|| format!("Reading measurement #{sequence} from instrument failed"))?;

    let latency = moment.elapsed();

    Ok((datetime, moment, latency, reading))
}
