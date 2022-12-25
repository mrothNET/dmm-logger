use crate::scpi::DEFAULT_PORT;
use anyhow::{bail, Result};
use clap::Parser;
use std::time::Duration;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

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
        help = "Switch off instruments display during logging",
        long,
        conflicts_with_all = ["display_text"]
    )]
    display_off: bool,

    #[arg(
        help = "Displays a text message on instrument during logging",
        long,
        value_name = "TEXT",
        aliases = ["display-message", "display-text"],
        conflicts_with_all = ["display_off"]
    )]
    display_text: Option<Option<String>>,

    #[arg(help = "Drop delayed samples or samples with high latency", long)]
    drop_slow_samples: bool,

    #[arg(
        help = "Configures instrument for voltage measurement",
        short = 'U',
        long,
        value_name = "RANGE",
        aliases = ["volts", "volt"],
        conflicts_with_all = ["current", "resistance", "two", "four"]
    )]
    voltage: Option<String>,

    #[arg(
        help = "Configures instrument for current measurement",
        short = 'I',
        long,
        value_name = "RANGE",
        aliases = ["amperes", "ampere"],
        conflicts_with_all = ["voltage", "resistance", "two", "four"]
    )]
    current: Option<String>,

    #[arg(
        help = "Selects DC-mode for voltage or current [default]",
        long = "DC",
        alias = "dc",
        requires = "voltage",
        requires = "current",
        conflicts_with_all = ["resistance", "ac", "two", "four"]
    )]
    dc: bool,

    #[arg(
        help = "Selects AC-mode for voltage or current",
        long = "AC",
        alias = "ac",
        requires = "voltage",
        requires = "current",
        conflicts_with_all = ["resistance", "dc", "two", "four"]
    )]
    ac: bool,

    #[arg(
        help = "Configures instrument for resistance measurement",
        short = 'R',
        long,
        value_name = "RANGE",
        aliases = ["ohms", "ohm"],
        conflicts_with_all = ["voltage", "current", "dc", "ac"]
    )]
    resistance: Option<String>,

    #[arg(
        help = "Selects 2-wire resistance measurement [default]",
        short = '2',
        long = "two-wire",
        aliases = ["2-wire", "2wire", "twowire", "two"],
        requires = "resistance",
        conflicts_with_all = ["four", "voltage", "current", "dc", "ac"]
    )]
    two: bool,

    #[arg(
        help = "Selects 4-wire resistance measurement",
        short = '4',
        long = "four-wire",
        aliases = ["4-wire", "4wire", "fourwire","four"],
        requires = "resistance",
        conflicts_with_all = ["two", "voltage", "current", "dc", "ac"]
    )]
    four: bool,

    #[arg(
        help = "Resolution in units as the measurement function",
        long,
        value_name = "VALUE",
        aliases = ["res"],
        requires = "voltage",
        requires = "current",
        requires = "resistance",
        conflicts_with_all = ["nplc"]
    )]
    resolution: Option<String>,

    #[arg(
        help = "Integration time in number of power line cycles",
        long,
        value_name = "NPLC",
        requires = "voltage",
        requires = "current",
        requires = "resistance",
        conflicts_with_all = ["resolution"]
    )]
    nplc: Option<String>,

    #[arg(
        help = "Add a custom message to the CSV file",
        short,
        long,
        aliases = ["msg"],
        value_name = "TEXT",
        conflicts_with_all = ["message_from"],
    )]
    message: Option<String>,

    #[arg(
        help = "Add file content as custom message to the CSV file",
        long,
        aliases = ["msg-from"],
        value_name = "FILE",
        conflicts_with_all = ["message"],
    )]
    message_from: Option<String>,

    #[arg(help = "Beep instrument when logging finished", long)]
    beep: bool,

    #[arg(help = "Performs instrument reset before logging", long)]
    reset: bool,

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

    #[arg(
        help = "Filename to save the CSV lines into.\nIf omitted, lines are written to stdout.",
        value_name = "FILE"
    )]
    output: Option<String>,
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

    pub fn drop_slow_samples(&self) -> bool {
        self.drop_slow_samples
    }

    pub fn reset(&self) -> bool {
        self.reset
    }

    pub fn debug(&self) -> bool {
        self.debug
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn message_from(&self) -> Option<&str> {
        self.message_from.as_deref()
    }

    pub fn host(&self) -> &str {
        self.host.as_ref()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn output(&self) -> Option<&str> {
        self.output.as_deref()
    }

    pub fn sample_period(&self) -> Duration {
        Duration::from_secs_f64(self.rate.map(|f| 1.0 / f).unwrap_or(self.interval))
    }

    pub fn num_samples(&self) -> u32 {
        self.num_samples.unwrap_or(u32::MAX)
    }

    pub fn describe(&self) -> Vec<(String, String)> {
        let mut infos = Vec::new();

        infos.push((
            "Sampling interval".into(),
            format!("{} seconds", self.sample_period().as_secs_f64()),
        ));

        infos.push((
            "Sample rate".into(),
            format!("{} Hz", 1.0 / self.sample_period().as_secs_f64()),
        ));

        if self.drop_slow_samples {
            infos.push(("Drop slow samples".into(), "ON".into()));
        }

        if self.display_off {
            infos.push(("Display".into(), "OFF".into()));
        } else if self.display_text.is_some() {
            infos.push(("Display".into(), "Text".into()));
        }

        let dc_ac = if self.ac { "AC" } else { "DC" };

        if let Some(range) = self.voltage.as_ref() {
            infos.push((format!("{dc_ac}-Voltage"), format!("{range} Volts")));
        } else if let Some(range) = self.current.as_ref() {
            infos.push((format!("{dc_ac}-Current"), format!("{range} Amperes")));
        }

        if let Some(range) = self.resistance.as_ref() {
            let mode = if self.four { "4-wire" } else { "2-wire" };
            infos.push((format!("Resistance ({mode})"), format!("{range} Ohms")));
        }

        if let Some(resolution) = self.resolution.as_ref() {
            infos.push(("Resolution".into(), resolution.clone()));
        }

        if let Some(nplc) = self.nplc.as_ref() {
            infos.push(("NPLC".into(), nplc.to_string()));
        }

        if self.reset {
            infos.push(("Reset instrument".into(), "ON".into()));
        }

        infos
    }

    pub fn configuration_commands(&self) -> Vec<String> {
        let mut configs = Vec::new();

        let dc_ac = if self.ac { "AC" } else { "DC" };
        let res_fres = if self.four { "FRES" } else { "RES" };

        if let Some(volts) = self.voltage.as_ref() {
            configs.push(format!("CONF:VOLT:{dc_ac} {volts}"));
        } else if let Some(amps) = self.current.as_ref() {
            configs.push(format!("CONF:CURR:{dc_ac} {amps}"));
        } else if let Some(ohms) = self.resistance.as_ref() {
            configs.push(format!("CONF:{res_fres} {ohms}"));
        }

        if let Some(resolution) = &self.resolution {
            if self.voltage.is_some() {
                configs.push(format!("VOLT:{dc_ac}:RES {resolution}"));
            } else if self.current.is_some() {
                configs.push(format!("CURR:{dc_ac}:RES {resolution}"));
            } else if self.resistance.is_some() {
                configs.push(format!("{res_fres}:RES {resolution}"));
            }
        };

        if let Some(nplc) = &self.nplc {
            if self.voltage.is_some() {
                configs.push(format!("VOLT:{dc_ac}:NPLC {nplc}"));
            } else if self.current.is_some() {
                configs.push(format!("CURR:{dc_ac}:NPLC {nplc}"));
            } else if self.resistance.is_some() {
                configs.push(format!("{res_fres}:NPLC {nplc}"));
            }
        };

        if self.display_off || self.display_text.is_some() {
            configs.push("DISP OFF".into());
        }

        if let Some(text) = self.display_text.as_ref() {
            let default = format!("DMM Logger ({PKG_VERSION})");
            let text = text.as_deref().unwrap_or(&default);
            configs.push(format!("DISP:TEXT \"{text}\""));
        }

        configs
    }

    pub fn unconfiguration_commands(&self) -> Vec<String> {
        let mut unconfigs = Vec::<String>::new();

        if self.display_text.is_some() {
            unconfigs.push("DISP:TEXT:CLE".into());
        }

        if self.display_off || self.display_text.is_some() {
            unconfigs.push("DISP ON".into());
        }

        if self.beep {
            unconfigs.push("SYST:BEEP".into());
        }

        unconfigs
    }
}
