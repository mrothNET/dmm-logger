use anyhow::Result;
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

    let mut dmm = instrument::connect(cli.host(), cli.port())?;
    dmm.set_debug(cli.debug());

    let identification = instrument::identification(&mut dmm)?;

    instrument::configure(&mut dmm, cli.scpi_commands())?;

    let mut output = if let Some(filename) = cli.output() {
        CsvFile::create_new(filename)?
    } else {
        CsvFile::stdout()
    };

    output.write_header(&identification)?;

    app::run(dmm, output, cli.sample_period(), cli.num_samples())
}
