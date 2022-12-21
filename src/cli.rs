use crate::scpi::DEFAULT_PORT;
use anyhow::{bail, Result};
use clap::Parser;
use std::time::Duration;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
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

impl Cli {
    pub fn validate(&self) -> Result<&Self> {
        if self.interval == 0.0 {
            bail!("Sampling interval 0.0 seconds is not allowed");
        }

        if self.rate == Some(0.0) {
            bail!("Sampling rate 0.0 hertz is not allowed");
        }

        if self.num_samples == Some(0) {
            bail!("Number of samples 0 is not allowed");
        }

        Ok(self)
    }

    pub fn debug(&self) -> bool {
        self.debug
    }

    pub fn host(&self) -> &str {
        self.host.as_ref()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn sample_period(&self) -> Duration {
        Duration::from_secs_f64(self.rate.map(|f| 1.0 / f).unwrap_or(self.interval))
    }

    pub fn num_samples(&self) -> u32 {
        self.num_samples.unwrap_or(u32::MAX)
    }

    pub fn scpi_commands(&self) -> Vec<String> {
        let mut configs = Vec::<String>::new();

        let dc_ac = ["DC", "AC"][usize::from(self.ac)];
        let res_fres = ["RES", "FRES"][usize::from(self.four)];

        if let Some(volts) = self.voltage.as_ref() {
            configs.push(format!("CONF:VOLT:{dc_ac} {volts}"));
        } else if let Some(amps) = self.current.as_ref() {
            configs.push(format!("CONF:CURR:{dc_ac} {amps}"));
        } else if let Some(ohms) = self.resistance.as_ref() {
            configs.push(format!("CONF:{res_fres} {ohms}"));
        }

        if let Some(nplc) = &self.nplc {
            if self.voltage.is_some() {
                configs.push(format!("VOLT:{dc_ac}:NPLC {nplc}"));
            } else if self.current.is_some() {
                configs.push(format!("CURR:{dc_ac}:NPLC {nplc}"));
            } else if self.resistance.is_some() {
                configs.push(format!("{res_fres}:NPLC {nplc}"));
            }
        };

        configs
    }
}