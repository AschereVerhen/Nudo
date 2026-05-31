use std::collections::HashMap;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use nix::unistd::{Group, User};

use crate::cli::RunArgs;
use crate::errors::{AuthError, InternalError, NudoError, NudoResult};
use crate::{config::*, get_calling_user};
use regex::Regex;
//
fn ensure_access(
    user: &User,
    config: &NudoersConfig,
    program: &PathBuf,
    arguments: &[String],
) -> NudoResult<()> {
    let user_group =
        Group::from_gid(user.gid)?.ok_or(NudoError::Auth(AuthError::InvalidGroup {
            group_id: user.gid.as_raw(),
        }))?;
    //extracting the regex string from config:
    let user_config = config.users.get(&user.name);
    let group_config = &config.groups;

    match (user_config, group_config) {
        (None, None) => return Err(NudoError::Auth(AuthError::InsufficientPriviledges)),
        (Some(conf), _) => match_execution_priv(conf, program, arguments)?,
        (_, Some(conf)) => {
            let group_config = conf.get(&user_group.name);
            if let Some(conf) = group_config {
                match_execution_priv(conf, program, arguments)?;
            } else {
                return Err(NudoError::Auth(AuthError::InsufficientPriviledges));
            }
        }
    }
    Ok(())
}

fn match_execution_priv(conf: &Config, program: &PathBuf, arguments: &[String]) -> NudoResult<()> {
    const INSUFF_AUTH_ERROR: NudoError = NudoError::Auth(AuthError::InsufficientPriviledges);
    let rules = &conf.rules;

    let cmdrule = rules
        .iter()
        .find(|st| &st.command == program || &st.command == "*")
        .ok_or(INSUFF_AUTH_ERROR)?;
    let regex_arg = Regex::new(&cmdrule.args)
        .map_err(|_| NudoError::Config(crate::errors::ConfigError::InvalidRegex))?;
    let success = regex_arg.is_match(&arguments.join(""));

    if !success {
        return Err(INSUFF_AUTH_ERROR);
    }

    Ok(())
}

fn full_path_of(program_name: &str, path: &[PathBuf]) -> Option<PathBuf> {
    path.iter().find_map(|dir| {
        let candidate = dir.join(program_name);

        if candidate.exists() {
            candidate.canonicalize().ok()
        } else {
            None
        }
    })
}
#[allow(unused)]
fn extract_envs_to_keep(
    user: &User,
    program: &Path,
    config: &NudoersConfig,
) -> Option<HashMap<String, String>> {
    let user = config.users.get(&user.name)?;
    let rule = user
        .rules
        .iter()
        .filter(|s| &s.command == "*" || s.command == program)
        .collect::<Vec<&CommandRule>>();
    if rule.len() != 1 {
        return None;
    }
    let rule = rule[0];
    let envs = &rule.env;
    let res = envs.clone()?;

    Some(res)
}

pub fn execute(args: &RunArgs) -> NudoResult<()> {
    let commands = &args.commands;

    //Safety: Clap prevents empty command vecs because required was enforced.
    let program = commands
        .first()
        .ok_or(NudoError::Nudo(InternalError::InvalidInvariant))?;
    let arguments = commands.get(1..).unwrap_or(&[]);
    let calling_user = get_calling_user()?;
    let target_user = args.get_user()?;
    let uid = target_user.uid.as_raw();
    let gid = target_user.gid.as_raw();

    let nudoconfig = parse_config::<NudoConfig>()?;
    let nudoersconfig = parse_config::<NudoersConfig>()?;
    let path = &nudoconfig.path;

    let program = full_path_of(program, path).unwrap(); //TODO: Better error handling

    ensure_access(&calling_user, &nudoersconfig, &program, arguments)?;

    let envs = extract_envs_to_keep(&calling_user, &program, &nudoersconfig);

    let mut password = String::new();
    print!("Enter Password for {}: ", &calling_user.name);
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut password)?;
    let password = password.trim().to_string();
    crate::pam::authenticate_user(calling_user, password)?;

    let mut program = Command::new(program);
    program
        .args(arguments)
        .uid(uid)
        .gid(gid)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::inherit());

    if !args.preserve_env {
        program.env_clear();
    }

    if let Some(env) = envs {
        program.envs(env);
    }
    let e = program.exec();

    //If the above exec returns, it means that an error occured
    Err(NudoError::Std(e))
}
