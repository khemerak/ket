use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use reqwest::header::CONTENT_LENGTH;
use std::io::Read;

struct PackageInfo {
    name: &'static str,
    display_name: &'static str,
    url: &'static str,
    filename: &'static str,
}

const PACKAGES: &[PackageInfo] = &[
    PackageInfo {
        name: "python",
        display_name: "Python 3",
        url: "https://www.python.org/ftp/python/3.12.7/python-3.12.7-amd64.exe",
        filename: "python-3.12.7-amd64.exe",
    },
    PackageInfo {
        name: "git",
        display_name: "Git",
        url: "https://github.com/git-for-windows/git/releases/download/v2.47.1.windows.2/Git-2.47.1.2-64-bit.exe",
        filename: "Git-2.47.1.2-64-bit.exe",
    },
    PackageInfo {
        name: "nodejs",
        display_name: "Node.js LTS",
        url: "https://nodejs.org/dist/v22.12.0/node-v22.12.0-x64.msi",
        filename: "node-v22.12.0-x64.msi",
    },
    PackageInfo {
        name: "node",
        display_name: "Node.js LTS",
        url: "https://nodejs.org/dist/v22.12.0/node-v22.12.0-x64.msi",
        filename: "node-v22.12.0-x64.msi",
    },
    PackageInfo {
        name: "vscode",
        display_name: "Visual Studio Code",
        url: "https://update.code.visualstudio.com/latest/win32-x64/stable",
        filename: "VSCodeSetup-x64.exe",
    },
    PackageInfo {
        name: "ffmpeg",
        display_name: "FFmpeg",
        url: "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip",
        filename: "ffmpeg-release-essentials.zip",
    },
    PackageInfo {
        name: "7zip",
        display_name: "7-Zip",
        url: "https://www.7-zip.org/a/7z2409-x64.exe",
        filename: "7z2409-x64.exe",
    },
    PackageInfo {
        name: "vlc",
        display_name: "VLC Media Player",
        url: "https://get.videolan.org/vlc/3.0.21/win64/vlc-3.0.21-win64.exe",
        filename: "vlc-3.0.21-win64.exe",
    },
    PackageInfo {
        name: "rust",
        display_name: "Rust (rustup)",
        url: "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe",
        filename: "rustup-init.exe",
    },
];

fn find_package(query: &str) -> Option<&'static PackageInfo> {
    let q = query.to_lowercase();
    PACKAGES.iter().find(|p| p.name == q)
}

fn list_available_packages() {
    println!();
    println!("  {}", style("Available packages:").cyan().bold());
    println!();
    // Deduplicate display (node/nodejs map to same thing)
    let mut seen = Vec::new();
    for pkg in PACKAGES {
        if !seen.contains(&pkg.display_name) {
            println!(
                "    {} {:12} {}",
                style("•").cyan(),
                style(pkg.name).green().bold(),
                style(pkg.display_name).dim()
            );
            seen.push(pkg.display_name);
        }
    }
    println!();
}

pub fn install_software(package_name: &str) -> Result<()> {
    let pkg = match find_package(package_name) {
        Some(p) => p,
        None => {
            println!();
            println!(
                "  {} Package '{}' not found in ket's registry.",
                style("⚠").yellow().bold(),
                style(package_name).red()
            );
            list_available_packages();
            println!(
                "  {}",
                style("Tip: You can also paste a direct URL to download any file.").dim()
            );
            return Ok(());
        }
    };

    println!();
    println!(
        "{} {} ({})",
        style("📦 Downloading:").cyan().bold(),
        style(pkg.display_name).green().bold(),
        style(pkg.filename).dim()
    );

    // Download to user's Downloads directory (or current dir as fallback)
    let dest_dir = dirs::download_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    std::fs::create_dir_all(&dest_dir).ok();
    let installer_path = dest_dir.join(pkg.filename);

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .context("Failed to create HTTP client")?;

    let mut resp = client
        .get(pkg.url)
        .send()
        .context(format!("Failed to download {} from {}", pkg.display_name, pkg.url))?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "Server returned HTTP {} when downloading {}",
            resp.status(),
            pkg.display_name
        );
    }

    let total_size = resp
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());

    let bar = match total_size {
        Some(len) => {
            let b = ProgressBar::new(len);
            b.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} eta: {eta}")
                    .unwrap()
                    .progress_chars("=> "),
            );
            b
        }
        None => {
            let b = ProgressBar::new_spinner();
            b.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg} {bytes}")
                    .unwrap(),
            );
            b.set_message("Downloading...");
            b
        }
    };

    let mut file = std::fs::File::create(&installer_path)
        .context(format!("Failed to create temp file: {}", installer_path.display()))?;

    let mut downloaded: u64 = 0;
    let mut buffer = vec![0u8; 8192];

    loop {
        let bytes_read = resp
            .read(&mut buffer)
            .context("Connection dropped while downloading installer")?;
        if bytes_read == 0 {
            break;
        }
        std::io::Write::write_all(&mut file, &buffer[..bytes_read])
            .context("Failed to write installer to disk")?;
        downloaded += bytes_read as u64;
        bar.set_position(downloaded);
    }

    bar.finish_with_message("Download complete");

    println!(
        "  {} Saved to {}",
        style("✔").green().bold(),
        style(installer_path.display()).green()
    );
/*
    // Run the installer
    if pkg.silent_args.is_empty() {
        // For archives (like ffmpeg zip), just tell the user where it is
        println!();
        println!(
            "  {} {} downloaded to: {}",
            style("📂").cyan(),
            pkg.display_name,
            style(installer_path.display()).green()
        );
        println!(
            "  {}",
            style("Extract it manually and add to your PATH.").dim()
        );
        return Ok(());
    }

    println!();
    println!(
        "{} Installing {} ...",
        style("→").cyan().bold(),
        pkg.display_name
    );

    // For .msi files, use msiexec
    let is_msi = pkg.filename.ends_with(".msi");

    let status = if is_msi {
        let mut cmd = Command::new("msiexec");
        cmd.arg("/i").arg(&installer_path);
        for arg in pkg.silent_args {
            cmd.arg(arg);
        }
        cmd.status()
            .context(format!("Failed to run msiexec for {}", pkg.display_name))?
    } else {
        let mut cmd = Command::new(&installer_path);
        for arg in pkg.silent_args {
            cmd.arg(arg);
        }
        cmd.status()
            .context(format!("Failed to run installer for {}", pkg.display_name))?
    };

    if status.success() {
        println!();
        println!(
            "  {} {} installed successfully!",
            style("✔").green().bold(),
            style(pkg.display_name).green()
        );
        println!(
            "  {}",
            style("You may need to restart your terminal for PATH changes to take effect.").dim()
        );
    } else {
        println!();
        println!(
            "  {} Installer exited with: {}",
            style("⚠").yellow().bold(),
            status
        );
        println!(
            "  {}",
            style("The package may have installed partially. Check manually.").dim()
        );
    }
*/
    Ok(())
}
