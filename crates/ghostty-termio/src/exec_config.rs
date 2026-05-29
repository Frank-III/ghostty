//! Build [`CommandSpec`] from derived termio config (`Exec.zig` env/cwd/shell subset).

use std::path::Path;

use ghostty_config::{DerivedTermioConfig, ShellIntegration};

use crate::command::{CommandBuildError, CommandBuilder, CommandSpec};
use crate::shell::ShellCommandBuilder;
use crate::shell_integration::{apply_shell_integration, ShellIntegrationContext};

const DEFAULT_SHELL: &str = "/bin/sh";

/// Resolve the PTY child command from derived termio config.
pub fn command_from_termio_config(
    cfg: &DerivedTermioConfig,
    shell_integration: ShellIntegration,
    resources_dir: Option<&Path>,
) -> Result<CommandSpec, CommandBuildError> {
    let mut builder = if let Some(cmd) = cfg.command.as_deref() {
        CommandBuilder::new()
            .path(DEFAULT_SHELL)
            .arg("sh")
            .arg("-c")
            .arg(cmd)
    } else {
        CommandBuilder::new().path(DEFAULT_SHELL).arg("sh")
    };

    builder = builder.env("TERM", &cfg.term);
    for (key, value) in &cfg.env {
        builder = builder.env(key, value);
    }
    if let Some(cwd) = &cfg.working_directory {
        builder = builder.cwd(cwd);
    }

    if cfg.wait_after_command {
        let mut shell_cmd = ShellCommandBuilder::new();
        if let Some(cmd) = cfg.command.as_deref() {
            shell_cmd.append_arg(cmd);
            shell_cmd.append_arg(";");
        }
        shell_cmd.append_arg("exec");
        shell_cmd.append_arg(DEFAULT_SHELL);
        let wrapped = shell_cmd.as_str();
        builder = CommandBuilder::new()
            .path(DEFAULT_SHELL)
            .arg("sh")
            .arg("-c")
            .arg(wrapped);
        builder = builder.env("TERM", &cfg.term);
        for (key, value) in &cfg.env {
            builder = builder.env(key, value);
        }
        if let Some(cwd) = &cfg.working_directory {
            builder = builder.cwd(cwd);
        }
    }

    let mut spec = builder.build()?;
    apply_shell_integration(
        &mut spec,
        &ShellIntegrationContext::new(shell_integration, resources_dir),
    );
    Ok(spec)
}

/// True when exit code should be treated as abnormal per config runtime threshold.
pub fn is_abnormal_exit(cfg: &DerivedTermioConfig, exit_code: u32, elapsed_ms: u32) -> bool {
    elapsed_ms <= cfg.abnormal_command_exit_runtime && exit_code != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use ghostty_config::Config;
    use std::ffi::OsStr;

    #[test]
    fn term_env_set() {
        let cfg = DerivedTermioConfig::from(&Config::with_defaults());
        let spec =
            command_from_termio_config(&cfg, ShellIntegration::default(), None).expect("spec");
        let env = spec.env.as_ref().expect("env");
        assert_eq!(
            env.get(&std::ffi::OsString::from("TERM"))
                .map(|s| s.to_str()),
            Some(Some("xterm-ghostty"))
        );
    }

    #[test]
    fn working_directory_applied() {
        let mut cfg = Config::with_defaults();
        cfg.working_directory = Some("/tmp".to_string());
        let derived = DerivedTermioConfig::from(&cfg);
        let spec =
            command_from_termio_config(&derived, ShellIntegration::default(), None).expect("spec");
        assert_eq!(spec.cwd.as_deref(), Some(std::path::Path::new("/tmp")));
    }

    #[test]
    fn zsh_integration_sets_zdotdir() {
        let tmp = std::env::temp_dir().join(format!("ghostty-exec-{}", std::process::id()));
        std::fs::create_dir_all(tmp.join("shell-integration/zsh")).unwrap();
        let mut cfg = Config::with_defaults();
        cfg.shell_integration = ShellIntegration::Zsh;
        let derived = DerivedTermioConfig::from(&cfg);
        let spec =
            command_from_termio_config(&derived, cfg.shell_integration, Some(&tmp)).expect("spec");
        let env = spec.env.as_ref().expect("env");
        assert!(env.contains_key(OsStr::new("ZDOTDIR")));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
