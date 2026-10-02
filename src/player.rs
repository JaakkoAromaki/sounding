use std::{
    io::{self, Write},
    os::unix::net::UnixStream,
    path::Path,
    process::{ Command},
    thread,
    time::Duration,
};

#[derive(Debug)]
pub struct Player {
    socket: UnixStream,
}

const SOCKET: &str = "/tmp/my-player.sock";

impl Player {
    pub fn new() -> io::Result<Self> {
        let _mpv = Command::new("mpv")
            .args(["--idle=yes", "--no-video", &format!("--input-ipc-server={SOCKET}")])
            .spawn()?;

        for _ in 0..50 {
            if let Ok(socket) = UnixStream::connect(SOCKET) {
                return Ok(Self { socket });
            }

            thread::sleep(Duration::from_millis(20));
        }

        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "mpv socket was not created",
        ))
    }

    #[allow(dead_code)]
    pub fn play(&mut self, path: &Path) -> io::Result<()> {
        let command = serde_json::json!({
            "command": [
                "loadfile", path.to_string_lossy(),
            ]
        });

        writeln!(self.socket, "{command}")?;

        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn pause(&mut self) -> io::Result<()> {
        let command = serde_json::json!({
            "command": ["set_property", "pause", true]
        });

        writeln!(self.socket, "{command}")?;

        Ok(())
    }
}
