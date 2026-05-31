#![allow(clippy::result_large_err)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::undocumented_unsafe_blocks)]
use std::{path::PathBuf, sync::LazyLock};

use nix::unistd::{Uid, User};

use crate::errors::{AuthError, NudoError, NudoResult};

pub mod cli;
pub mod config;
pub mod errors;
pub mod execution;
pub mod pam;
pub mod priviledges;

pub static USER_CONFIG: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from("/etc/nudo.d/nudoers.toml"));
pub static NUDO_CONFIG: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from("/etc/nudo.d/nudo.toml"));

pub fn get_calling_user() -> NudoResult<User> {
    let e = nix::unistd::getresuid()?;
    let userid = e.real;

    let user = User::from_uid(userid)?;

    #[cold]
    #[inline(never)]
    fn unlikely_failure(userid: Uid) -> NudoError {
        NudoError::Auth(AuthError::InvalidUserId {
            userid: userid.as_raw(),
        })
    }

    user.ok_or_else(|| unlikely_failure(userid))
}
