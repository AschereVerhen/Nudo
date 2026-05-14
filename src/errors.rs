use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[allow(clippy::result_large_err)]
#[derive(Debug, Error, Diagnostic)]
pub enum NudoError {
    #[error("Pam Error occured.")]
    #[diagnostic(code(nudo::pam::error))]
    Pam(#[from] pam::PamError),

    #[error("Unix error occured")]
    #[diagnostic(code(nudo::unix::error))]
    Unix(#[from] nix::Error),

    #[error("StdLib error occured")]
    #[diagnostic(code(nudo::io::error))]
    Std(#[from] std::io::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Nudo(#[from] InternalError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Auth(#[from] AuthError),

    #[error("Error while parsing the config file")]
    #[diagnostic(code(nudo::config::error))]
    Config(#[from] ConfigError),
}

#[derive(Debug, Error, Diagnostic)]
pub enum AuthError {
    #[error("The User `{user}` is not a valid user.")]
    #[diagnostic(
        code(nudo::auth::invalid_user),
        help("Check /etc/passwd or try a different user.")
    )]
    InvalidUser { user: String },

    #[error("The userid `{userid}` does not represent a valid user.")]
    #[diagnostic(
        code(nudo::auth::invalid_userid),
        help("Check /etc/passwd or try another user id.")
    )]
    InvalidUserId { userid: u32 },

    #[error("Password entered for `{username}` was incorrect")]
    #[diagnostic(
        code(nudo::auth::incorrect_password),
        help("Please enter the correct password and try again.")
    )]
    IncorrectPassword { username: String },
}

#[derive(Debug, Error, Diagnostic)]
pub enum InternalError {
    #[error("An invariant was violated due to an internal bug")]
    #[diagnostic(
        code(nudo::internal_error),
        help(
            "Please try to update nudo; or else if on latest release please open an issue on the url"
        ),
        url("https://github.com/AschereVerhen/Nudo")
    )]
    InvalidInvariant,
}

//InvalidInvariant helpers
pub trait OptionExt<T> {
    fn required(self) -> Result<T, NudoError>;
}

impl<T> OptionExt<T> for Option<T> {
    fn required(self) -> Result<T, NudoError> {
        self.ok_or(NudoError::Nudo(InternalError::InvalidInvariant))
    }
}

#[cold]
#[inline(never)]
pub fn invalid_invariant<T>() -> NudoResult<T> {
    Err(NudoError::Nudo(InternalError::InvalidInvariant))
}
#[macro_export]
macro_rules! invalid_invariant {
    () => {
        $crate::errors::invalid_invariant()
    };
}

//ConfigError
#[derive(Error, Debug, Diagnostic)]
#[error("Failed to parse the config")]
pub struct ConfigError {
    #[source_code]
    src: NamedSource<String>,

    #[label]
    span: SourceSpan,

    #[source]
    err: Box<toml::de::Error>, //Decrease the struct size on the stack
}

impl NudoError {
    pub fn from_toml_error(
        source_name: impl AsRef<str>,
        source: String,
        err: toml::de::Error,
    ) -> Self {
        let span = err
            .span()
            .map(|r| (r.start, r.end - r.start).into())
            .unwrap_or((0, 0).into());
        Self::Config(ConfigError {
            span,
            src: NamedSource::new(source_name, source),
            err: Box::new(err),
        })
    }
}

pub type NudoResult<T> = Result<T, NudoError>; //Ease of use
