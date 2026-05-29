//! Build subprocess specs from application config.

use std::path::Path;

use ghostty_config::{Config, DerivedTermioConfig};
use ghostty_termio::{command_from_termio_config, CommandBuildError, CommandSpec};

/// Resolve the PTY child command from config (`command` key or default shell).
pub fn command_from_config(
    config: &Config,
    resources_dir: Option<&Path>,
) -> Result<CommandSpec, CommandBuildError> {
    command_from_termio_config(
        &DerivedTermioConfig::from(config),
        config.shell_integration,
        resources_dir,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ghostty_config::Config;

    #[test]
    fn default_is_shell() {
        let spec = command_from_config(&Config::with_defaults(), None).expect("spec");
        assert_eq!(spec.path, std::ffi::OsStr::new("/bin/sh"));
        assert_eq!(spec.args.len(), 1);
    }

    #[test]
    fn config_command_uses_sh_c() {
        let mut cfg = Config::with_defaults();
        cfg.command = Some("printf cfg-cmd".to_string());
        let spec = command_from_config(&cfg, None).expect("spec");
        assert_eq!(spec.args.len(), 3);
        assert_eq!(spec.args[2], std::ffi::OsStr::new("printf cfg-cmd"));
    }
}
