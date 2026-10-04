use std::{
    io::{self, BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::Path,
    process::{ Command},
    thread,
    time::Duration,
};

#[derive(Debug)]
pub struct Player {
    socket: UnixStream,
    paused: bool,
}

const SOCKET: &str = "/tmp/my-player.sock";

impl Player {
    pub fn new() -> io::Result<Self> {
        let paused = false;

        if let Ok(socket) = UnixStream::connect(SOCKET) {
            return Ok(Self { socket, paused });
        }

        let _mpv = Command::new("mpv")
            .args([
                "--idle=yes",
                "--no-video",
                "--no-terminal",
                &format!("--input-ipc-server={SOCKET}"),
            ])
            .spawn()?;

        for _ in 0..50 {
            if let Ok(socket) = UnixStream::connect(SOCKET) {
                return Ok(Self { socket, paused });
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
    pub fn toggle_pause_play(&mut self) -> io::Result<()> {
        self.paused = !self.paused; 
        let command = serde_json::json!({
            "command": ["set_property", "pause", self.paused]
        });

        writeln!(self.socket, "{command}")?;

        Ok(())
    }
    #[allow(dead_code)]
    pub fn depause(&mut self) -> io::Result<()> {
        let command = serde_json::json!({
            "command": ["set_property", "pause", false]
        });

        writeln!(self.socket, "{command}")?;

        Ok(())
    }
    #[allow(dead_code)]
    pub fn get_playing(&mut self) -> io::Result<Option<String>> {
        let command = serde_json::json!({
            "command": ["get_property", "path"]
        });

        writeln!(self.socket, "{command}")?;
        self.socket.flush()?;

        let reader_socket = self.socket.try_clone()?;
        let mut reader = BufReader::new(reader_socket);

        let mut response = String::new();
        reader.read_line(&mut response)?;

        let json: serde_json::Value = serde_json::from_str(&response)?;

        Ok(json["data"].as_str().map(String::from))
    }
    pub fn get_duration(&mut self) -> io::Result<Option<f64>> {
        let command = serde_json::json!({
            "command": ["get_property", "duration"]
        });

        writeln!(self.socket, "{command}")?;
        self.socket.flush()?;

        let reader_socket = self.socket.try_clone()?;
        let mut reader = BufReader::new(reader_socket);

        let mut response = String::new();
        reader.read_line(&mut response)?;

        let json: serde_json::Value = serde_json::from_str(&response)?;

        Ok(json["data"].as_f64())
    }

    pub fn get_progress(&mut self) -> io::Result<Option<f64>> {
        let command = serde_json::json!({
            "command": ["get_property", "time-pos"]
        });

        writeln!(self.socket, "{command}")?;
        self.socket.flush()?;

        let reader_socket = self.socket.try_clone()?;
        let mut reader = BufReader::new(reader_socket);

        let mut response = String::new();
        reader.read_line(&mut response)?;

        let json: serde_json::Value = serde_json::from_str(&response)?;

        Ok(json["data"].as_f64())
    }

}
