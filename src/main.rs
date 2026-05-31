//First, a minimal proof of working.
#![allow(unused)]
use std::io::Write;

use clap::Parser;
use miette::IntoDiagnostic;
use nudo::{
    cli::{Cli, Commands},
    config::{NudoConfig, NudoersConfig, parse_config},
    errors::NudoResult,
    execution,
    priviledges::drop_privs_temp,
};

fn main() -> miette::Result<()> {
    let calling_user = nudo::get_calling_user().into_diagnostic()?;
    println!("{:#?}", parse_config::<NudoersConfig>());
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
            todo!()
        }
        Commands::Shell { program } => {
            todo!()
        }
        Commands::DryRun => {
            todo!()
        }
    }
}
