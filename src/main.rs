use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use chrono::prelude::*;
use clap::Parser;

mod lxi;
use lxi::{LxiDevice, DEFAULT_PORT};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(
        help = "Sampling period in seconds",
        short = 't',
        long = "period",
        value_name = "SECONDS",
        default_value = "1.0"
    )]
    period: f64,

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
        conflicts_with_all = ["current", "resistance", "two", "four"]
    )]
    voltage: Option<String>,

    #[arg(
        help = "Switch to current measurement",
        short = 'I',
        long,
        value_name = "RANGE",
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
        help = "remote port for SCPI",
        short,
        long,
        default_value_t = DEFAULT_PORT
    )]
    port: u16,

    #[arg(help = "Network name or IP address of the instrument")]
    host: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let dc_ac = usize::from(cli.ac);
    let two_four = usize::from(cli.four);

    let conf = cli
        .voltage
        .map(|range| format!("CONF:VOLT:{} {}", ["DC", "AC"][dc_ac], range))
        .or_else(|| {
            cli.current
                .map(|range| format!("CONF:CURR:{} {}", ["DC", "AC"][dc_ac], range))
        })
        .or_else(|| {
            cli.resistance
                .map(|range| format!("CONF:{} {}", ["RES", "FRES"][two_four], range))
        });

    let host = &cli.host;
    let port = cli.port;

    let period = Duration::from_secs_f64(cli.period);
    let num_samples = cli.num_samples.unwrap_or(u32::MAX);

    let mut dmm = LxiDevice::connect_with_port(host, port)
        .with_context(|| format!("Connecting to instrument `{host}` failed"))?;

    let ident = dmm
        .request("*IDN?")
        .context("Requesting instrument identification failed")?;

    if let Some(conf) = conf {
        dmm.send(&conf)
            .context("Setting voltage measurement range failed")?;
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
            let planed = started + sequence * period;
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
