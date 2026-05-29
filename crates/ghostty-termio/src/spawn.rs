//! POSIX fork/exec spawn (`src/Command.zig` minimal subset).

#[cfg(unix)]
use std::collections::BTreeMap;
#[cfg(unix)]
use std::ffi::{CString, OsStr, OsString};
#[cfg(unix)]
use std::os::fd::RawFd;

#[cfg(unix)]
use std::os::fd::AsRawFd;

#[cfg(unix)]
use crate::command::CommandSpec;
#[cfg(unix)]
use crate::pty::{PosixPty, PtyOpenError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnError {
    ForkFailed,
    ExecFailed,
    InvalidArgument,
}

/// Open a PTY at `winsize` and spawn `spec` on its slave fd.
#[cfg(unix)]
pub fn spawn_pty_command(
    spec: &CommandSpec,
    winsize: crate::winsize::Winsize,
) -> Result<(crate::pty::PosixPty, libc::pid_t), SpawnPtyError> {
    let mut pty = PosixPty::open(winsize).map_err(SpawnPtyError::from)?;
    let slave = pty.take_slave();
    let pid = spawn_on_pty(spec, slave.as_raw_fd())?;
    Ok((pty, pid))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnPtyError {
    PtyOpenFailed,
    SpawnFailed,
}

#[cfg(unix)]
impl From<PtyOpenError> for SpawnPtyError {
    fn from(_: PtyOpenError) -> Self {
        Self::PtyOpenFailed
    }
}

#[cfg(unix)]
impl From<SpawnError> for SpawnPtyError {
    fn from(_: SpawnError) -> Self {
        Self::SpawnFailed
    }
}

/// Spawn `spec` with stdin/stdout/stderr attached to `slave_fd`.
///
/// All string/env allocation happens before `fork` (Zig contract).
#[cfg(unix)]
pub fn spawn_on_pty(spec: &CommandSpec, slave_fd: RawFd) -> Result<libc::pid_t, SpawnError> {
    if slave_fd < 0 {
        return Err(SpawnError::InvalidArgument);
    }

    let path = os_to_cstring(&spec.path)?;
    let argv = build_argv(&spec.args)?;
    let envp = build_envp(spec.env.as_ref())?;
    let argv_ptrs: Vec<*const libc::c_char> = argv
        .iter()
        .map(|s| s.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect();
    let env_ptrs: Vec<*const libc::c_char> = envp
        .iter()
        .map(|s| s.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect();

    let pid = unsafe { libc::fork() };
    if pid < 0 {
        return Err(SpawnError::ForkFailed);
    }

    if pid != 0 {
        return Ok(pid);
    }

    // Child: attach PTY and exec (no heap allocation from here on).
    child_setup(
        slave_fd,
        spec.cwd.as_deref(),
        &path,
        argv_ptrs.as_ptr(),
        env_ptrs.as_ptr(),
    );
}

#[cfg(unix)]
fn child_setup(
    slave_fd: RawFd,
    cwd: Option<&std::path::Path>,
    path: &CString,
    argv: *const *const libc::c_char,
    envp: *const *const libc::c_char,
) -> ! {
    unsafe {
        if libc::setsid() < 0 {
            libc::_exit(127);
        }

        if dup2_fd(slave_fd, libc::STDIN_FILENO).is_err()
            || dup2_fd(slave_fd, libc::STDOUT_FILENO).is_err()
            || dup2_fd(slave_fd, libc::STDERR_FILENO).is_err()
        {
            libc::_exit(127);
        }

        if slave_fd > libc::STDERR_FILENO {
            let _ = libc::close(slave_fd);
        }

        if let Some(dir) = cwd {
            let _ = std::env::set_current_dir(dir);
        }

        libc::execve(path.as_ptr(), argv, envp);
        libc::_exit(127);
    }
}

#[cfg(unix)]
fn dup2_fd(src: RawFd, dst: RawFd) -> Result<(), ()> {
    loop {
        let rc = unsafe { libc::dup2(src, dst) };
        if rc >= 0 {
            return Ok(());
        }
        let err = std::io::Error::last_os_error();
        if err.kind() == std::io::ErrorKind::Interrupted {
            continue;
        }
        return Err(());
    }
}

#[cfg(unix)]
fn os_to_cstring(value: &OsStr) -> Result<CString, SpawnError> {
    CString::new(value.as_encoded_bytes()).map_err(|_| SpawnError::InvalidArgument)
}

#[cfg(unix)]
fn build_argv(args: &[OsString]) -> Result<Vec<CString>, SpawnError> {
    args.iter().map(|arg| os_to_cstring(arg)).collect()
}

#[cfg(unix)]
fn build_envp(env: Option<&BTreeMap<OsString, OsString>>) -> Result<Vec<CString>, SpawnError> {
    let iter: Box<dyn Iterator<Item = (OsString, OsString)>> = match env {
        Some(map) => Box::new(map.iter().map(|(k, v)| (k.clone(), v.clone()))),
        None => Box::new(std::env::vars_os().map(|(k, v)| (k, v))),
    };

    iter.map(|(k, v)| {
        let mut pair = k.as_encoded_bytes().to_vec();
        pair.push(b'=');
        pair.extend_from_slice(v.as_encoded_bytes());
        CString::new(pair).map_err(|_| SpawnError::InvalidArgument)
    })
    .collect()
}

#[cfg(unix)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::AsRawFd;

    use crate::command::CommandBuilder;
    use crate::pty::PosixPty;
    use crate::winsize::Winsize;

    #[test]
    fn spawn_pty_command_runs_child() {
        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("exit 0")
            .build()
            .expect("command spec");

        let (pty, pid) = spawn_pty_command(
            &spec,
            Winsize {
                rows: 24,
                cols: 80,
                x_pixels: 0,
                y_pixels: 0,
            },
        )
        .expect("spawn pty command");

        assert!(pid > 0);
        let mut status: i32 = 0;
        loop {
            let rc = unsafe { libc::waitpid(pid, &mut status, 0) };
            if rc == pid {
                break;
            }
        }
        assert!(libc::WIFEXITED(status));
        assert_eq!(libc::WEXITSTATUS(status), 0);
        drop(pty);
    }

    #[test]
    fn spawn_echo_exits_zero() {
        let mut pty = PosixPty::open(Winsize {
            rows: 24,
            cols: 80,
            x_pixels: 0,
            y_pixels: 0,
        })
        .expect("openpty");

        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("exit 0")
            .build()
            .expect("command spec");

        let slave = pty.take_slave();
        let pid = spawn_on_pty(&spec, slave.as_raw_fd()).expect("spawn");
        assert!(pid > 0);

        let mut status: i32 = 0;
        loop {
            let rc = unsafe { libc::waitpid(pid, &mut status, 0) };
            if rc == pid {
                break;
            }
            if rc < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() != std::io::ErrorKind::Interrupted {
                    panic!("waitpid failed: {err}");
                }
            }
        }

        assert!(libc::WIFEXITED(status));
        assert_eq!(libc::WEXITSTATUS(status), 0);
    }

    #[test]
    fn spawn_pty_command_preserves_winsize() {
        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("exit 0")
            .build()
            .expect("command spec");

        let winsize = Winsize {
            rows: 40,
            cols: 120,
            x_pixels: 0,
            y_pixels: 0,
        };

        let (pty, pid) = spawn_pty_command(&spec, winsize).expect("spawn pty command");
        assert!(pid > 0);

        let size = pty.size().expect("TIOCGWINSZ");
        assert_eq!(size.rows, winsize.rows);
        assert_eq!(size.cols, winsize.cols);

        let mut status: i32 = 0;
        loop {
            let rc = unsafe { libc::waitpid(pid, &mut status, 0) };
            if rc == pid {
                break;
            }
        }
        assert!(libc::WIFEXITED(status));
        assert_eq!(libc::WEXITSTATUS(status), 0);
    }
}
