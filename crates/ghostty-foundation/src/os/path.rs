//! PATH lookup helpers ported from `src/os/path.zig`.
//!
//! Windows PATH expansion is deferred until the termio phase needs it.

use std::path::{Path, PathBuf};

/// Search `PATH` for an executable named `cmd` and return its absolute path.
///
/// If `cmd` contains a path separator, returns a copy of `cmd` (relative or
/// absolute) without searching `PATH`, matching Zig `os.path.expand`.
pub fn expand(cmd: &str) -> Result<Option<PathBuf>, ExpandError> {
    if cmd.is_empty() {
        return Ok(None);
    }

    if cmd.contains('/') || cmd.contains('\\') {
        return Ok(Some(PathBuf::from(cmd)));
    }

    let path_var = std::env::var("PATH").map_err(|_| ExpandError::PathNotSet)?;
    let delimiter = if cfg!(windows) { ';' } else { ':' };

    let mut saw_eacces = false;
    for search_path in path_var.split(delimiter) {
        if search_path.is_empty() {
            continue;
        }
        let candidate = Path::new(search_path).join(cmd);
        match std::fs::metadata(&candidate) {
            Ok(meta) if meta.is_file() && is_executable(&meta) => {
                return Ok(Some(candidate));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                saw_eacces = true;
            }
            Err(e) => return Err(ExpandError::Io(e.to_string())),
            _ => {}
        }
    }

    if saw_eacces {
        return Err(ExpandError::AccessDenied);
    }
    Ok(None)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpandError {
    PathNotSet,
    AccessDenied,
    Io(String),
}

#[cfg(unix)]
fn is_executable(meta: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_meta: &std::fs::Metadata) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::expand;
    use std::path::PathBuf;

    #[test]
    fn relative_cmd_is_returned_as_is() {
        let out = expand("./bin/ghostty").unwrap().expect("path");
        assert_eq!(out, PathBuf::from("./bin/ghostty"));
    }

    #[test]
    fn expand_finds_hostname_or_uname() {
        let cmd = if cfg!(windows) { "hostname.exe" } else { "uname" };
        let path = expand(cmd).unwrap().expect("executable in PATH");
        assert!(path.as_os_str().len() > cmd.len());
    }
}
