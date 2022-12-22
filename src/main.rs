use anyhow::{Context, Result};
use clap::Parser;

mod app;
mod cli;
mod csvfile;
mod instrument;
mod scpi;

use csvfile::CsvFile;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    cli.validate()?;

    let message_from = read_message_from(cli.message_from())?;
    let message = message_from.as_deref().or_else(|| cli.message());

    let mut dmm = instrument::connect(cli.host(), cli.port())?;
    dmm.set_debug(cli.debug());

    let identification = instrument::identification(&mut dmm)?;

    instrument::configure(&mut dmm, cli.scpi_commands(), cli.reset())?;

    let mut output = if let Some(filename) = cli.output() {
        CsvFile::create_new(filename)?
    } else {
        CsvFile::stdout()
    };

    output.write_header(&identification, message)?;

    app::run(&mut dmm, output, cli.sample_period(), cli.num_samples())?;

    if cli.beep() {
        instrument::beep(&mut dmm)?;
    }

    instrument::disconnect(dmm)
}

fn read_message_from(path: Option<&str>) -> Result<Option<String>> {
    path.map(std::fs::read_to_string)
        .transpose()
        .with_context(|| {
            format!(
                "Reading message content from file `{}` failed",
                path.unwrap()
            )
        })
}
