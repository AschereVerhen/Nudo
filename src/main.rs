//First, a minimal proof of working.

use clap::Parser;
use miette::IntoDiagnostic;
use nudo::{
    cli::{Cli, Commands, RunArgs, Target},
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
        Commands::Shell { program } => {
            let runargs = RunArgs {
                commands: vec![program.clone(), "-l".to_string()],
                preserve_env: false,
                user: Target {
                    user_id: Some(0),
                    user: None,
                },
            };
            execution::execute(&runargs)?;

            Ok(())
        }
    }
}
