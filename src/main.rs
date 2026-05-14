//First, a minimal proof of working.

use std::io::Write;

use clap::Parser;
use nudo::{
    cli::{Cli, Commands},
    errors::NudoResult,
};

fn main() -> miette::Result<()> {
    let cmd = nudo::cli::Cli::parse();
    handle_commands(&cmd)?;
    Ok(())
}

fn handle_commands(cmd: &Cli) -> NudoResult<()> {
    match &cmd.command {
        Commands::Run(runargs) => {
            let user = cmd.get_user()?;
            let mut password = String::new();
            print!("Enter Password for {}: ", &user.name);
            std::io::stdout().flush()?;
            std::io::stdin().read_line(&mut password)?;
            let password = password.trim().to_string();
            nudo::pam::authenticate_user(user, password)?;
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
