//! Interactive PTY/TTY support for running interactive commands
//! Uses std::process::Command with pipe-based I/O as a portable fallback.

use std::io::{Read, Write};
use std::sync::mpsc;

/// PTY output event
#[allow(dead_code)] // public API for interactive command execution
#[derive(Debug)]
pub enum PtyEvent {
    Output(String),
    Exit(i32),
    Error(String),
}

/// PTY session handle
#[allow(dead_code)] // public API for interactive command execution
pub struct PtySession {
    reader_thread: Option<std::thread::JoinHandle<()>>,
    stderr_thread: Option<std::thread::JoinHandle<()>>,
    child: Option<std::process::Child>,
    output_rx: mpsc::Receiver<PtyEvent>,
    writer: Option<Box<dyn Write + Send>>,
}

impl PtySession {
    /// Spawn an interactive PTY session
    #[allow(dead_code)] // public API for interactive command execution
    pub fn spawn(command: &str, _cols: u16, _rows: u16) -> Result<Self, String> {
        let (output_tx, output_rx) = mpsc::channel();

        let mut child = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn: {}", e))?;

        let stdout = child.stdout.take().ok_or("No stdout")?;
        let stderr = child.stderr.take().ok_or("No stderr")?;
        let stdin = child.stdin.take().ok_or("No stdin")?;

        let tx = output_tx.clone();
        let reader_thread = std::thread::spawn(move || {
            let mut reader = stdout;
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let s = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = tx.send(PtyEvent::Output(s));
                    }
                    Err(e) => {
                        let _ = tx.send(PtyEvent::Error(e.to_string()));
                        break;
                    }
                }
            }
        });

        let tx = output_tx;
        let stderr_thread = std::thread::spawn(move || {
            let mut reader = stderr;
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let s = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = tx.send(PtyEvent::Output(s));
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            reader_thread: Some(reader_thread),
            stderr_thread: Some(stderr_thread),
            child: Some(child),
            output_rx,
            writer: Some(Box::new(stdin)),
        })
    }

    /// Read pending output (non-blocking)
    #[allow(dead_code)] // public API for interactive command execution
    pub fn read_output(&self) -> Vec<PtyEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.output_rx.try_recv() {
            events.push(event);
        }
        events
    }

    /// Write input to the PTY
    #[allow(dead_code)] // public API for interactive command execution
    pub fn write_input(&mut self, data: &str) -> Result<(), String> {
        if let Some(writer) = &mut self.writer {
            writer
                .write_all(data.as_bytes())
                .map_err(|e| e.to_string())?;
            writer.flush().map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Check if the child process has exited
    #[allow(dead_code)] // public API for interactive command execution
    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            matches!(child.try_wait(), Ok(None))
        } else {
            false
        }
    }

    /// Wait for the child process to exit and return exit code
    #[allow(dead_code)] // public API for interactive command execution
    pub fn wait(&mut self) -> Result<i32, String> {
        if let Some(child) = &mut self.child {
            child
                .wait()
                .map(|status| status.code().unwrap_or(-1))
                .map_err(|e| e.to_string())
        } else {
            Err("No child process".to_string())
        }
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        drop(self.writer.take());
        // Kill child if still running
        if let Some(child) = &mut self.child {
            let _ = child.kill();
        }
        if let Some(thread) = self.reader_thread.take() {
            let _ = thread.join();
        }
        if let Some(thread) = self.stderr_thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_spawn_echo() {
        let session = PtySession::spawn("echo hello", 80, 24);
        assert!(session.is_ok());
        let session = session.unwrap();
        std::thread::sleep(std::time::Duration::from_millis(200));
        let events = session.read_output();
        assert!(
            events
                .iter()
                .any(|e| matches!(e, PtyEvent::Output(s) if s.contains("hello")))
        );
    }

    #[test]
    fn test_pty_write_input() {
        let session = PtySession::spawn("cat", 80, 24);
        assert!(session.is_ok());
        let mut session = session.unwrap();
        session.write_input("test\n").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(200));
        let events = session.read_output();
        assert!(
            events
                .iter()
                .any(|e| matches!(e, PtyEvent::Output(s) if s.contains("test")))
        );
    }

    #[test]
    fn test_pty_is_running() {
        let session = PtySession::spawn("sleep 10", 80, 24);
        assert!(session.is_ok());
        let mut session = session.unwrap();
        assert!(session.is_running());
    }

    #[test]
    fn test_pty_event_debug() {
        let event = PtyEvent::Output("test".to_string());
        assert!(format!("{:?}", event).contains("Output"));
    }
}
