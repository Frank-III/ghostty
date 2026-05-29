//! CLI argument subset (`src/cli/args.zig`).

use std::path::PathBuf;

/// Parsed Ghostty CLI flags used by the Rust port bootstrap.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CliArgs {
    pub config_files: Vec<PathBuf>,
    pub help: bool,
    pub version: bool,
}

impl CliArgs {
    pub fn parse(argv: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        let mut out = Self::default();
        let mut iter = argv.into_iter().map(|s| s.as_ref().to_string()).peekable();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--help" | "-h" => out.help = true,
                "--version" | "-v" => out.version = true,
                "--config-file" | "--config" => {
                    if let Some(path) = iter.next() {
                        out.config_files.push(PathBuf::from(path));
                    }
                }
                _ if arg.starts_with("--config-file=") => {
                    let path = arg.trim_start_matches("--config-file=");
                    out.config_files.push(PathBuf::from(path));
                }
                _ => {}
            }
        }
        out
    }

    /// Load config from explicit files, or default discovery when none given.
    pub fn load_config(&self) -> Result<crate::Config, crate::error::LoadError> {
        let mut cfg = crate::Config::with_defaults();
        if self.config_files.is_empty() {
            cfg.load_default_files();
            cfg.finalize(None);
            return Ok(cfg);
        }
        for path in &self.config_files {
            let expanded = crate::path::expand_path(path.to_string_lossy().as_ref());
            cfg.load_file(&expanded)?;
        }
        cfg.finalize(None);
        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_file_flag() {
        let args = CliArgs::parse(["ghostty", "--config-file", "/tmp/x.conf"]);
        assert_eq!(args.config_files, vec![PathBuf::from("/tmp/x.conf")]);
    }

    #[test]
    fn parse_help() {
        let args = CliArgs::parse(["ghostty", "--help"]);
        assert!(args.help);
    }
}
