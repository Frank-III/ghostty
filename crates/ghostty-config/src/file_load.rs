//! Default config file paths (`src/config/file_load.zig`).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::LoadError;

/// XDG config path: `$XDG_CONFIG_HOME/ghostty/config.ghostty` or `~/.config/ghostty/config.ghostty`.
pub fn default_xdg_path() -> PathBuf {
    xdg_config_dir().join("ghostty").join("config.ghostty")
}

/// Legacy XDG path (Ghostty &lt; 1.3.0).
pub fn legacy_default_xdg_path() -> PathBuf {
    xdg_config_dir().join("ghostty").join("config")
}

/// macOS Application Support path for `config.ghostty`.
#[cfg(target_os = "macos")]
pub fn default_app_support_path() -> PathBuf {
    home_dir()
        .join("Library")
        .join("Application Support")
        .join("com.mitchellh.ghostty")
        .join("config.ghostty")
}

#[cfg(target_os = "macos")]
pub fn legacy_default_app_support_path() -> PathBuf {
    home_dir()
        .join("Library")
        .join("Application Support")
        .join("com.mitchellh.ghostty")
        .join("config")
}

/// Preferred default file path (exists check, then legacy, then new default).
pub fn preferred_default_file_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let app = default_app_support_path();
        if app.is_file() {
            return app;
        }
        let xdg = preferred_xdg_path();
        if xdg.is_file() {
            return xdg;
        }
        return app;
    }
    #[cfg(not(target_os = "macos"))]
    {
        preferred_xdg_path()
    }
}

fn preferred_xdg_path() -> PathBuf {
    let xdg = default_xdg_path();
    if xdg.is_file() {
        return xdg;
    }
    let legacy = legacy_default_xdg_path();
    if legacy.is_file() {
        return legacy;
    }
    xdg
}

fn xdg_config_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    home_dir().join(".config")
}

fn home_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home);
    }
    dirs_fallback_home()
}

#[cfg(target_os = "windows")]
fn dirs_fallback_home() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(not(target_os = "windows"))]
fn dirs_fallback_home() -> PathBuf {
    PathBuf::from(".")
}

/// Open constraints for config files (absolute path, non-empty regular file).
pub fn validate_config_path(path: &Path) -> Result<(), LoadError> {
    if !path.is_absolute() {
        return Err(LoadError::NotAbsolute);
    }
    let meta = std::fs::metadata(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            LoadError::FileNotFound
        } else {
            LoadError::FileOpenFailed
        }
    })?;
    if !meta.is_file() {
        return Err(LoadError::NotAFile);
    }
    if meta.len() == 0 {
        return Err(LoadError::FileIsEmpty);
    }
    Ok(())
}

/// Resolve a `config-file` path relative to the including file's directory.
pub fn resolve_config_file_path(base_file: &Path, entry: &str) -> PathBuf {
    let trimmed = entry.trim();
    if trimmed.starts_with('/') {
        return PathBuf::from(trimmed);
    }
    base_file
        .parent()
        .map(|dir| dir.join(trimmed))
        .unwrap_or_else(|| PathBuf::from(trimmed))
}

/// Load a config file and any nested `config-file` entries (cycle-safe).
pub fn load_recursive(
    config: &mut crate::Config,
    path: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<(), LoadError> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canonical.clone()) {
        return Ok(());
    }
    validate_config_path(&canonical)?;
    let content =
        std::fs::read_to_string(&canonical).map_err(|_| LoadError::FileOpenFailed)?;
    let includes = config.load_from_str_collect_includes(&content, &canonical.to_string_lossy());
    for include in includes {
        let resolved = resolve_config_file_path(&canonical, &include);
        load_recursive(config, &resolved, visited)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xdg_path_suffix() {
        let p = default_xdg_path();
        assert!(p.ends_with("ghostty/config.ghostty"));
    }
}
