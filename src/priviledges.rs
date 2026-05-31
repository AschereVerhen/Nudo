use nix::unistd::{self, Gid, Uid};

use crate::errors::NudoResult;

pub fn drop_privs_temp(target_user: Uid, target_group: Gid) -> NudoResult<()> {
    unistd::setegid(target_group)?;
    unistd::seteuid(target_user)?;

    Ok(())
}

///Change the gid/uid to something else.
pub fn chpriv(
    target_user: Uid,
    target_group: Gid,
    clear_supplementary_groups: bool,
) -> NudoResult<()> {
    if clear_supplementary_groups {
        unistd::setgroups(&[])?;
    }
    unistd::setgid(target_group)?;
    unistd::setuid(target_user)?;

    Ok(())
}
