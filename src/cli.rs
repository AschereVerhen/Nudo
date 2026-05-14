use clap::{Args, Parser, Subcommand};
use nix::unistd::{Uid, User};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{AuthError, NudoError, NudoResult},
    invalid_invariant,
};

#[derive(Parser, Deserialize, Serialize, Clone, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn get_user(&self) -> NudoResult<User> {
        let commands = &self.command;
        match commands {
            Commands::Run(s) => {
                let target = &s.user;
                match (&target.user, &target.user_id) {
                    (Some(name), None) => {
                        User::from_name(name)?.ok_or(NudoError::Auth(AuthError::InvalidUser {
                            user: name.clone(),
                        }))
                    }
                    (None, Some(uid)) => User::from_uid(Uid::from_raw(*uid))?
                        .ok_or(NudoError::Auth(AuthError::InvalidUserId { userid: *uid })),
                    (None, None) => User::from_uid(Uid::from_raw(0))?
                        .ok_or(NudoError::Auth(AuthError::InvalidUserId { userid: 0 })),
                    _ => unreachable!(),
                }
            }
            _ => invalid_invariant!(),
        }
    }
}

#[derive(Deserialize, Serialize, Subcommand, Clone, Debug)]
pub enum Commands {
    Run(RunArgs),
    CheckConfig,
    Shell {
        #[arg(long, short, default_value_t = String::from("sh"))]
        program: String,
    },
    DryRun,
}

#[derive(Args, Debug, Clone, Deserialize, Serialize)]
pub struct RunArgs {
    #[command(flatten)]
    user: Target,

    #[arg(long, short, default_value_t = false)] //We dont wanna keep the default env by default.
    preserve_env: bool,

    #[arg(required = true, trailing_var_arg = true)]
    commands: Vec<String>,
}

#[derive(Args, Debug, Clone, Deserialize, Serialize)]
#[group(required = false, multiple = false)]
struct Target {
    #[arg(long)]
    user: Option<String>,
    #[arg(long)]
    user_id: Option<u32>,
}
