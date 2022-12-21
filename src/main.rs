use anyhow::Result;
use clap::Parser;

mod app;
mod cli;
mod csvfile;
mod instrument;
mod scpi;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    cli.validate()?;

    let mut dmm = instrument::connect(cli.host(), cli.port())?;
    dmm.set_debug(cli.debug());

    instrument::configure(&mut dmm, cli.scpi_commands())?;

    app::run(&mut dmm, cli.sample_period(), cli.num_samples())?;

    dmm.close()?;
    Ok(())
}
