//! Background termio I/O thread (`src/termio/Termio.zig` mailbox + pump cycle).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{
    mpsc::{self, Receiver, Sender, TryRecvError},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use ghostty_foundation::FoundationResult;

use crate::command::CommandSpec;
use crate::exec::ExecSpawn;
use crate::spawn::SpawnPtyError;
use crate::stream::PtyStream;
use crate::termio::TermioMessage;
use crate::winsize::Winsize;
use crate::TermioSink;

/// Events delivered from the termio thread to the embedder/main thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TermioThreadEvent {
    PtyOutput(Vec<u8>),
    ChildExit(u32),
    ResizeAck { cols: u16, rows: u16 },
    SetTitle(String),
    RedrawRequested,
}

/// Handle to a running termio background thread.
pub struct TermioThreadHandle {
    pid: libc::pid_t,
    inbox: Sender<TermioMessage>,
    events: Receiver<TermioThreadEvent>,
    shutdown: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
    child_exit: Option<u32>,
}

impl TermioThreadHandle {
    pub fn spawn(spec: &CommandSpec, winsize: Winsize) -> Result<Self, SpawnPtyError> {
        let exec = ExecSpawn::spawn(spec, winsize)?;
        let pid = exec.pid();
        exec.pty
            .set_nonblocking(true)
            .map_err(|_| SpawnPtyError::SpawnFailed)?;

        let (inbox_tx, inbox_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        let shutdown = Arc::new(AtomicBool::new(false));

        let shutdown_flag = Arc::clone(&shutdown);
        let join = thread::Builder::new()
            .name("ghostty-termio".into())
            .spawn(move || {
                termio_thread_main(exec, inbox_rx, event_tx, shutdown_flag);
            })
            .map_err(|_| SpawnPtyError::SpawnFailed)?;

        Ok(Self {
            pid,
            inbox: inbox_tx,
            events: event_rx,
            shutdown,
            join: Some(join),
            child_exit: None,
        })
    }

    pub fn pid(&self) -> libc::pid_t {
        self.pid
    }

    pub fn push(&self, msg: TermioMessage) -> FoundationResult<()> {
        if self.shutdown.load(Ordering::SeqCst) {
            return Err(ghostty_foundation::FoundationError::Unsupported);
        }
        self.inbox
            .send(msg)
            .map_err(|_| ghostty_foundation::FoundationError::OutOfMemory)
    }

    pub fn try_recv_event(&mut self) -> Option<TermioThreadEvent> {
        loop {
            match self.events.try_recv() {
                Ok(TermioThreadEvent::ChildExit(code)) => {
                    self.child_exit = Some(code);
                    return Some(TermioThreadEvent::ChildExit(code));
                }
                Ok(other) => return Some(other),
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Disconnected) => {
                    self.shutdown.store(true, Ordering::SeqCst);
                    return None;
                }
            }
        }
    }

    pub fn poll_child_exit(&mut self) -> Option<u32> {
        while let Some(event) = self.try_recv_event() {
            if let TermioThreadEvent::ChildExit(code) = event {
                return Some(code);
            }
        }
        self.child_exit
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }

    fn request_shutdown(&self) {
        let _ = self.inbox.send(TermioMessage::Shutdown);
    }
}

impl Drop for TermioThreadHandle {
    fn drop(&mut self) {
        self.request_shutdown();
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn termio_thread_main(
    exec: ExecSpawn,
    inbox: Receiver<TermioMessage>,
    events: Sender<TermioThreadEvent>,
    shutdown: Arc<AtomicBool>,
) {
    let winsize = exec.pty.size().unwrap_or(Winsize {
        cols: 80,
        rows: 24,
        x_pixels: 0,
        y_pixels: 0,
    });
    let mut stream = PtyStream::new(exec.pty, winsize);
    let mut child = exec.child;
    let mut pending: Vec<TermioMessage> = Vec::new();

    struct EventSink<'a> {
        tx: &'a Sender<TermioThreadEvent>,
    }

    impl TermioSink for EventSink<'_> {
        fn write_terminal(&mut self, bytes: &[u8]) {
            if !bytes.is_empty() {
                let _ = self.tx.send(TermioThreadEvent::PtyOutput(bytes.to_vec()));
            }
        }

        fn resize_terminal(&mut self, cols: u16, rows: u16) {
            let _ = self.tx.send(TermioThreadEvent::ResizeAck { cols, rows });
        }
    }

    while !shutdown.load(Ordering::SeqCst) && !stream.is_shutdown() {
        while let Ok(msg) = inbox.try_recv() {
            pending.push(msg);
        }

        if !pending.is_empty() {
            let batch: Vec<_> = pending.drain(..).collect();
            let mut sink = EventSink { tx: &events };
            for msg in &batch {
                if let TermioMessage::SetTitle(title) = msg {
                    let _ = events.send(TermioThreadEvent::SetTitle(title.clone()));
                }
                if matches!(msg, TermioMessage::RedrawRequested) {
                    let _ = events.send(TermioThreadEvent::RedrawRequested);
                }
            }
            let _ = stream.apply_messages(batch, &mut sink);
            if stream.is_shutdown() {
                shutdown.store(true, Ordering::SeqCst);
            }
        }

        if let Some(code) = child.poll_exit() {
            let _ = events.send(TermioThreadEvent::ChildExit(code));
            shutdown.store(true, Ordering::SeqCst);
            break;
        }

        let mut sink = EventSink { tx: &events };
        let _ = stream.pump_into(&mut sink);

        match stream.poll_readable(50) {
            Ok(true) => continue,
            Ok(false) => thread::sleep(Duration::from_millis(5)),
            Err(_) => break,
        }
    }

    child.terminate();
    shutdown.store(true, Ordering::SeqCst);
}

#[cfg(all(unix, test))]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::command::CommandBuilder;

    #[test]
    fn thread_echo_round_trip() {
        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("cat")
            .build()
            .expect("spec");
        let winsize = Winsize {
            cols: 80,
            rows: 24,
            x_pixels: 0,
            y_pixels: 0,
        };
        let mut handle = TermioThreadHandle::spawn(&spec, winsize).expect("spawn");
        handle
            .push(TermioMessage::Write(b"thread-ping\n".to_vec()))
            .expect("write");

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut saw = false;
        while Instant::now() < deadline {
            if let Some(TermioThreadEvent::PtyOutput(out)) = handle.try_recv_event() {
                if out.windows(11).any(|w| w == b"thread-ping") {
                    saw = true;
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(saw, "expected thread-ping in PTY output");
    }

    #[test]
    fn thread_child_exit() {
        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("exit 7")
            .build()
            .expect("spec");
        let winsize = Winsize {
            cols: 80,
            rows: 24,
            x_pixels: 0,
            y_pixels: 0,
        };
        let mut handle = TermioThreadHandle::spawn(&spec, winsize).expect("spawn");
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if handle.poll_child_exit() == Some(7) {
                return;
            }
            if Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("expected child exit 7");
    }
}
