//! Subprocess launch spec (`src/Command.zig` skeleton — no fork/exec yet).

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// How a child process exited (`Command.Exit` on POSIX).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessExit {
    Exited(u8),
    Signaled(u32),
    Stopped(u32),
    Unknown(u32),
}

#[cfg(unix)]
pub fn process_exit_from_wait_status(status: i32) -> ProcessExit {
    if libc::WIFEXITED(status) {
        ProcessExit::Exited(libc::WEXITSTATUS(status) as u8)
    } else if libc::WIFSIGNALED(status) {
        ProcessExit::Signaled(libc::WTERMSIG(status) as u32)
    } else if libc::WIFSTOPPED(status) {
        ProcessExit::Stopped(libc::WSTOPSIG(status) as u32)
    } else {
        ProcessExit::Unknown(status as u32)
    }
}

#[cfg(not(unix))]
pub fn process_exit_from_wait_status(status: i32) -> ProcessExit {
    ProcessExit::Exited(status as u8)
}

/// Immutable description of a command to run once spawn is implemented.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub path: OsString,
    pub args: Vec<OsString>,
    pub env: Option<BTreeMap<OsString, OsString>>,
    pub cwd: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBuildError {
    EmptyPath,
    EmptyArgs,
}

/// Builder mirroring Zig `Command` fields used at spawn time.
#[derive(Debug, Default)]
pub struct CommandBuilder {
    path: Option<OsString>,
    args: Vec<OsString>,
    env: Option<BTreeMap<OsString, OsString>>,
    cwd: Option<PathBuf>,
}

impl CommandBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn path(mut self, path: impl AsRef<OsStr>) -> Self {
        self.path = Some(path.as_ref().to_os_string());
        self
    }

    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|s| s.as_ref().to_os_string()));
        self
    }

    pub fn env(mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.env
            .get_or_insert_with(BTreeMap::new)
            .insert(key.as_ref().to_os_string(), value.as_ref().to_os_string());
        self
    }

    pub fn cwd(mut self, cwd: impl AsRef<Path>) -> Self {
        self.cwd = Some(cwd.as_ref().to_path_buf());
        self
    }

    pub fn build(self) -> Result<CommandSpec, CommandBuildError> {
        let path = self.path.ok_or(CommandBuildError::EmptyPath)?;
        if path.is_empty() {
            return Err(CommandBuildError::EmptyPath);
        }

        let mut args = self.args;
        if args.is_empty() {
            args.push(path.clone());
        } else if args[0].is_empty() {
            return Err(CommandBuildError::EmptyArgs);
        }

        Ok(CommandSpec {
            path,
            args,
            env: self.env,
            cwd: self.cwd,
        })
    }
}

impl CommandSpec {
    /// Argument vector for `execvp`-style APIs: `argv[0]` is the program name.
    pub fn argv(&self) -> Vec<&OsStr> {
        self.args.iter().map(OsStr::new).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_sets_argv0_from_path_when_args_empty() {
        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .build()
            .unwrap();
        assert_eq!(spec.path, OsStr::new("/bin/sh"));
        assert_eq!(spec.args.len(), 1);
        assert_eq!(spec.args[0], OsStr::new("/bin/sh"));
    }

    #[test]
    fn builder_preserves_explicit_args() {
        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-l")
            .cwd("/tmp")
            .env("TERM", "xterm-ghostty")
            .build()
            .unwrap();
        assert_eq!(spec.argv(), vec![OsStr::new("sh"), OsStr::new("-l")]);
        assert_eq!(spec.cwd.as_deref(), Some(Path::new("/tmp")));
        assert_eq!(
            spec.env.as_ref().unwrap().get(OsStr::new("TERM")).map(OsStr::new),
            Some(OsStr::new("xterm-ghostty"))
        );
    }

    #[test]
    fn empty_path_is_rejected() {
        assert_eq!(
            CommandBuilder::new().path("").build(),
            Err(CommandBuildError::EmptyPath)
        );
    }

    #[cfg(unix)]
    #[test]
    fn wait_status_normal_exit() {
        // status for exit 42
        let status = 42 << 8;
        assert_eq!(
            process_exit_from_wait_status(status),
            ProcessExit::Exited(42)
        );
    }
}
