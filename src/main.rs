extern crate clap;

use clap::{Arg, App};
use reqwest::blocking::Client;
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
use indicatif::{ProgressBar, ProgressStyle, HumanBytes};
use console::style;
use std::fs::File;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use anyhow::{Context, Result};

// Custom print function from the tutorial
fn print(msg: String, quiet_mode: bool) {
    if !quiet_mode {
        println!("{}", msg);
    }
}

// Missing helper function to save the file
fn save_to_file(buf: &[u8], fname: &str) -> Result<()> {
    let mut file = File::create(fname).context(format!("Failed to create output file: {}", fname))?;
    file.write_all(buf).context("Failed to write data to the file")?;
    Ok(())
}

fn create_progress_bar(quiet_mode: bool, msg: &str, length: Option<u64>) -> ProgressBar {
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

fn download(target: &str, output_file: Option<&str>, quiet_mode: bool) -> Result<()> {
    let client = Client::new();
    
    // 2. Add context to the network request
    let mut resp = client.get(target).send()
        .context(format!("Failed to connect to the URL: {}", target))?;

    print(format!("HTTP request sent... {}", style(resp.status()).green()), quiet_mode);

    if resp.status().is_success() {
        // ... (Keep all your existing header parsing and progress bar logic here, it is fine!) ...
        let headers = resp.headers().clone();
        
        let ct_len = headers.get(CONTENT_LENGTH)
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.parse::<u64>().ok());
            
        let ct_type = headers.get(CONTENT_TYPE)
            .and_then(|val| val.to_str().ok())
            .unwrap_or("unknown");

        match ct_len {
            Some(len) => {
                print(format!("Length: {} ({})",
                    style(len).green(),
                    style(HumanBytes(len)).red()), quiet_mode);
            },
            None => {
                print(format!("Length: {}", style("unknown").red()), quiet_mode);
            },
        }

        print(format!("Type: {}", style(ct_type).green()), quiet_mode);
        
        let fname = match output_file {
            // If the user provided the -O flag, use their filename
            Some(name) => name, 
            // If they didn't, guess it from the URL like before
            None => target.split('/').last().unwrap_or("downloaded_file"),
        };
        print(format!("Saving to: {}", style(fname).green()), quiet_mode);

        let chunk_size = match ct_len {
            Some(x) => (x / 99) as usize,
            None => 1024usize,
        };

        let chunk_size = std::cmp::max(chunk_size, 1024);
        let mut buf = Vec::new();
        let bar = create_progress_bar(quiet_mode, fname, ct_len);

        loop {
            let mut buffer = vec![0; chunk_size];
            // 3. Add context to reading the network stream
            let bcount = resp.read(&mut buffer[..])
                .context("Connection dropped while downloading the file")?;
                
            buffer.truncate(bcount);
            
            if !buffer.is_empty() {
                buf.extend_from_slice(&buffer);
                bar.inc(bcount as u64);
            } else {
                break;
            }
        }
        
        bar.finish();
        save_to_file(&buf, fname)?;
    } else {
        // 4. Handle non-200 HTTP codes (like 404 Not Found) cleanly
        anyhow::bail!("Server returned an error: {}", resp.status());
    }
    
    Ok(())
}

fn download_media(target: &str, output_file: Option<&str>, audio_only: bool, quiet_mode: bool) -> Result<()> {
    print("Detected media URL. Delegating to yt-dlp...".to_string(), quiet_mode);
    
    // Check if yt-dlp is installed
    let yt_dlp_check = Command::new("yt-dlp")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
        
    if yt_dlp_check.is_err() {
        anyhow::bail!("'yt-dlp' is not installed or not in PATH. Please install it to download media from this source.");
    }

    let mut cmd = Command::new("yt-dlp");
    
    if audio_only {
        cmd.arg("-x").arg("--audio-format").arg("mp3");
    } else {
        // Enforce mp4 for video files per feature request
        cmd.arg("-f").arg("bestvideo[ext=mp4]+bestaudio[ext=m4a]/mp4");
        cmd.arg("--merge-output-format").arg("mp4");
    }
    
    if let Some(file) = output_file {
        cmd.arg("-o").arg(file);
    }
    
    if quiet_mode {
        cmd.arg("--quiet");
    }
    
    cmd.arg(target);
    
    let mut child = cmd.spawn().context("Failed to spawn yt-dlp process")?;
    let status = child.wait().context("Failed to wait on yt-dlp process")?;
    
    if !status.success() {
        anyhow::bail!("yt-dlp exited with an error status: {}", status);
    }
    
    Ok(())
}

fn is_media_url(url: &str) -> bool {
    // A simple heuristic for known media sites
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

fn main() -> Result<()> {
    let matches = App::new("Ket")
        .version("0.1.0")
        .author("Pav Khemerak <pavkhemerak.official@gmail.com>")
        .about("wget clone written in Rust, renamed to ket")
        .arg(Arg::with_name("URL")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("url to download"))
        .arg(Arg::with_name("OUTPUT")
                .short("O")
                .long("output")
                .takes_value(true)
                .help("write documents to FILE"))
        .arg(Arg::with_name("MEDIA")
                .short("m")
                .long("media")
                .help("Force fallback to yt-dlp for media downloading"))
        .arg(Arg::with_name("AUDIO")
                .short("a")
                .long("audio")
                .help("Download audio only (using yt-dlp)"))
        .get_matches();
        
    let url = matches.value_of("URL").unwrap();
    let output_file = matches.value_of("OUTPUT");
    let force_media = matches.is_present("MEDIA");
    let audio_only = matches.is_present("AUDIO");
    
    println!("Target: {}", url);

    // Determine the baseline filename from either output_file or URL
    let mut fname = match output_file {
        Some(name) => name.to_string(),
        None => url.split('/').last().unwrap_or("downloaded_file").to_string(),
    };

    // Use dirs to get the User Downloads directory
    if output_file.is_none() {
        if let Some(mut downloads_path) = dirs::download_dir() {
            downloads_path.push(&fname);
            fname = downloads_path.to_string_lossy().to_string();
        }
    }

    if audio_only || force_media || is_media_url(url) {
        // We pass the potentially modified `fname`
        download_media(url, Some(&fname), audio_only, false)?;
    } else {
        // Note: we need to update standard download to receive `output_file` as an Option still
        download(url, Some(&fname), false)?;
    }
    
    Ok(())
}
