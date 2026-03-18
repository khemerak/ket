use crate::utils::{print, save_to_file, create_progress_bar};
use reqwest::blocking::Client;
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT};
use indicatif::HumanBytes;
use console::style;
use anyhow::{Context, Result};
use std::io::Read;

pub fn download(target: &str, output_file: Option<&str>, quiet_mode: bool) -> Result<()> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .context("Failed to create HTTP client")?;
    
    let mut resp = client.get(target)
        .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send()
        .context(format!("Failed to connect to the URL: {}", target))?;

    print(format!("HTTP request sent... {}", style(resp.status()).green()), quiet_mode);

    if resp.status().is_success() {
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
            Some(name) => name, 
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
        anyhow::bail!("Server returned an error: {}", resp.status());
    }
    
    Ok(())
}
