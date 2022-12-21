use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use chrono::prelude::*;
use clap::Parser;

mod scpi;
use scpi::{ScpiDevice, DEFAULT_PORT};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(
        help = "Sampling interval in seconds",
        long,
        value_name = "SECONDS",
        default_value = "1.0",
        conflicts_with_all = ["rate"]
    )]
    interval: f64,

    #[arg(
        help = "Sampling rate in hertz",
        long,
        value_name = "HERTZ",
        conflicts_with_all = ["interval"]
    )]
    rate: Option<f64>,

    #[arg(
        short = 'n',
        value_name = "COUNT",
        help = "Number of samples to take [default: unlimited]"
    )]
    num_samples: Option<u32>,

    #[arg(
        help = "Switch to voltage measurement",
        short = 'U',
        long,
        value_name = "RANGE",
        aliases = ["volts", "volt"],
        conflicts_with_all = ["current", "resistance", "two", "four"]
    )]
    voltage: Option<String>,

    #[arg(
        help = "Switch to current measurement",
        short = 'I',
        long,
        value_name = "RANGE",
        aliases = ["amperes", "ampere"],
        conflicts_with_all = ["voltage", "resistance", "two", "four"]
    )]
    current: Option<String>,

    #[arg(
        help = "DC-mode for voltage or current [default]",
        long = "DC",
        alias = "dc",
        requires = "voltage",
        requires = "current",
        conflicts_with_all = ["resistance", "ac", "two", "four"]
    )]
    dc: bool,

    #[arg(
        help = "AC-mode for voltage or current",
        long = "AC",
        alias = "ac",
        requires = "voltage",
        requires = "current",
        conflicts_with_all = ["resistance", "dc", "two", "four"]
    )]
    ac: bool,

    #[arg(
        help = "Switch to resistance measurement",
        short = 'R',
        long,
        value_name = "RANGE",
        aliases = ["ohms", "ohm"],
        conflicts_with_all = ["voltage", "current", "dc", "ac"]
    )]
    resistance: Option<String>,

    #[arg(
        help = "2-wire resistance measurement [default]",
        short = '2',
        long = "two-wire",
        aliases = ["2-wire", "2wire", "twowire", "two"],
        requires = "resistance",
        conflicts_with_all = ["four", "voltage", "current", "dc", "ac"]
    )]
    two: bool,

    #[arg(
        help = "4-wire resistance measurement",
        short = '4',
        long = "four-wire",
        aliases = ["4-wire", "4wire", "fourwire","four"],
        requires = "resistance",
        conflicts_with_all = ["two", "voltage", "current", "dc", "ac"]
    )]
    four: bool,

    #[arg(
        help = "Number of Power Line Cycles",
        long,
        value_name = "NPLC",
        requires = "voltage",
        requires = "current",
        requires = "resistance"
    )]
    nplc: Option<String>,

    #[arg(
        help = "Network port for SCPI",
        long,
        default_value_t = DEFAULT_PORT
    )]
    port: u16,

    #[arg(help = "Print SCPI communication to stderr", long)]
    debug: bool,

    #[arg(help = "Network name or IP address of the instrument.")]
    host: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let dc_ac = ["DC", "AC"][usize::from(cli.ac)];
    let res_fres = ["RES", "FRES"][usize::from(cli.four)];

    #[allow(clippy::manual_map)]
    let conf = {
        if let Some(volts) = cli.voltage.as_ref() {
            Some(format!("CONF:VOLT:{dc_ac} {volts}"))
        } else if let Some(amps) = cli.current.as_ref() {
            Some(format!("CONF:CURR:{dc_ac} {amps}"))
        } else if let Some(ohms) = cli.resistance.as_ref() {
            Some(format!("CONF:{res_fres} {ohms}"))
        } else {
            None
        }
    };

    let nplc = cli.nplc.map(|nplc| {
        if cli.voltage.is_some() {
            format!("VOLT:{dc_ac}:NPLC {nplc}")
        } else if cli.current.is_some() {
            format!("CURR:{dc_ac}:NPLC {nplc}")
        } else if cli.resistance.is_some() {
            format!("{res_fres}:NPLC {nplc}")
        } else {
            unreachable!()
        }
    });

    if cli.interval == 0.0 {
        bail!("Sampling interval 0.0 seconds is not allowed");
    }

    if cli.rate == Some(0.0) {
        bail!("Sampling rate 0.0 hertz is not allowed");
    }

    let host = &cli.host;
    let port = cli.port;

    let sample_period = Duration::from_secs_f64(cli.rate.map(|f| 1.0 / f).unwrap_or(cli.interval));
    let num_samples = cli.num_samples.unwrap_or(u32::MAX);

    let mut dmm = ScpiDevice::connect_with_port(host, port)
        .with_context(|| format!("Connecting to instrument `{host}` failed"))?;

    dmm.set_debug(cli.debug);

    let ident = dmm
        .request("*IDN?")
        .context("Requesting instrument identification failed")?;

    dmm.send("*CLS")?;

    if let Some(error) = dmm.fetch_error()? {
        bail!(
            "Clearing error state failed with error code {}: {}",
            error.code,
            error.text
        );
    }

    if let Some(conf) = conf {
        dmm.send(&conf)
            .context("Setting voltage measurement range failed")?;
    }

    if let Some(nplc) = nplc {
        dmm.send(&nplc)
            .context("Setting number of power line cycles failed")?;
    }

    if let Some(error) = dmm.fetch_error()? {
        bail!(
            "Setting up instrument failed with error code {}: {}",
            error.code,
            error.text
        );
    }

    println!("# Identification: {ident}");
    println!("sequence,date,time,moment,delay,latency,reading");

    let log = |sequence: u32,
               datetime: DateTime<Local>,
               moment: f64,
               delay: f64,
               latency: f64,
               reading: f64| {
        let date = datetime.format("%Y-%m-%d");
        let time = datetime.format("%H:%M:%S.%3f");
        println!("{sequence},{date},{time},{moment},{delay},{latency},{reading}");
    };

    if num_samples > 0 {
        let term = install_signal_hooks()?;

        let datetime = Local::now();
        let (started, latency, first_reading) = dmm
            .timed_read()
            .context("Reading first measurement from instrument failed")?;

        log(0, datetime, 0.0, 0.0, latency.as_secs_f64(), first_reading);

        for sequence in 1..num_samples {
            let planed = started + sequence * sample_period;
            if sleep_until(planed, &term) {
                let datetime = Local::now();
                let (moment, latency, reading) = dmm.timed_read().with_context(|| {
                    format!("Reading measurement #{sequence} from instrument failed")
                })?;
                let delay = (moment - planed).as_secs_f64();
                let moment = (moment - started).as_secs_f64();
                let latency = latency.as_secs_f64();
                log(sequence, datetime, moment, delay, latency, reading);
            } else {
                break;
            }
        }
    }

    dmm.close()?;
    Ok(())
}

fn install_signal_hooks() -> Result<Arc<AtomicBool>> {
    let term = Arc::new(AtomicBool::new(false));
    for signal in signal_hook::consts::TERM_SIGNALS {
        signal_hook::flag::register(*signal, Arc::clone(&term))?;
    }
    Ok(term)
}

fn sleep_until(until: Instant, term: &AtomicBool) -> bool {
    use std::thread::sleep;

    while !term.load(Ordering::Relaxed) {
        let now = Instant::now();
        if now >= until {
            return true;
        }
        let remaining = until - now;
        if remaining.as_millis() > 150 {
            sleep(Duration::from_millis(100));
        } else {
            sleep(remaining);
        }
    }

    false
}
