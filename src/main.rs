extern crate clap;

use clap::{Arg, App};
use reqwest::blocking::Client;
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
use indicatif::{ProgressBar, ProgressStyle, HumanBytes};
use console::style;
use dialoguer::{Input, Confirm, Select};
use std::fs::File;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::env;
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

/// Check if yt-dlp is available; if not, offer to install it automatically.
/// Returns Ok(true) if yt-dlp is now available, Ok(false) if user declined.
fn check_and_install_ytdlp() -> Result<bool> {
    // Check if yt-dlp is already installed
    let check = Command::new("yt-dlp")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if check.is_ok() {
        return Ok(true); // already installed
    }

    println!();
    println!("{}", style("⚠  yt-dlp is not installed or not in PATH.").yellow().bold());
    println!("   yt-dlp is required to download media from YouTube, TikTok, Twitter, etc.");
    println!();

    let install = Confirm::new()
        .with_prompt("Would you like to install yt-dlp now?")
        .default(true)
        .interact()
        .unwrap_or(false);

    if !install {
        println!("{}", style("Skipped. You can install it manually: https://github.com/yt-dlp/yt-dlp#installation").dim());
        return Ok(false);
    }

    // Strategy 1: Try pip install
    println!("{}", style("→ Attempting: pip install yt-dlp ...").cyan());
    let pip_result = Command::new("pip")
        .args(["install", "-U", "yt-dlp"])
        .status();

    if let Ok(status) = pip_result {
        if status.success() {
            println!("{}", style("✔ yt-dlp installed successfully via pip!").green().bold());
            return Ok(true);
        }
    }

    println!("{}", style("  pip install failed or pip not found. Trying standalone download...").yellow());

    // Strategy 2: Download standalone binary (Windows only for now)
    if cfg!(target_os = "windows") {
        let download_url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";

        // Determine install path: same directory as the ket executable
        let exe_dir = env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let ytdlp_path = exe_dir.join("yt-dlp.exe");

        println!("{}", style(format!("→ Downloading yt-dlp.exe to {} ...", ytdlp_path.display())).cyan());

        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .context("Failed to create HTTP client")?;

        let mut resp = client.get(download_url).send()
            .context("Failed to download yt-dlp.exe from GitHub")?;

        if resp.status().is_success() {
            let mut file = File::create(&ytdlp_path)
                .context(format!("Failed to create file: {}", ytdlp_path.display()))?;
            std::io::copy(&mut resp, &mut file)
                .context("Failed to write yt-dlp.exe")?;

            println!("{}", style(format!("✔ yt-dlp.exe saved to {}", ytdlp_path.display())).green().bold());

            // Verify it works
            let verify = Command::new(&ytdlp_path)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();

            if verify.is_ok() {
                println!("{}", style("✔ yt-dlp is ready to use!").green().bold());
                return Ok(true);
            }
        } else {
            println!("{}", style(format!("Download failed: HTTP {}", resp.status())).red());
        }
    } else {
        // For Linux/macOS: try curl download
        println!("{}", style("→ Attempting: curl download of yt-dlp ...").cyan());
        let curl_result = Command::new("sh")
            .args(["-c", "curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && chmod +x /usr/local/bin/yt-dlp"])
            .status();

        if let Ok(status) = curl_result {
            if status.success() {
                println!("{}", style("✔ yt-dlp installed successfully!").green().bold());
                return Ok(true);
            }
        }

        // Fallback: try to install to user home
        let home_bin = dirs::home_dir().map(|h| h.join(".local").join("bin"));
        if let Some(bin_dir) = home_bin {
            let _ = std::fs::create_dir_all(&bin_dir);
            let ytdlp_path = bin_dir.join("yt-dlp");
            let cmd = format!(
                "curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o {} && chmod +x {}",
                ytdlp_path.display(), ytdlp_path.display()
            );
            let result = Command::new("sh").args(["-c", &cmd]).status();
            if let Ok(status) = result {
                if status.success() {
                    println!("{}", style(format!("✔ yt-dlp installed to {}", ytdlp_path.display())).green().bold());
                    println!("{}", style("  Make sure ~/.local/bin is in your PATH.").dim());
                    return Ok(true);
                }
            }
        }
    }

    println!("{}", style("✘ Could not install yt-dlp automatically.").red().bold());
    println!("  Please install it manually: {}", style("https://github.com/yt-dlp/yt-dlp#installation").underlined());
    Ok(false)
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
    
    // Check and offer to install yt-dlp if missing
    let available = check_and_install_ytdlp()?;
    if !available {
        anyhow::bail!("Cannot download media without yt-dlp. Aborting.");
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

/// Resolve the final output path from url and optional user-provided output name.
fn resolve_output_path(url: &str, output_file: Option<&str>) -> String {
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

/// Interactive terminal UI mode — launched when ket.exe is double-clicked with no arguments.
fn interactive_mode() -> Result<()> {
    // Print styled banner
    println!();
    println!("  {}", style("┌─────────────────────────────────────────┐").cyan());
    println!("  {}", style("│                                         │").cyan());
    println!("  {}  {}  {}", style("│").cyan(), style("  ket 🦀  — Download Anything Fast  ").white().bold(), style("│").cyan());
    println!("  {}  {}  {}", style("│").cyan(), style("        v1.0.0 • Interactive Mode    ").dim(), style("│").cyan());
    println!("  {}", style("│                                         │").cyan());
    println!("  {}", style("└─────────────────────────────────────────┘").cyan());
    println!();
    println!("  {}", style("Type a URL to start downloading. Type 'q' to quit.").dim());
    println!();

    loop {
        // Prompt for URL
        let url: String = Input::new()
            .with_prompt(format!("  {}", style("📎 Paste URL").green().bold()))
            .interact_text()
            .context("Failed to read URL input")?;

        let url = url.trim().to_string();

        if url.eq_ignore_ascii_case("q") || url.eq_ignore_ascii_case("quit") || url.eq_ignore_ascii_case("exit") {
            println!();
            println!("  {}", style("👋 Goodbye!").cyan().bold());
            break;
        }

        if url.is_empty() {
            println!("  {}", style("⚠  Please enter a valid URL.").yellow());
            continue;
        }

        // Detect if it's a media URL
        let is_media = is_media_url(&url);

        // Ask for download type if it's a media URL
        let audio_only = if is_media {
            let options = vec!["🎬 Video (MP4)", "🎵 Audio only (MP3)"];
            let selection = Select::new()
                .with_prompt(format!("  {}", style("Download type").green()))
                .items(&options)
                .default(0)
                .interact()
                .unwrap_or(0);
            selection == 1
        } else {
            false
        };

        // Ask for optional custom filename
        let custom_name: String = Input::new()
            .with_prompt(format!("  {}", style("📁 Output filename (Enter to auto-detect)").green()))
            .default(String::new())
            .show_default(false)
            .interact_text()
            .unwrap_or_default();

        let custom_name = custom_name.trim().to_string();
        let output_file = if custom_name.is_empty() { None } else { Some(custom_name.as_str()) };

        let fname = resolve_output_path(&url, output_file);

        println!();
        println!("  {} {}", style("Target:").bold(), style(&url).underlined());
        println!();

        // Perform the download
        let result = if audio_only || is_media {
            download_media(&url, Some(&fname), audio_only, false)
        } else {
            download(&url, Some(&fname), false)
        };

        match result {
            Ok(()) => {
                println!();
                println!("  {}", style("✔ Download complete!").green().bold());
            }
            Err(e) => {
                println!();
                println!("  {} {}", style("✘ Error:").red().bold(), e);
            }
        }

        println!();

        // Ask if user wants to download another
        let again = Confirm::new()
            .with_prompt(format!("  {}", style("Download another?").green()))
            .default(true)
            .interact()
            .unwrap_or(false);

        if !again {
            println!();
            println!("  {}", style("👋 Goodbye!").cyan().bold());
            break;
        }
        println!();
    }

    Ok(())
}

fn main() -> Result<()> {
    let matches = App::new("Ket")
        .version("0.1.0")
        .author("Pav Khemerak <pavkhemerak.official@gmail.com>")
        .about("wget clone written in Rust, renamed to ket")
        .arg(Arg::with_name("URL")
                .required(false)  // No longer required — interactive mode when missing
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
    
    // If no URL provided, launch interactive mode
    let url = match matches.value_of("URL") {
        Some(u) => u,
        None => {
            return interactive_mode();
        }
    };

    let output_file = matches.value_of("OUTPUT");
    let force_media = matches.is_present("MEDIA");
    let audio_only = matches.is_present("AUDIO");
    
    println!("Target: {}", url);

    let fname = resolve_output_path(url, output_file);

    if audio_only || force_media || is_media_url(url) {
        download_media(url, Some(&fname), audio_only, false)?;
    } else {
        download(url, Some(&fname), false)?;
    }
    
    Ok(())
}
