use pam::{Client, PamError, PamReturnCode};

use crate::errors::{AuthError, NudoResult};

pub fn authenticate_user(user: nix::unistd::User, password: String) -> NudoResult<()> {
    let mut e = Client::with_password("system-auth")?;
    e.conversation_mut().set_credentials(&user.name, &password);
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
