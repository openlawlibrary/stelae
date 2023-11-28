use anyhow::{bail, Result};
use lazy_static::lazy_static;
use std::path::{Path, PathBuf};
use stelae::utils::paths::fix_unc_path;

lazy_static! {
    static ref SCRIPT_PATH: PathBuf = {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/fixtures/scripts");
        path
    };
}

pub fn execute_script(script_name: &str, mut script_result_directory: PathBuf) -> Result<()> {
    let script_absolute_path = fix_unc_path(&SCRIPT_PATH.join(script_name).canonicalize()?);
    let env_path = std::env::current_dir()?.join(script_name);
    let mut cmd = std::process::Command::new(&script_absolute_path);
    let output = match configure_command(&mut cmd, &script_result_directory).output() {
        Ok(out) => out,
        Err(err)
            if err.kind() == std::io::ErrorKind::PermissionDenied || err.raw_os_error() == Some(193) /* windows */ =>
        {
            cmd = std::process::Command::new("bash");
            let output = configure_command(cmd.arg(&script_absolute_path), &script_result_directory).output().unwrap();
            output
        }
        Err(err) => return Err(err.into()),
    };
    Ok(())
}

fn configure_command<'a>(
    cmd: &'a mut std::process::Command,
    script_result_directory: &Path,
) -> &'a mut std::process::Command {
    let never_path = if cfg!(windows) { "-" } else { ":" };
    dbg!(&cmd);
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir(script_result_directory)
        .env_remove("GIT_DIR")
        .env_remove("GIT_ASKPASS")
        .env_remove("SSH_ASKPASS")
        .env("GIT_CONFIG_SYSTEM", never_path)
        .env("GIT_CONFIG_GLOBAL", never_path)
        .env("GIT_TERMINAL_PROMPT", "false")
        .env("GIT_AUTHOR_DATE", "2000-01-01 00:00:00 +0000")
        .env("GIT_AUTHOR_EMAIL", "author@openlawlib.org")
        .env("GIT_AUTHOR_NAME", "author")
        .env("GIT_COMMITTER_DATE", "2000-01-02 00:00:00 +0000")
        .env("GIT_COMMITTER_EMAIL", "committer@openlawlib.org")
        .env("GIT_COMMITTER_NAME", "committer")
}
