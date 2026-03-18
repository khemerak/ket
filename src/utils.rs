use indicatif::{ProgressBar, ProgressStyle};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;

pub fn print(msg: String, quiet_mode: bool) {
    if !quiet_mode {
        println!("{}", msg);
    }
}

pub fn save_to_file(buf: &[u8], fname: &str) -> Result<()> {
    let mut file = File::create(fname).context(format!("Failed to create output file: {}", fname))?;
    file.write_all(buf).context("Failed to write data to the file")?;
    Ok(())
}

pub fn create_progress_bar(quiet_mode: bool, msg: &str, length: Option<u64>) -> ProgressBar {
    let bar = match quiet_mode {
        true => ProgressBar::hidden(),
        false => {
            match length {
                Some(len) => ProgressBar::new(len),
                None => ProgressBar::new_spinner(),
            }
        }
    };

    bar.set_message(msg.to_string());
    match length.is_some() {
        true => bar.set_style(ProgressStyle::default_bar()
            .template("{msg} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} eta: {eta}").unwrap()
            .progress_chars("=> ")),
        false => bar.set_style(ProgressStyle::default_spinner()),
    };
    bar
}

pub fn resolve_output_path(url: &str, output_file: Option<&str>) -> String {
    let mut fname = match output_file {
        Some(name) => name.to_string(),
        None => url.split('/').last().unwrap_or("downloaded_file").to_string(),
    };

    if output_file.is_none() {
        if let Some(mut downloads_path) = dirs::download_dir() {
            downloads_path.push(&fname);
            fname = downloads_path.to_string_lossy().to_string();
        }
    }

    fname
}

pub fn is_media_url(url: &str) -> bool {
    url.contains("youtube.com") || 
    url.contains("youtu.be") || 
    url.contains("vimeo.com") || 
    url.contains("soundcloud.com") ||
    url.contains("bilibili.com") ||
    url.contains("tiktok.com") ||
    url.contains("twitter.com") ||
    url.contains("x.com") ||
    url.contains("twitch.tv")
}
