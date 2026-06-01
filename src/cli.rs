use clap::{Args, Parser, Subcommand};
use nix::unistd::{Uid, User};
use serde::{Deserialize, Serialize};

use crate::errors::{AuthError, NudoError, NudoResult};

#[derive(Parser, Deserialize, Serialize, Clone, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Deserialize, Serialize, Subcommand, Clone, Debug)]
pub enum Commands {
    Run(RunArgs),
    CheckConfig,
    Shell {
        #[arg(long, short, default_value_t = String::from("sh"))]
        program: String,
    },
}

#[derive(Args, Debug, Clone, Deserialize, Serialize)]
pub struct RunArgs {
    #[command(flatten)]
    pub user: Target,

    #[arg(long, short, default_value_t = false)] //We dont wanna keep the default env by default.
    pub preserve_env: bool,

    #[arg(required = true, trailing_var_arg = true)]
    pub commands: Vec<String>,
}

#[derive(Args, Debug, Clone, Deserialize, Serialize)]
#[group(required = false, multiple = false)]
pub struct Target {
    #[arg(long)]
    pub user: Option<String>,
    #[arg(long)]
    pub user_id: Option<u32>,
}

impl RunArgs {
    pub fn get_user(&self) -> NudoResult<User> {
        let target = &self.user;
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
            _ => {
                //SAFETY: any remaining states are impossible.
                unsafe { std::hint::unreachable_unchecked() }
            }
        }
    }
}
