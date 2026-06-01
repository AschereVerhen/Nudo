use crate::errors::{AuthError, NudoResult};
use nix::unistd::User;
use pam::{Client, PamError, PamReturnCode};
use std::io::Write;

fn authenticate_user(user: &nix::unistd::User, password: &str) -> NudoResult<()> {
    let mut e = Client::with_password("system-auth")?;
    e.conversation_mut().set_credentials(&user.name, password);
    match e.authenticate() {
        Ok(_) => (),
        Err(e) => match e {
            PamError(PamReturnCode::Auth_Err) => {
                return Err(crate::errors::NudoError::Auth(
                    AuthError::IncorrectPassword {
                        username: user.name.to_string(),
                    },
                ));
            }
            PamError(PamReturnCode::User_Unknown) => {
                return Err(crate::errors::NudoError::Auth(AuthError::InvalidUser {
                    user: user.name.to_string(),
                }));
            }
            _ => return Err(crate::errors::NudoError::Pam(e)),
        },
    }

    Ok(())
}

fn set_echo(set_value: bool) -> NudoResult<()> {
    let stdin = std::io::stdin();

    let mut term = nix::sys::termios::tcgetattr(&stdin)?;

    if !set_value {
        term.local_flags.remove(nix::sys::termios::LocalFlags::ECHO)
    } else {
        term.local_flags.insert(nix::sys::termios::LocalFlags::ECHO);
    };

    nix::sys::termios::tcsetattr(&stdin, nix::sys::termios::SetArg::TCSANOW, &term)?;

    Ok(())
}

pub fn prompt_for_user_password_and_authenticate(calling_user: &User) -> NudoResult<()> {
    let mut password = String::new();
    print!("Enter Password for {}: ", calling_user.name);
    set_echo(false)?;
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut password)?;
    let password = password.trim().to_string();
    set_echo(true)?;
    println!();
    crate::pam::authenticate_user(calling_user, &password)?;
    Ok(())
}
