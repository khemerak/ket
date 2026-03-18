```text
██╗  ██╗███████╗████████╗
██║ ██╔╝██╔════╝╚══██╔══╝
█████╔╝ █████╗     ██║   
██╔═██╗ ██╔══╝     ██║   
██║  ██╗███████╗   ██║   
╚═╝  ╚═╝╚══════╝   ╚═╝   
```  

> A `wget` clone written in Rust — built for speed, simplicity, and beautiful terminal UX.

---

## 📖 About

`ket` is a minimal command-line file downloader inspired by GNU `wget`. It takes a URL, makes an HTTP GET request, and saves the response body to disk — all while showing a beautiful, real-time progress bar in the terminal.

The project demonstrates the following Rust concepts and crates:

| Concept | What it teaches |
|---|---|
| CLI argument parsing | How to define flags and positional args with `clap` |
| HTTP client | How to make blocking HTTP requests with `reqwest` |
| Reading response headers | How to extract `Content-Length` and `Content-Type` |
| Streaming I/O | How to read a response body in chunks efficiently |
| Progress bars | How to render terminal UX with `indicatif` |
| Colorized output | How to style terminal text with `console` |
| Error handling | How to propagate and enrich errors using `anyhow` |
| File I/O | How to create and write binary data to a file |
| Interactive TUI | How to build an interactive terminal UI with `dialoguer` |

---

## ✨ Features

- 📥 Download any file from a given HTTP/HTTPS URL
- 🎬 Download videos (`.mp4`) and audio (`.mp3`) from media sites (YouTube, Vimeo, etc.) using `yt-dlp` integration (`-m` / `-a` flags)
- 🔧 **Self-managed yt-dlp** — `ket` automatically downloads and manages its own `yt-dlp` binary on first use (no PATH setup, no pip, no manual install)
- 🖥️ **Interactive Mode** — double-click `ket.exe` (or run without arguments) to launch a beautiful terminal UI for pasting URLs, choosing download type, and more
- 📊 Real-time progress bar with elapsed time, speed, and ETA (when `Content-Length` is available)
- 🚀 **Seamless yt-dlp progress monitoring** — parses `yt-dlp` output silently behind a clean, single-line progress bar
- 🌀 Spinner fallback for responses with unknown content length
- 🎨 Colorized terminal output (green for OK, red for sizes/warnings)
- 📁 Custom output filename via `-O` / `--output` flag (defaults to system `Downloads` directory)
- ⚠️ Descriptive error messages on connection failures, HTTP errors, and I/O errors
- 🔇 Quiet mode support (hidden progress bar when silent mode is enabled internally)

---

## 📥 Installation

There are multiple ways to install `ket` onto your system.

### Option 1: Download Pre-compiled Binary (Recommended)
You can download the pre-compiled executable directly from the **[Releases](../../releases)** tab on GitHub. Just download the appropriate binary for your OS (Windows/macOS/Linux) and add it to your system's `PATH`.

