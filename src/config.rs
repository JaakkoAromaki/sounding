use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub music_directory: String,

    pub ui: UiConfig,
    pub player: PlayerConfig,
}

#[derive(Debug, Deserialize)]
pub struct UiConfig {
    pub show_hidden: bool,
    pub theme: String,
}

#[derive(Debug, Deserialize)]
pub struct PlayerConfig {
    pub volume: u8,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not find config directory")?;

        let sounding_dir = config_dir.join("sounding");
        let config_path = sounding_dir.join("config.toml");

        if !config_path.exists() {
            fs::create_dir_all(&sounding_dir)?;

            fs::write(
                &config_path,
                r#"
music_directory = "/home/hamlak/music"

[ui]
show_hidden = false
theme = "default"

[player]
volume = 80
"#,
            )?;
        }

        let contents = fs::read_to_string(config_path)?;
        let config = toml::from_str(&contents)?;

        Ok(config)
    }
}
