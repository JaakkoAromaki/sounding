use std::process::Command;
use std::path::{Path, PathBuf};

use yt_dlp::Downloader;
use yt_dlp::client::deps::Libraries;
use yt_dlp::model::playlist::Playlist;

#[allow(dead_code)]
fn find_program(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let output = Command::new("which").arg(name).output()?;

    if !output.status.success() {return Err(format!("{name} was not found in PATH. Please add it to PATH or install it.").into())}

    let path = String::from_utf8(output.stdout)?;
    Ok(PathBuf::from(path.trim()))
}

#[allow(dead_code)]
pub async fn download(url: &str, output: &Path, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let youtube = find_program("yt-dlp")?;
    let ffmpeg = find_program("ffmpeg")?;

    let libraries = Libraries::new(youtube, ffmpeg);

    let downloader = Downloader::builder(libraries, output)
        .build()
        .await?;

    let video = downloader.fetch_video_infos(url).await?;

    downloader
        .download_audio_stream(&video, name)
        .await?;

    Ok(())
}
pub async fn search(query: &str) -> Result<Playlist, Box<dyn std::error::Error>> {
    let youtube = find_program("yt-dlp")?;
    let ffmpeg = find_program("ffmpeg")?;

    let libraries = Libraries::new(youtube, ffmpeg);

    let downloader = Downloader::builder(libraries, "output")
        .build()
        .await?;


    let youtube = downloader.youtube_extractor();

    let results = youtube.search(query, 10).await?;

    Ok(results)
}