use std::fs::File;
use std::io::{stdout, BufWriter, Write};

use anyhow::{Context, Ok, Result};
use chrono::prelude::*;
use unicode_segmentation::UnicodeSegmentation;

use crate::scpi::Identification;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct CsvFile {
    filename: Option<String>,
    output: BufWriter<Box<dyn Write>>,
    width: usize,
}

impl CsvFile {
    pub fn stdout() -> CsvFile {
        let output: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout()));
        let filename = None;

        CsvFile {
            filename,
            output,
            width: 0,
        }
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

        Ok(CsvFile {
            filename,
            output,
            width: 0,
        })
    }

    pub fn write_header(
        &mut self,
        settings: &Vec<(String, String)>,
        ident: &Identification,
        user_message: Option<&str>,
    ) -> Result<()> {
        self.ensure_width(
            settings
                .iter()
                .map(|(label, _)| label.len())
                .max()
                .unwrap_or(0),
        );

        self.ensure_width("Manufacturer".len());

        (|| {
            self.write_title()?;
            self.write_user_message(user_message)?;
            self.write_settings_description(settings)?;
            self.write_instrument_identification(ident)?;
            self.write_column_description()?;
            self.write_column_headers()?;
            self.output.flush()?;
            Ok(())
        })()
        .with_context(|| {
            if let Some(filename) = self.filename.as_deref() {
                format!("Writing headers to CSV file '{filename}' failed")
            } else {
                "Writing CSV headers to stdout failed".into()
            }
        })
    }

    pub fn write_reading(
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
                "{sequence},{date},{time},{moment:.4},{delay:.4},{latency:.4},{reading}"
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

    pub fn write_comment<T: std::fmt::Display>(&mut self, comment: T) -> Result<()> {
        writeln!(self.output, "# {comment}")?;
        Ok(())
    }

    fn ensure_width(&mut self, width: usize) {
        self.width = std::cmp::max(self.width, width);
    }

    fn write_title(&mut self) -> Result<()> {
        writeln!(
            self.output,
            "# File created by DMM logger ({PKG_VERSION}) on {}",
            Local::now().format("%Y-%m-%d %H:%M:%S (%Z)")
        )?;
        writeln!(self.output, "#")?;
        Ok(())
    }

    fn write_user_message(&mut self, msg: Option<&str>) -> Result<()> {
        if let Some(msg) = msg {
            let msg = msg.trim();

            if let Some(max_len) = msg
                .lines()
                .map(|line| line.trim_end().graphemes(true).count())
                .max()
            {
                let hline = "-".repeat(max_len);

                writeln!(self.output, "# {hline}")?;

                for line in msg.lines() {
                    writeln!(self.output, "# {}", line.trim_end())?;
                }

                writeln!(self.output, "# {hline}")?;
                writeln!(self.output, "#")?;
            }
        }

        Ok(())
    }

    fn write_settings_description(&mut self, description: &Vec<(String, String)>) -> Result<()> {
        if !description.is_empty() {
            writeln!(self.output, "# Settings")?;
            writeln!(self.output, "# --------")?;

            for (label, value) in description {
                self.write_label_value(label, value)?;
            }

            writeln!(self.output, "#")?;
        }

        Ok(())
    }

    fn write_instrument_identification(&mut self, ident: &Identification) -> Result<()> {
        writeln!(self.output, "# Instrument")?;
        writeln!(self.output, "# ----------")?;

        self.write_label_value("Manufacturer", &ident.manufacturer)?;
        self.write_label_value("Model", &ident.model)?;
        self.write_label_value("Serial", &ident.serial)?;
        self.write_label_value("Firmware", &ident.firmware)?;

        writeln!(self.output, "#")?;

        Ok(())
    }

    fn write_column_description(&mut self) -> Result<()> {
        writeln!(self.output, "# Fields")?;
        writeln!(self.output, "# ------")?;

        self.write_label_value("sequence", "Sequential sample number starting at 0")?;

        self.write_label_value("date", "Local date of measurement: YEAR-MONTH-DAY")?;

        self.write_label_value(
            "time",
            "Local clock time of measurement: HOURS:MINUTES:SECONDS.MILLISECONDS",
        )?;

        self.write_label_value("moment", "Time in seconds since first measurement")?;

        self.write_label_value(
            "delay",
            "Delay of the measurement in seconds, caused by non-real-time behavior or fast logging rate"
        )?;

        self.write_label_value(
            "latency",
            "Measurement duration in seconds including network roundtrip time",
        )?;

        self.write_label_value("reading", "Measured value returned from instrument")?;

        writeln!(self.output, "#")?;

        Ok(())
    }

    pub fn write_column_headers(&mut self) -> Result<()> {
        writeln!(
            self.output,
            "sequence,date,time,moment,delay,latency,reading"
        )?;
        Ok(())
    }

    fn write_label_value<T1, T2>(&mut self, label: T1, value: T2) -> Result<()>
    where
        T1: std::fmt::Display,
        T2: std::fmt::Display,
    {
        let width = self.width;
        writeln!(self.output, "# {label:width$} : {value}")?;
        Ok(())
    }
}
