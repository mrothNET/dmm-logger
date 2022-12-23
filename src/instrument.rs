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

    batch_commands("Configuring", dmm, configs)?;

    let result = dmm
        .request("*OPC?")
        .context("Waiting for operation condition on instrument failed")?;

    if result != "1" {
        bail!("Unexpected result from instrument for operation condition");
    }

    Ok(())
}

pub fn unconfigure(dmm: &mut scpi::Device, unconfigs: Vec<String>) -> Result<()> {
    batch_commands("Un-configuring", dmm, unconfigs)
}

pub fn batch_commands(context: &str, dmm: &mut scpi::Device, commands: Vec<String>) -> Result<()> {
    if !commands.is_empty() {
        for cmd in commands.iter() {
            dmm.send(cmd)
                .context(format!("{context} instrument failed"))?;
        }

        if let Some(error) = dmm.fetch_error()? {
            bail!(
                "{} instrument failed, instrument returned error code {}: {}",
                context,
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