### Option 2: Install via Cargo
If you are a Rust developer and have [Cargo installed](https://www.rust-lang.org/tools/install), you can build and install `ket` directly from source into your `~/.cargo/bin` folder:

```bash
cargo install --path .
```
*(Make sure `~/.cargo/bin` is in your environment `PATH`)*

### Option 3: Compile from Source Manually
To build the repository from scratch:

```bash
cargo build --release
```
The compiled binary will be located at `target/release/ket.exe` (Windows) or `target/release/ket` (Linux/macOS).

---

## 🚀 Usage

### CLI Mode (traditional)

```bash
# Basic download — saves file using the name from the URL
ket https://example.com/file.zip

# Save to a custom filename in the current directory
ket https://example.com/file.zip -O my_file.zip
ket https://example.com/file.zip --output my_file.zip

# Download a YouTube video (automatically uses yt-dlp and saves as .mp4)
ket https://www.youtube.com/watch?v=... 

# Download only the audio from a YouTube video (saves as .mp3)
ket https://www.youtube.com/watch?v=... -a

# Force standard URL to use yt-dlp fallback
ket https://example.com/video.mp4 -m
```

### 🖥️ Interactive Mode (new!)

Simply run `ket` with **no arguments** — or double-click `ket.exe` on Windows — to launch the interactive terminal UI:

```text
  ██╗  ██╗███████╗████████╗
  ██║ ██╔╝██╔════╝╚══██╔══╝
  █████╔╝ █████╗     ██║   
  ██╔═██╗ ██╔══╝     ██║   
  ██║  ██╗███████╗   ██║   
  ╚═╝  ╚═╝╚══════╝   ╚═╝   
        v1.0.1 • Interactive Mode

  Type a URL to start downloading. Type 'q' to quit.

  📎 Paste URL: https://www.youtube.com/watch?v=...
  Download type: 🎬 Video (MP4) / 🎵 Audio only (MP3)
  📁 Output filename (Enter to auto-detect):

  Processing media... [00:03] [=================>  ] 90% eta: 00:01
  ✔ Download complete!

  Download another? (y/n)
```

**Interactive mode features:**
- Paste any URL directly
- Auto-detects media sites (YouTube, TikTok, Twitter, etc.)
- Choose between video or audio-only for media downloads
- Optional custom filename
- Loop to download multiple files in one session
- Type `q`, `quit`, or `exit` to close

### 🔧 Self-Managed yt-dlp Binary

`ket` manages its own `yt-dlp` binary — you never need to install it yourself or touch your PATH.

On first media download, `ket` will:

1. Resolve your platform's app data directory using the `directories` crate
2. Download the latest standalone `yt-dlp` release from GitHub
3. Set executable permissions (Unix)
4. Verify the binary works

The binary is stored at:

| OS | Location |
|---|---|
| Windows | `%APPDATA%\ket\data\yt-dlp.exe` |
| Linux | `~/.local/share/ket/yt-dlp` |
| macOS | `~/Library/Application Support/com.ket.ket/yt-dlp` |

> **Note:** For regular HTTP/HTTPS file downloads, `yt-dlp` is **not required**. The auto-download only happens when downloading from media sites like YouTube, TikTok, etc.

### Help

```bash
ket --help
```

```
USAGE:
    ket [OPTIONS] [URL]

ARGS:
    <URL>    url to download (omit to launch interactive mode)

OPTIONS:
    -O, --output <FILE>    write documents to FILE
    -m, --media            Force fallback to yt-dlp for media downloading
    -a, --audio            Download audio only (using yt-dlp)
    -h, --help             Print help information
    -V, --version          Print version information
```

---

## 🏗️ Project Structure

```
ket/
├── src/
│   ├── main.rs         # Entry point and CLI argument parsing
│   ├── http.rs         # Standard HTTP file downloads (reqwest)
│   ├── media.rs        # yt-dlp integration and self-managed binary
│   ├── install.rs      # Software installation via winget
│   ├── tui.rs          # Interactive terminal UI mode
│   └── utils.rs        # Shared utilities (progress bars, helpers)
├── Cargo.toml          # Project metadata and dependencies
├── Cargo.lock          # Exact locked dependency versions
└── .gitignore          # Excludes /target from version control
```

---

## 📦 Dependencies

Defined in [`Cargo.toml`](./Cargo.toml):

| Crate | Version | Purpose |
|---|---|---|
| [`clap`](https://crates.io/crates/clap) | `2.33` | CLI argument parsing |
| [`reqwest`](https://crates.io/crates/reqwest) | `0.11` (blocking) | HTTP client for making GET requests |
| [`indicatif`](https://crates.io/crates/indicatif) | `0.17` | Progress bars and spinners in the terminal |
| [`console`](https://crates.io/crates/console) | `0.15` | Colorized and styled terminal text output |
| [`anyhow`](https://crates.io/crates/anyhow) | `1.0` | Flexible, ergonomic error handling |
| [`dirs`](https://crates.io/crates/dirs) | `6.0` | Cross-platform identification of system folders like `Downloads` |
| [`dialoguer`](https://crates.io/crates/dialoguer) | `0.11` | Interactive terminal prompts (input, confirm, select) |
| [`directories`](https://crates.io/crates/directories) | `5.0` | Cross-platform app data directories for self-managed yt-dlp binary |

---

## 🔍 How It Works — Code Walkthrough

### 1. CLI Parsing (`clap`)

`main()` uses `clap`'s builder API to define the app's interface:
- An **optional** positional argument `URL` (omit to launch interactive mode)
- An optional `-O` / `--output` flag for the destination filename
- `-m` / `--media` to force yt-dlp usage
- `-a` / `--audio` for audio-only downloads

```rust
let matches = App::new("Ket")
    .arg(Arg::with_name("URL").required(false).index(1))
    .arg(Arg::with_name("OUTPUT").short("O").long("output").takes_value(true))
    .get_matches();
```

### 2. Interactive Mode (`dialoguer`)

When no URL is provided, `ket` launches an interactive terminal UI using `dialoguer`:

```rust
let url: String = Input::new()
    .with_prompt("📎 Paste URL")
    .interact_text()?;
```

This uses `dialoguer::Input` for text entry, `dialoguer::Confirm` for yes/no prompts, and `dialoguer::Select` for choosing between video/audio formats.

### 3. Self-Managed yt-dlp Binary

`ensure_ytdlp_binary()` resolves a cross-platform data directory using the `directories` crate and downloads the standalone `yt-dlp` binary on first use:

```rust
pub fn ensure_ytdlp_binary() -> Result<PathBuf> {
    let proj = ProjectDirs::from("com", "ket", "ket")
        .context("Could not determine app data directory")?;
    let binary_path = proj.data_dir().join(YTDLP_BINARY_NAME);

    if binary_path.exists() { /* verify & return */ }

    // First-run: download from GitHub releases
    let mut resp = client.get(YTDLP_DOWNLOAD_URL).send()?;
    let mut file = fs::File::create(&binary_path)?;
    std::io::copy(&mut resp, &mut file)?;

    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&binary_path, Permissions::from_mode(0o755))?;
    }
    Ok(binary_path)
}
```

All `Command::new()` calls for yt-dlp now use the `PathBuf` returned by this function instead of relying on the system PATH.

### 4. HTTP Request (`reqwest`)

`reqwest::blocking::Client` is used to send a synchronous GET request. The blocking feature is explicitly enabled in `Cargo.toml` since this is a simple CLI tool (no async runtime needed).

```rust
let client = Client::new();
let mut resp = client.get(target).send()
    .context(format!("Failed to connect to the URL: {}", target))?;
```

Response headers (`Content-Length`, `Content-Type`) are extracted to display file metadata before downloading.

### 5. Chunked Download Loop

The response body is read in chunks rather than all at once, which is memory-efficient for large files. The chunk size is calculated relative to `Content-Length`, with a minimum of 1024 bytes.

```rust
loop {
    let bcount = resp.read(&mut buffer[..])?;
    buffer.truncate(bcount);
    if !buffer.is_empty() {
        buf.extend_from_slice(&buffer);
        bar.inc(bcount as u64);
    } else {
        break; // EOF
    }
}
```

### 6. Progress Bar (`indicatif`)

`create_progress_bar()` decides which style to render:
- **Known length** → a determinate bar with `{bytes}/{total_bytes}` and `eta`
- **Unknown length** → an indeterminate spinner

```rust
match length {
    Some(len) => ProgressBar::new(len),
    None      => ProgressBar::new_spinner(),
}
```

### 7. Colorized Output (`console`)

The `style()` wrapper from the `console` crate applies ANSI color codes to strings, e.g. green for success, red for sizes.

```rust
print(format!("HTTP request sent... {}", style(resp.status()).green()), quiet_mode);
```

### 8. Error Handling (`anyhow`)

`anyhow::Result<()>` is used throughout. The `.context()` method wraps low-level errors with human-readable descriptions, and `anyhow::bail!` handles non-2xx HTTP responses:

```rust
anyhow::bail!("Server returned an error: {}", resp.status());
```

### 9. File Saving

`save_to_file()` creates the output file and writes the fully buffered byte vector to disk:

```rust
fn save_to_file(buf: &[u8], fname: &str) -> Result<()> {
    let mut file = File::create(fname)?;
    file.write_all(buf)?;
    Ok(())
}
```

---

## 🧪 Changelog
### v1.0.1

| Area | Before | After |
|---|---|---|
| yt-dlp management | Auto-install to system PATH | ✅ Self-managed binary in app data directory |
| Code architecture | Single `main.rs` file | ✅ Modular: `http.rs`, `media.rs`, `install.rs`, `tui.rs`, `utils.rs` |
| Package install | Not available | ✅ `winget` integration for installing software by name |
| Dependencies | 7 crates | 8 crates (added `directories`) |

---

## 🎯 Learning Goals

This project covers the following Rust concepts in a practical setting:

- **Ownership & Borrowing**: Passing `&str` vs `String`, working with `&[u8]`
- **Error Propagation**: Using `?` operator, `Result<T, E>`, `anyhow`
- **Pattern Matching**: `match` on `Option<u64>` for content length
- **Traits**: `Read` and `Write` from `std::io`
- **Iterators**: `split('/')`, `.last()`, `.unwrap_or()`
- **Closures**: Used internally in `indicatif` style templates
- **Process Management**: Spawning child processes (`yt-dlp`), checking exit codes
- **Interactive I/O**: Using `dialoguer` for terminal prompts and user input
- **Crate ecosystem**: Integrating multiple community crates together

---

## 👤 Author

**Pav Khemerak** — [pavkhemerak.official@gmail.com](mailto:pavkhemerak.official@gmail.com)

---

## 📄 License
This project is an unlicensed "homelab" type project built for personal use. No formal license is provided.
