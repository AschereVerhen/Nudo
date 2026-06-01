//First, a minimal proof of working.

use clap::Parser;
use miette::IntoDiagnostic;
use nudo::{
    cli::{Cli, Commands},
    config::{NudoConfig, NudoersConfig, parse_config},
    errors::NudoResult,
    execution,
};

fn main() -> miette::Result<()> {
    let cmd = nudo::cli::Cli::parse();
    handle_commands(&cmd).into_diagnostic()?;
    Ok(())
}

fn handle_commands(cmd: &Cli) -> NudoResult<()> {
    match &cmd.command {
        Commands::Run(runargs) => {
            execution::execute(runargs)?;
            Ok(())
        }
        Commands::CheckConfig => {
            parse_config::<NudoersConfig>()?;
            parse_config::<NudoConfig>()?;

            Ok(())
        }
    }
}
