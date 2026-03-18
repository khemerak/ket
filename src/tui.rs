use anyhow::{Context, Result};
use console::style;
use dialoguer::{Confirm, Input, Select};
use crate::utils::{is_media_url, resolve_output_path};
use crate::http::download;
use crate::media::download_media;
use crate::install::install_software;

pub fn interactive_mode() -> Result<()> {
    // Print styled banner
    println!();
    println!("{}", style("  ██╗  ██╗███████╗████████╗").cyan().bold());
    println!("{}", style("  ██║ ██╔╝██╔════╝╚══██╔══╝").cyan().bold());
    println!("{}", style("  █████╔╝ █████╗     ██║   ").cyan().bold());
    println!("{}", style("  ██╔═██╗ ██╔══╝     ██║   ").cyan().bold());
    println!("{}", style("  ██║  ██╗███████╗   ██║   ").cyan().bold());
    println!("{}", style("  ╚═╝  ╚═╝╚══════╝   ╚═╝   ").cyan().bold());
    println!("        {} • {}", style("v1.0.1").dim(), style("Interactive Mode").white());
    println!();
    println!("  {}", style("Type a URL to download, or a package name to install (e.g. 'python'). Type 'q' to quit.").dim());
    println!();

    loop {
        let url: String = Input::new()
            .with_prompt(format!("  {}", style("📎 Paste URL or Package Name").green().bold()))
            .interact_text()
            .context("Failed to read URL input")?;

        let url = url.trim().to_string();

        if url.eq_ignore_ascii_case("q") || url.eq_ignore_ascii_case("quit") || url.eq_ignore_ascii_case("exit") {
            println!();
            println!("  {}", style("👋 Goodbye!").cyan().bold());
            break;
        }

        if url.is_empty() {
            println!("  {}", style("⚠  Please enter a valid input.").yellow());
            continue;
        }

        let is_url = url.starts_with("http://") || url.starts_with("https://");

        if is_url {
            let is_media = is_media_url(&url);

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

            let custom_name: String = Input::new()
                .with_prompt(format!("  {}", style("📁 Output filename (Enter to auto-detect)").green()))
                .default(String::new())
                .show_default(false)
                .interact_text()
                .unwrap_or_default();

            let custom_name = custom_name.trim().to_string();
            let output_file = if custom_name.is_empty() { None } else { Some(custom_name.as_str()) };

            let result = if audio_only || is_media {
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

        } else {
            // It's not a URL, so try to install it as software
            let result = install_software(&url);
            match result {
                Ok(()) => {
                    // install_software already prints success
                }
                Err(e) => {
                    println!();
                    println!("  {} {}", style("✘ Error:").red().bold(), e);
                }
            }
        }

        println!();

        let again = Confirm::new()
            .with_prompt(format!("  {}", style("Download/Install another?").green()))
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
