mod utils;
mod http;
mod media;
mod install;
mod tui;

extern crate clap;

use clap::{Arg, App};
use anyhow::Result;
use crate::utils::{is_media_url, resolve_output_path};
use crate::http::download;
use crate::media::download_media;
use crate::install::install_software;
use crate::tui::interactive_mode;

fn main() -> Result<()> {
    let matches = App::new("Ket")
        .version("1.0.1")
        .author("Pav Khemerak <pavkhemerak.official@gmail.com>")
        .about("wget clone written in Rust, renamed to ket")
        .arg(Arg::with_name("URL_OR_PACKAGE")
                .required(false) 
                .takes_value(true)
                .index(1)
                .help("URL to download, or software package to install"))
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
    
    let target = match matches.value_of("URL_OR_PACKAGE") {
        Some(u) => u.trim(),
        None => {
            return interactive_mode();
        }
    };

    let output_file = matches.value_of("OUTPUT");
    let force_media = matches.is_present("MEDIA");
    let audio_only = matches.is_present("AUDIO");
    
    let is_url = target.starts_with("http://") || target.starts_with("https://");

    if is_url {
        println!("Target URL: {}", target);
        if audio_only || force_media || is_media_url(target) {
            download_media(target, output_file, audio_only, false)?;
        } else {
            let fname = resolve_output_path(target, output_file);
            download(target, Some(&fname), false)?;
        }
    } else {
        // Not a URL -> Attempt to install via winget
        install_software(target)?;
    }
    
    Ok(())
}
