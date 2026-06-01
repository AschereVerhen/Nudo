use serde::{Deserialize, Serialize};

use std::{collections::HashMap, path::PathBuf};

use crate::{
    NUDO_CONFIG, USER_CONFIG,
    errors::{ConfigError, NudoError, NudoResult},
};

mod private {
    pub trait Sealed {}
}
pub trait NudoConfigurationStruct: serde::de::DeserializeOwned + Default + private::Sealed {
    fn path() -> PathBuf;
}
/*
    We want users to manually create the toml file. considering its not that hard to do so. But we will provide a basic file,
    with root and group.nudo
*/

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct NudoersConfig {
    pub users: HashMap<String, Config>,
    pub groups: Option<HashMap<String, Config>>,
}
impl private::Sealed for NudoersConfig {}
impl NudoConfigurationStruct for NudoersConfig {
    fn path() -> PathBuf {
        (*USER_CONFIG).clone()
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    //Example usage:
    pub rules: Vec<CommandRule>,
    pub password_required: bool,
    //Optionally one can add an env.
    pub keep_env: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct CommandRule {
    pub command: PathBuf,
    // Supports either a single catch-all regex or a precise positional string sequence matrix
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
}

fn default_path_value() -> Vec<PathBuf> {
    //Since sudo uses: "/usr/local/sbin:/usr/local/bin:/usr/bin" as its default path var,
    //we will do the same.
    vec![
        "/usr/bin".into(),
        "/usr/local/sbin".into(),
        "/usr/local/bin".into(),
    ]
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct NudoConfig {
    #[serde(default = "default_path_value")]
    pub path: Vec<PathBuf>,
}
impl private::Sealed for NudoConfig {}
impl NudoConfigurationStruct for NudoConfig {
    fn path() -> PathBuf {
        (*NUDO_CONFIG).clone()
    }
}

pub fn parse_config<T: NudoConfigurationStruct>() -> NudoResult<T> {
    let path = T::path();

    if !path.exists() {
        return Err(NudoError::Config(ConfigError::ConfigNotFound {
            missing_config: path.display().to_string(),
        }));
    }

    let contents = std::fs::read_to_string(&path)?;
    let contents = contents.trim();
    if contents.is_empty() {
        return Ok(T::default());
    }
    let config: Result<T, toml::de::Error> = toml::from_str(contents);

    match config {
        Ok(conf) => Ok(conf),
        Err(err) => Err(NudoError::from_toml_error(
            path.display().to_string(),
            contents.to_string(),
            err,
        )),
    }
}
