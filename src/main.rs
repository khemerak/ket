extern crate clap;

use clap::{Arg, App};
use reqwest::blocking::Client;
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT};
use indicatif::{ProgressBar, ProgressStyle, HumanBytes};
use console::style;
use dialoguer::{Input, Confirm, Select};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
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
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .context("Failed to create HTTP client")?;
    
    // 2. Add context to the network request
    let mut resp = client.get(target)
        .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send()
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

/// Check if ffmpeg is available on the system.
fn is_ffmpeg_installed() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn download_media(target: &str, output_file: Option<&str>, audio_only: bool, quiet_mode: bool) -> Result<()> {
    // We remove the inner printing of "Delegating to yt-dlp..." to keep the interface very clean for the user.
    // Check and offer to install yt-dlp if missing
    let available = check_and_install_ytdlp()?;
    if !available {
        anyhow::bail!("Cannot download media without yt-dlp. Aborting.");
    }

    let has_ffmpeg = is_ffmpeg_installed();

    let mut cmd = Command::new("yt-dlp");
    
    if audio_only {
        cmd.arg("-x").arg("--audio-format").arg("mp3");
    } else if has_ffmpeg {
        // ffmpeg available: download best video + best audio separately, merge into mp4
        cmd.arg("-f").arg("bestvideo[ext=mp4]+bestaudio[ext=m4a]/bestvideo+bestaudio/best");
        cmd.arg("--merge-output-format").arg("mp4");
    } else {
        // No ffmpeg: use a single pre-merged stream to avoid split files
        if !quiet_mode {
            println!("{}", style("⚠  ffmpeg not found — using single-stream mode (lower quality). Install ffmpeg for best results.").yellow());
        }
        cmd.arg("-f").arg("best[ext=mp4]/best");
    }

    // Ensure proper mp4 container (fixes TikTok and other sites with non-standard formats)
    if !audio_only {
        cmd.arg("--remux-video").arg("mp4");
    }
    
    // Try to let yt-dlp handle the filename automatically unless the user explicitly provided one.
    if let Some(file) = output_file {
        cmd.arg("-o").arg(file);
    } else {
        // If no user file is provided, yt-dlp's default is usually fine (Title [ID].ext).
        // However, if we want to save it to Downloads automatically, we can pass a format string
        // instead of a hardcoded name. This forces it to save in Downloads but keeps the auto-name.
        if let Some(downloads_path) = dirs::download_dir() {
            let out_template = downloads_path.join("%(title)s [%(id)s].%(ext)s");
            cmd.arg("-o").arg(out_template.to_string_lossy().to_string());
        }
    }
    
    // Force yt-dlp to output newline-delimited progress and no warnings
    // We will parse it to show a clean progress bar
    cmd.arg("--newline");
    cmd.arg("--no-warnings");
    if quiet_mode {
        cmd.arg("--quiet");
    }
    
    cmd.arg(target);
    
    if quiet_mode {
        // If entirely quiet, just spawn and wait
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        let mut child = cmd.spawn().context("Failed to spawn yt-dlp process")?;
        let status = child.wait().context("Failed to wait on yt-dlp")?;
        if !status.success() {
            anyhow::bail!("yt-dlp exited with an error status: {}", status);
        }
        return Ok(());
    }

    // Interactive mode: pipe stdout so we can parse progress
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null()); // Ignore all stderr noise (like VideoRemuxer etc)

    print(format!("{} {}", style("Starting download:").cyan().bold(), target), quiet_mode);

    let mut child = cmd.spawn().context("Failed to spawn yt-dlp process")?;
    let stdout = child.stdout.take().context("Failed to capture yt-dlp stdout")?;
    let reader = BufReader::new(stdout);

    // Create a generic spinner that will upgrade to a progress bar if we detect percentages
    let bar = ProgressBar::new_spinner();
    bar.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap());
    bar.set_message("Processing media (this may take a moment)...");

    let mut is_downloading = false;

    // Parse loop
    for line in reader.lines() {
        if let Ok(line) = line {
            // yt-dlp progress lines look like:
            // [download]   3.4% of 10.00MiB at  1.23MiB/s ETA 00:05
            if line.starts_with("[download]") && line.contains("%") {
                if !is_downloading {
                    is_downloading = true;
                    // Switch the bar to an actual visual progress bar
                    bar.set_length(1000); // 100.0% * 10
                    bar.set_style(ProgressStyle::default_bar()
                        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {percent}% {msg}")
                        .unwrap()
                        .progress_chars("=> "));
                }

                // Extract the percentage and speed clean
                let clean_line = line.replace("[download]", "").trim().to_string();
                let parts: Vec<&str> = clean_line.split('%').collect();
                
                if parts.len() >= 2 {
                    if let Ok(pct) = parts[0].trim().parse::<f64>() {
                        bar.set_position((pct * 10.0) as u64);
                    }
                    bar.set_message(parts[1].trim().to_string());
                } else {
                    bar.set_message(clean_line);
                }
            } else if line.starts_with("[youtube]") || line.starts_with("[info]") {
                // Ignore these lines completely
                bar.tick();
            }
        }
    }

    let status = child.wait().context("Failed to wait on yt-dlp process")?;
    
    if status.success() {
        bar.finish_with_message("Processing complete");
    } else {
        bar.finish_with_message(format!("{}", style("Download failed").red()));
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
/// (Used primarily for standard HTTP downloads, not yt-dlp, since yt-dlp does this better)
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
    println!("{}", style("  ██╗  ██╗███████╗████████╗").cyan().bold());
    println!("{}", style("  ██║ ██╔╝██╔════╝╚══██╔══╝").cyan().bold());
    println!("{}", style("  █████╔╝ █████╗     ██║   ").cyan().bold());
    println!("{}", style("  ██╔═██╗ ██╔══╝     ██║   ").cyan().bold());
    println!("{}", style("  ██║  ██╗███████╗   ██║   ").cyan().bold());
    println!("{}", style("  ╚═╝  ╚═╝╚══════╝   ╚═╝   ").cyan().bold());
    println!("        {} • {}", style("v1.0.0").dim(), style("Interactive Mode").white());
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

        // Perform the download
        let result = if audio_only || is_media {
            // Let yt-dlp handle the default filename (it adds the correct extension)
            download_media(&url, output_file, audio_only, false)
        } else {
            let fname = resolve_output_path(&url, output_file);
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

    if audio_only || force_media || is_media_url(url) {
        // Let yt-dlp determine the filename if output_file is None
        download_media(url, output_file, audio_only, false)?;
    } else {
        let fname = resolve_output_path(url, output_file);
        download(url, Some(&fname), false)?;
    }
    
    Ok(())
}
