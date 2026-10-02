use std::process::Command;
use std::path::PathBuf;

use yt_dlp::Downloader;
use yt_dlp::client::deps::Libraries;

#[allow(dead_code)]
fn find_program(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let output = Command::new("which").arg(name).output()?;

    if !output.status.success() {return Err(format!("{name} was not found in PATH. Please add it to PATH or install it.").into())}

    let path = String::from_utf8(output.stdout)?;
    Ok(PathBuf::from(path.trim()))
}

#[allow(dead_code)]
pub async fn download(url: &str, output: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = PathBuf::from(output);

    let youtube = find_program("yt-dlp")?;
    let ffmpeg = find_program("ffmpeg")?;

    println!("depencies checked");

    let libraries = Libraries::new(youtube, ffmpeg);

    let downloader = Downloader::builder(libraries, output_dir)
        .build()
        .await?;

    let video = downloader.fetch_video_infos(url).await?;

    downloader
        .download_audio_stream(&video, name)
        .await?;

    Ok(())
}
