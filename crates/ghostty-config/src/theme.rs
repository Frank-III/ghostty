//! Theme file discovery (`src/config/theme.zig` subset).

use std::path::{Path, PathBuf};

use crate::file_load;

/// Resolve a theme config value to a readable file path.
///
/// Absolute paths are used as-is. Relative names are searched under the user
/// themes directory, then `{resources_dir}/themes`.
pub fn resolve_theme_path(
    theme: &str,
    resources_dir: Option<&Path>,
) -> Result<PathBuf, ThemeResolveError> {
    let theme = theme.trim();
    if theme.is_empty() {
        return Err(ThemeResolveError::Empty);
    }

    let path = Path::new(theme);
    if path.is_absolute() {
        return Ok(crate::expand_path(theme));
    }

    if theme.contains(std::path::MAIN_SEPARATOR) {
        return Err(ThemeResolveError::RelativeWithSeparators);
    }

    for dir in theme_search_dirs(resources_dir) {
        let candidate = dir.join(theme);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(ThemeResolveError::NotFound(theme.to_string()))
}

/// Directories searched for named themes, in priority order.
pub fn theme_search_dirs(resources_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(user) = user_themes_dir() {
        if user.is_dir() {
            dirs.push(user);
        }
    }
    if let Some(resources) = resources_dir {
        let res = resources.join("themes");
        if res.is_dir() {
            dirs.push(res);
        }
    } else if let Some(env) = std::env::var_os("GHOSTTY_RESOURCES_DIR") {
        let res = PathBuf::from(env).join("themes");
        if res.is_dir() {
            dirs.push(res);
        }
    }
    dirs
}

/// `$XDG_CONFIG_HOME/ghostty/themes` or `~/.config/ghostty/themes`.
pub fn user_themes_dir() -> Option<PathBuf> {
    Some(file_load::xdg_config_dir().join("ghostty").join("themes"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeResolveError {
    Empty,
    RelativeWithSeparators,
    NotFound(String),
}

impl std::fmt::Display for ThemeResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "theme name is empty"),
            Self::RelativeWithSeparators => {
                write!(
                    f,
                    "theme cannot include path separators unless it is absolute"
                )
            }
            Self::NotFound(name) => write!(f, "theme \"{name}\" not found"),
        }
    }
}

impl std::error::Error for ThemeResolveError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_theme_uses_expand() {
        let path = resolve_theme_path("/tmp/my-theme", None).unwrap();
        assert_eq!(path, PathBuf::from("/tmp/my-theme"));
    }

    #[test]
    fn relative_with_separator_errors() {
        assert_eq!(
            resolve_theme_path("foo/bar", None).unwrap_err(),
            ThemeResolveError::RelativeWithSeparators
        );
    }

    #[test]
    fn named_theme_in_resources_dir() {
        let dir = std::env::temp_dir().join(format!("ghostty-res-themes-{}", std::process::id()));
        let themes = dir.join("themes");
        std::fs::create_dir_all(&themes).unwrap();
        let theme_file = themes.join("dark");
        std::fs::write(&theme_file, "background = #000000\n").unwrap();
        let resolved = resolve_theme_path("dark", Some(&dir)).unwrap();
        assert_eq!(resolved, theme_file);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
