use anyhow::{Context, Result};
use clap::Parser;

mod app;
mod cli;
mod csvfile;
mod instrument;
mod scpi;
mod status;

use csvfile::CsvFile;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    cli.validate()?;

    let message_from = read_message_from(cli.message_from())?;
    let message = message_from.as_deref().or_else(|| cli.message());

    let mut dmm = instrument::connect(cli.host(), cli.port())?;
    dmm.set_debug(cli.debug());

    let identification = instrument::identification(&mut dmm)?;

    instrument::configure(&mut dmm, cli.configuration_commands(), cli.reset())?;

    let sample_period = cli.sample_period();
    let num_samples = cli.num_samples();

    let (mut output, bar) = if let Some(filename) = cli.output() {
        (
            CsvFile::create_new(filename)?,
            status::MyProgressBar::new(num_samples),
        )
    } else {
        (CsvFile::stdout(), status::MyProgressBar::none())
    };

    output.write_header(&identification, message)?;

    app::run(
        &mut dmm,
        output,
        sample_period,
        num_samples,
        bar,
        cli.drop_slow_samples(),
    )?;

    instrument::unconfigure(&mut dmm, cli.unconfiguration_commands())?;

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
