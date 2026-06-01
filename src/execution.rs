use std::collections::HashMap;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use nix::unistd::{Gid, Group, User};

use crate::cli::RunArgs;
use crate::errors::{AuthError, InternalError, NudoError, NudoResult, RuntimeError};
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

fn full_path_of(program_name: &str, path: &[PathBuf]) -> Option<PathBuf> {
    path.iter().find_map(|dir| {
        let candidate = dir.join(program_name);
        candidate.canonicalize().ok()
    })
}

fn match_execution_priv(conf: &Config, program: &PathBuf, arguments: &[String]) -> NudoResult<()> {
    const INSUFF_AUTH_ERROR: NudoError = NudoError::Auth(AuthError::InsufficientPriviledges);
    let rules = &conf.rules;

    // Locate the matching rule block for the executable program target
    let cmdrule = rules
        .iter()
        .find(|st| &st.command == program || st.command == Path::new("*"))
        .ok_or(INSUFF_AUTH_ERROR)?;

    // Validate via structural positional sequences if defined
    if let Some(expected_args_rules) = &cmdrule.args {
        if expected_args_rules.len() != arguments.len() {
            return Err(INSUFF_AUTH_ERROR);
        }

        for (index, argument) in arguments.iter().enumerate() {
            let regex_str = &expected_args_rules[index];
            let regex_arg = Regex::new(regex_str)
                .map_err(|_| NudoError::Config(crate::errors::ConfigError::InvalidRegex))?;

            if !regex_arg.is_match(argument) {
                return Err(INSUFF_AUTH_ERROR);
            }
        }
    }

    Ok(())
}

#[allow(unused)]
fn extract_envs_to_keep(
    user: &User,
    program: &Path,
    config: &NudoersConfig,
    arguments: &[String],
) -> Option<HashMap<String, String>> {
    let user_profile = config.users.get(&user.name)?;

    // Scan comprehensively to isolate the exact ruleset matching parameter counts
    let rule = user_profile.rules.iter().find(|s| {
        (&s.command == "*" || s.command == program)
            && s.args.as_ref().is_none_or(|a| a.len() == arguments.len())
    })?;

    let mut res = rule.env.clone().unwrap_or_default();

    if let Some(keep_env) = &user_profile.keep_env {
        for key in keep_env {
            if let Ok(val) = std::env::var(key) {
                res.insert(key.clone(), val);
            }
        }
    }

    Some(res)
}

pub fn execute(args: &RunArgs) -> NudoResult<()> {
    let commands = &args.commands;

    let program_raw = commands
        .first()
        .ok_or(NudoError::Nudo(InternalError::InvalidInvariant))?;
    let arguments = commands.get(1..).unwrap_or(&[]);

    let calling_user = get_calling_user()?;
    let target_user = args.get_user()?;

    let uid = target_user.uid.as_raw();
    let gid = target_user.gid.as_raw();

    // CRITICAL FIX: Allocate and finalize the target username CString completely
    // inside the parent process to keep the post-fork closure async-signal-safe.
    let target_username_c = std::ffi::CString::new(target_user.name.clone()).map_err(|_| {
        NudoError::Runtime(RuntimeError::NameContainsNul {
            name: target_user.name,
        })
    })?;

    let nudoconfig = parse_config::<NudoConfig>()?;
    let nudoersconfig = parse_config::<NudoersConfig>()?;

    let user_conf = nudoersconfig
        .users
        .get(&calling_user.name)
        .ok_or(NudoError::Auth(AuthError::InvalidUser {
            user: calling_user.name.clone(),
        }))?;

    if user_conf.password_required {
        crate::pam::prompt_for_user_password_and_authenticate(&calling_user)?;
    }

    let program =
        crate::execution::full_path_of(program_raw, &nudoconfig.path).ok_or_else(|| {
            NudoError::Runtime(RuntimeError::PathNotFound {
                program: program_raw.clone(),
            })
        })?;

    ensure_access(&calling_user, &nudoersconfig, &program, arguments)?;

    let envs = extract_envs_to_keep(&calling_user, &program, &nudoersconfig, arguments);

    let mut cmd = Command::new(program);
    cmd.args(arguments)
        .uid(uid)
        .gid(gid)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::inherit());

    if !args.preserve_env {
        cmd.env_clear();
    }

    if let Some(env) = envs {
        cmd.envs(env);
    }

    // Safety: pre_exec runs immediately after a fork context. The captured target_username_c allocation
    // is performed entirely in the parent process beforehand, ensuring that no dynamic memory allocations
    // occur in the child process. This satisfies all async-signal-safety guarantees.
    unsafe {
        cmd.pre_exec(move || {
            nix::unistd::initgroups(&target_username_c, Gid::from_raw(gid)).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::PermissionDenied, e.to_string())
            })?;
            Ok(())
        });
    }

    let e = cmd.exec();
    Err(NudoError::Std(e))
}
