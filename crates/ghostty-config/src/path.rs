//! Config path expansion (`src/config/path.zig` subset).

use std::path::{Path, PathBuf};

/// Expand leading `~/` to the current user's home directory.
pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

/// Expand tilde and return an absolute path when possible.
pub fn expand_path(path: &str) -> PathBuf {
    let expanded = expand_tilde(path);
    if expanded.is_absolute() {
        return expanded;
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(&expanded))
        .unwrap_or(expanded)
}

fn home_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
    #[cfg(not(unix))]
    {
        None
    }
}

/// If `path` is relative, resolve against `base` (config file directory).
pub fn resolve_relative(base: &Path, path: &str) -> PathBuf {
    let expanded = expand_tilde(path);
    if expanded.is_absolute() {
        expanded
    } else {
        base.join(expanded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_tilde_prefix() {
        if std::env::var_os("HOME").is_some() {
            let p = expand_tilde("~/ghostty/config");
            assert!(p.to_string_lossy().contains("ghostty"));
        }
    }

    #[test]
    fn resolve_relative_against_base() {
        let base = Path::new("/tmp/cfg");
        let p = resolve_relative(base, "child.conf");
        assert_eq!(p, PathBuf::from("/tmp/cfg/child.conf"));
    }
}
