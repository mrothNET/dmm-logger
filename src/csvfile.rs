use std::fs::File;
use std::io::{stdout, BufWriter, Write};

use anyhow::{Context, Ok, Result};
use chrono::prelude::*;

use crate::scpi::Identification;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct CsvFile {
    filename: Option<String>,
    output: BufWriter<Box<dyn Write>>,
}

impl CsvFile {
    pub fn stdout() -> CsvFile {
        let output: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout()));
        let filename = None;

        CsvFile { filename, output }
    }

    pub fn create_new(filename: &str) -> Result<CsvFile> {
        let output: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(
            File::options()
                .create_new(true)
                .write(true)
                .open(filename)
                .with_context(|| format!("Creating CSV file '{filename}' failed"))?,
        ));

        let filename = Some(filename.into());

        Ok(CsvFile { filename, output })
    }

    pub fn write_header(&mut self, ident: &Identification, msg: Option<&str>) -> Result<()> {
        (|| {
            writeln!(
                self.output,
                "# File created by DMM logger ({PKG_VERSION}) on {}",
                Local::now().format("%Y-%m-%d %H:%M:%S (%Z)")
            )?;
            writeln!(self.output, "#")?;


            if let Some(msg) = msg {
                let msg = msg.trim();

                if let Some(max_len) = msg.lines().map(|line| line.trim_end().len()).max() {
                    let hline = "-".repeat(max_len);

                    writeln!(self.output, "# {hline}")?;
                    for line in msg.lines() {
                        writeln!(self.output, "# {}", line.trim_end())?;
                    }
                    writeln!(self.output, "# {hline}")?;
                    writeln!(self.output, "#")?;
                }
            }

            writeln!(self.output, "# Instrument")?;
            writeln!(self.output, "# ----------")?;
            writeln!(self.output, "# Manufacturer : {}", ident.manufacturer)?;
            writeln!(self.output, "# Model        : {}", ident.model)?;
            writeln!(self.output, "# Serial       : {}", ident.serial)?;
            writeln!(self.output, "# Firmware     : {}", ident.firmware)?;
            writeln!(self.output, "#")?;

            writeln!(self.output, "# Fields")?;
            writeln!(self.output, "# ------")?;
            writeln!(self.output, "# sequence     : Sequential sample number starting at 0")?;
            writeln!(self.output, "# date         : Local date of measurement: YEAR-MONTH-DAY")?;
            writeln!(self.output, "# time         : Local clock time of measurement: HOURS:MINUTES:SECONDS.MILLISECONDS")?;
            writeln!(self.output, "# moment       : Time in seconds since first measurement")?;
            writeln!(self.output, "# delay        : Delay of the measurement in seconds, caused by non-real-time behavior or fast logging rate")?;
            writeln!(self.output, "# latency      : Measurement duration in seconds including network roundtrip time")?;
            writeln!(self.output, "# reading      : Measured value returned from instrument")?;
            writeln!(self.output, "#")?;

            writeln!(
                self.output,
                "sequence,date,time,moment,delay,latency,reading"
            )?;

            self.output.flush()
        })()
        .with_context(|| {
            if let Some(filename) = self.filename.as_deref() {
                format!("Writing headers to CSV file '{filename}' failed")
            } else {
                "Writing CSV headers to stdout failed".into()
            }
        })
    }

    pub fn write_line(
        &mut self,
        sequence: u32,
        datetime: DateTime<Local>,
        moment: f64,
        delay: f64,
        latency: f64,
        reading: f64,
    ) -> Result<()> {
        (|| {
            let date = datetime.format("%Y-%m-%d");
            let time = datetime.format("%H:%M:%S.%3f");

            writeln!(
                self.output,
                "{sequence},{date},{time},{moment},{delay},{latency},{reading}"
            )?;

            self.output.flush()
        })()
        .with_context(|| {
            if let Some(filename) = self.filename.as_deref() {
                format!("Writing data to CSV file '{filename}' failed")
            } else {
                "Writing data to stdout failed".into()
            }
        })
    }
}
