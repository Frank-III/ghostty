//! POSIX PTY open/resize (`src/pty.zig` `PosixPty`).

use std::mem::ManuallyDrop;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};

use crate::mode::PtyMode;
use crate::winsize::Winsize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtyOpenError {
    OpenptyFailed,
    GetModeFailed,
    SetSizeFailed,
    GetSizeFailed,
}

/// Master/slave pair from `openpty`.
///
/// On drop only the master is closed; take the slave with [`PosixPty::take_slave`]
/// before drop if you need to pass it to a child (Zig contract).
pub struct PosixPty {
    master: OwnedFd,
    slave: ManuallyDrop<OwnedFd>,
}

impl PosixPty {
    pub fn open(size: Winsize) -> Result<Self, PtyOpenError> {
        let mut master: RawFd = -1;
        let mut slave: RawFd = -1;
        let mut ws = size.to_libc();

        let rc = unsafe { libc::openpty(&mut master, &mut slave, std::ptr::null_mut(), std::ptr::null_mut(), &mut ws) };
        if rc != 0 {
            return Err(PtyOpenError::OpenptyFailed);
        }

        let master_fd = unsafe { OwnedFd::from_raw_fd(master) };
        let slave_fd = unsafe { OwnedFd::from_raw_fd(slave) };

        set_cloexec(master_fd.as_raw_fd());

        if !set_utf8_mode(master_fd.as_raw_fd()) {
            return Err(PtyOpenError::OpenptyFailed);
        }

        Ok(Self {
            master: master_fd,
            slave: ManuallyDrop::new(slave_fd),
        })
    }

    pub fn master(&self) -> &OwnedFd {
        &self.master
    }

    pub fn slave_fd(&self) -> RawFd {
        self.slave.as_raw_fd()
    }

    pub fn take_slave(&mut self) -> OwnedFd {
        // SAFETY: `take_slave` consumes the only remaining handle to this fd.
        unsafe { ManuallyDrop::take(&mut self.slave) }
    }

    pub fn mode(&self) -> Result<PtyMode, PtyOpenError> {
        let mut attrs: libc::termios = unsafe { std::mem::zeroed() };
        if unsafe { libc::tcgetattr(self.master.as_raw_fd(), &mut attrs) } != 0 {
            return Err(PtyOpenError::GetModeFailed);
        }
        Ok(PtyMode {
            canonical: (attrs.c_lflag & libc::ICANON) != 0,
            echo: (attrs.c_lflag & libc::ECHO) != 0,
        })
    }

    pub fn size(&self) -> Result<Winsize, PtyOpenError> {
        let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
        if unsafe { libc::ioctl(self.master.as_raw_fd(), libc::TIOCGWINSZ, &mut ws) } < 0 {
            return Err(PtyOpenError::GetSizeFailed);
        }
        Ok(Winsize::from_libc(ws))
    }

    pub fn set_size(&self, size: Winsize) -> Result<(), PtyOpenError> {
        let ws = size.to_libc();
        if unsafe { libc::ioctl(self.master.as_raw_fd(), libc::TIOCSWINSZ, &ws) } < 0 {
            return Err(PtyOpenError::SetSizeFailed);
        }
        Ok(())
    }

    /// Write bytes to the PTY master (child stdin).
    pub fn write(&self, buf: &[u8]) -> Result<usize, PtyIoError> {
        if buf.is_empty() {
            return Ok(0);
        }
        loop {
            let rc = unsafe {
                libc::write(
                    self.master.as_raw_fd(),
                    buf.as_ptr().cast(),
                    buf.len(),
                )
            };
            if rc >= 0 {
                return Ok(rc as usize);
            }
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(PtyIoError::WriteFailed);
        }
    }

    /// Read available bytes from the PTY master (child stdout/stderr).
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, PtyIoError> {
        if buf.is_empty() {
            return Ok(0);
        }
        loop {
            let rc = unsafe {
                libc::read(
                    self.master.as_raw_fd(),
                    buf.as_mut_ptr().cast(),
                    buf.len(),
                )
            };
            if rc >= 0 {
                return Ok(rc as usize);
            }
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            if err.kind() == std::io::ErrorKind::WouldBlock {
                return Ok(0);
            }
            return Err(PtyIoError::ReadFailed);
        }
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> Result<(), PtyIoError> {
        let fd = self.master.as_raw_fd();
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if flags < 0 {
            return Err(PtyIoError::SetModeFailed);
        }
        let next = if nonblocking {
            flags | libc::O_NONBLOCK
        } else {
            flags & !libc::O_NONBLOCK
        };
        if unsafe { libc::fcntl(fd, libc::F_SETFL, next) } < 0 {
            return Err(PtyIoError::SetModeFailed);
        }
        Ok(())
    }

    /// Wait until the master fd is readable or `timeout_ms` elapses.
    pub fn poll_readable(&self, timeout_ms: i32) -> Result<bool, PtyIoError> {
        let fd = self.master.as_raw_fd();
        let mut pfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        loop {
            let rc = unsafe { libc::poll(&mut pfd, 1, timeout_ms) };
            if rc < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(PtyIoError::PollFailed);
            }
            return Ok(rc > 0 && (pfd.revents & libc::POLLIN) != 0);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtyIoError {
    ReadFailed,
    WriteFailed,
    SetModeFailed,
    PollFailed,
}

fn set_cloexec(fd: RawFd) {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        return;
    }
    let _ = unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) };
}

/// Enable `IUTF8` on the master fd (macOS default-off; Linux often on).
fn set_utf8_mode(fd: RawFd) -> bool {
    let mut attrs: libc::termios = unsafe { std::mem::zeroed() };
    if unsafe { libc::tcgetattr(fd, &mut attrs) } != 0 {
        return false;
    }
    attrs.c_iflag |= libc::IUTF8;
    unsafe { libc::tcsetattr(fd, libc::TCSANOW, &attrs) == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_resize_and_mode() {
        let mut pty = PosixPty::open(Winsize {
            rows: 24,
            cols: 80,
            x_pixels: 0,
            y_pixels: 0,
        })
        .expect("openpty");

        let mode = pty.mode().expect("tcgetattr");
        assert!(mode.canonical);
        assert!(mode.echo);

        let initial = pty.size().expect("TIOCGWINSZ");
        assert_eq!(initial.rows, 24);
        assert_eq!(initial.cols, 80);

        pty.set_size(Winsize {
            rows: 30,
            cols: 100,
            x_pixels: 0,
            y_pixels: 0,
        })
        .expect("TIOCSWINSZ");

        let updated = pty.size().expect("TIOCGWINSZ");
        assert_eq!(updated.rows, 30);
        assert_eq!(updated.cols, 100);

        drop(pty.take_slave());
    }
}
