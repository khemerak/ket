use crate::utils::print;
use anyhow::{Context, Result};
use console::style;
use directories::ProjectDirs;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

// ---------------------------------------------------------------------------
// Platform-specific constants
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
const YTDLP_BINARY_NAME: &str = "yt-dlp.exe";
#[cfg(target_os = "windows")]
const YTDLP_DOWNLOAD_URL: &str =
    "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";

#[cfg(not(target_os = "windows"))]
const YTDLP_BINARY_NAME: &str = "yt-dlp";
#[cfg(not(target_os = "windows"))]
const YTDLP_DOWNLOAD_URL: &str =
    "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp";

// ---------------------------------------------------------------------------
// ensure_ytdlp_binary — The core "first-run download" function
// ---------------------------------------------------------------------------

/// Ensure the yt-dlp binary exists in ket's own app-data directory.
///
/// Returns the full `PathBuf` to the managed binary. On first run it
/// automatically downloads the latest standalone release from GitHub.
///
///   Windows: `%APPDATA%\ket\data\yt-dlp.exe`
///   Linux  : `~/.local/share/ket/yt-dlp`
///   macOS  : `~/Library/Application Support/com.ket.ket/yt-dlp`
pub fn ensure_ytdlp_binary() -> Result<PathBuf> {
    let proj = ProjectDirs::from("com", "ket", "ket")
        .context("Could not determine app data directory for ket")?;

    let data_dir = proj.data_dir();
    fs::create_dir_all(data_dir)
        .context(format!("Failed to create ket data directory: {}", data_dir.display()))?;

    let binary_path = data_dir.join(YTDLP_BINARY_NAME);

    // ── Already downloaded? Quick-verify it can run. ──────────────
    if binary_path.exists() {
        let ok = Command::new(&binary_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if ok {
            return Ok(binary_path);
        }
        // Binary exists but is broken — re-download.
        let _ = fs::remove_file(&binary_path);
    }

    // ── First-run download ────────────────────────────────────────

    let client = Client::builder()
        .user_agent("ket-downloader")
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .context("Failed to create HTTP client for yt-dlp download")?;

    let mut resp = client
        .get(YTDLP_DOWNLOAD_URL)
        .send()
        .context("Failed to reach GitHub releases for yt-dlp")?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "GitHub returned HTTP {} when downloading yt-dlp",
            resp.status()
        );
    }

    let mut file = fs::File::create(&binary_path)
        .context(format!("Failed to create file: {}", binary_path.display()))?;
    std::io::copy(&mut resp, &mut file)
        .context("Failed to write yt-dlp binary to disk")?;

    // ── Unix: make it executable ──────────────────────────────────
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&binary_path, perms)
            .context("Failed to set executable permissions on yt-dlp")?;
    }

    // ── Verify (with retry — Windows AV may briefly lock the file) ─
    for _attempt in 1..=3 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        match Command::new(&binary_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(s) if s.success() => {
                break;
            }
            _ => {}
        }
    }

    Ok(binary_path)
}

// ---------------------------------------------------------------------------

pub fn is_ffmpeg_installed() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// download_media — now uses the self-managed binary path
// ---------------------------------------------------------------------------

pub fn download_media(
    target: &str,
    output_file: Option<&str>,
    audio_only: bool,
    quiet_mode: bool,
) -> Result<()> {
    // Get (or download) ket's own yt-dlp binary.
    let ytdlp_path = ensure_ytdlp_binary()?;

    let has_ffmpeg = is_ffmpeg_installed();

    let mut cmd = Command::new(&ytdlp_path);

    if audio_only {
        cmd.arg("-x").arg("--audio-format").arg("mp3");
    } else if has_ffmpeg {
        cmd.arg("-f")
            .arg("bestvideo[height<=1080][ext=mp4]+bestaudio[ext=m4a]/bestvideo[height<=1080]+bestaudio/best[height<=1080]/best");
        cmd.arg("--merge-output-format").arg("mp4");
    } else {
        // println!("{}", style("  ⚠ Note: ffmpeg is not installed. Videos are capped at 720p/360p (single-stream mode) because YouTube requires ffmpeg to merge 1080p video & audio. Install ffmpeg and add it to your PATH for full 1080p support.").yellow().dim());
        println!();
        cmd.arg("-f").arg("best[height<=1080][ext=mp4]/best[height<=1080]/best");
    }

    if !audio_only {
        cmd.arg("--remux-video").arg("mp4");
    }

    if let Some(file) = output_file {
        cmd.arg("-o").arg(file);
    } else if let Some(downloads_path) = dirs::download_dir() {
        let out_template = downloads_path.join("%(title)s [%(id)s].%(ext)s");
        cmd.arg("-o")
            .arg(out_template.to_string_lossy().to_string());
    }

    cmd.arg("--newline");
    cmd.arg("--no-warnings");
    if quiet_mode {
        cmd.arg("--quiet");
    }

    cmd.arg(target);

    if quiet_mode {
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        let mut child = cmd.spawn().context("Failed to spawn yt-dlp process")?;
        let status = child.wait().context("Failed to wait on yt-dlp")?;
        if !status.success() {
            anyhow::bail!("yt-dlp exited with an error status: {}", status);
        }
        return Ok(());
    }

    // Interactive: pipe stdout so we can parse progress
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());

    print(
        format!(
            "{} {}",
            style("Starting download:").cyan().bold(),
            target
        ),
        quiet_mode,
    );

    let mut child = cmd.spawn().context("Failed to spawn yt-dlp process")?;
    let stdout = child
        .stdout
        .take()
        .context("Failed to capture yt-dlp stdout")?;
    let reader = BufReader::new(stdout);

    let bar = ProgressBar::new_spinner();
    bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    bar.set_message("Processing media (this may take a moment)...");

    let mut is_downloading = false;

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.starts_with("[download]") && line.contains("%") {
                if !is_downloading {
                    is_downloading = true;
                    bar.set_length(1000);
                    bar.set_style(
                        ProgressStyle::default_bar()
                            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {percent}% {msg}")
                            .unwrap()
                            .progress_chars("=> "),
                    );
                }

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
