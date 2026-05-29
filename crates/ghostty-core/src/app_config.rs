//! Application configuration handle (`Config.zig` / `ghostty_config_t`).
//!
//! Full config FFI and field getters remain in Zig for now; Rust holds an owned
//! [`ghostty_config::Config`] subset used when bootstrapping `App`.

use ghostty_config::{CliArgs, Config};

/// Owned application configuration (mirrors embedded `App.config`).
#[derive(Debug, Clone)]
pub struct AppConfig {
    inner: Config,
}

impl AppConfig {
    pub fn new(config: Config) -> Self {
        Self { inner: config }
    }

    pub fn with_defaults() -> Self {
        Self {
            inner: Config::with_defaults(),
        }
    }

    /// Load from CLI args (`src/cli/args.zig` subset).
    pub fn from_cli(args: &CliArgs) -> Result<Self, ghostty_config::LoadError> {
        Ok(Self::new(args.load_config()?))
    }

    /// Load a single config file (tilde-expanded).
    pub fn from_config_file(path: &std::path::Path) -> Result<Self, ghostty_config::LoadError> {
        let mut cfg = Config::with_defaults();
        let expanded = ghostty_config::expand_path(path.to_string_lossy().as_ref());
        cfg.load_file(&expanded)?;
        cfg.finalize();
        Ok(Self::new(cfg))
    }

    pub fn config(&self) -> &Config {
        &self.inner
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.inner
    }

    pub fn into_inner(self) -> Config {
        self.inner
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_crate() {
        let app = AppConfig::default();
        assert_eq!(app.config().font_size, Config::with_defaults().font_size);
    }
}
