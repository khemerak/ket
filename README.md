# ket 🦀

> A `wget` clone written in Rust — built as a hands-on learning project to explore Rust's ecosystem for CLI tools, HTTP networking, and terminal UX.

---

## 📖 About

`ket` is a minimal command-line file downloader inspired by GNU `wget`. It takes a URL, makes an HTTP GET request, and saves the response body to disk — all while showing a beautiful, real-time progress bar in the terminal.

The project was built as a **learning exercise** to explore the following Rust concepts and crates:

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

---

## ✨ Features

- 📥 Download any file from a given HTTP/HTTPS URL
- 🎬 Download videos (`.mp4`) and audio (`.mp3`) from media sites (YouTube, Vimeo, etc.) using `yt-dlp` integration (`-m` / `-a` flags)
- 📊 Real-time progress bar with elapsed time, speed, and ETA (when `Content-Length` is available)
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

### Help

```bash
ket --help
```

```
USAGE:
    ket [OPTIONS] <URL>

ARGS:
    <URL>    url to download

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
│   ├── main.rs         # All application logic (single-file project)
│   └── main.rs.bak     # Earlier version of main.rs (before refactoring)
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

---

## 🔍 How It Works — Code Walkthrough

### 1. CLI Parsing (`clap`)

`main()` uses `clap`'s builder API to define the app's interface:
- A required positional argument `URL`
- An optional `-O` / `--output` flag for the destination filename

```rust
let matches = App::new("Ket")
    .arg(Arg::with_name("URL").required(true).index(1))
    .arg(Arg::with_name("OUTPUT").short("O").long("output").takes_value(true))
    .get_matches();
```

### 2. HTTP Request (`reqwest`)

`reqwest::blocking::Client` is used to send a synchronous GET request. The blocking feature is explicitly enabled in `Cargo.toml` since this is a simple CLI tool (no async runtime needed).

```rust
let client = Client::new();
let mut resp = client.get(target).send()
    .context(format!("Failed to connect to the URL: {}", target))?;
```

Response headers (`Content-Length`, `Content-Type`) are extracted to display file metadata before downloading.

### 3. Chunked Download Loop

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

### 4. Progress Bar (`indicatif`)

`create_progress_bar()` decides which style to render:
- **Known length** → a determinate bar with `{bytes}/{total_bytes}` and `eta`
- **Unknown length** → an indeterminate spinner

```rust
match length {
    Some(len) => ProgressBar::new(len),
    None      => ProgressBar::new_spinner(),
}
```

### 5. Colorized Output (`console`)

The `style()` wrapper from the `console` crate applies ANSI color codes to strings, e.g. green for success, red for sizes.

```rust
print(format!("HTTP request sent... {}", style(resp.status()).green()), quiet_mode);
```

### 6. Error Handling (`anyhow`)

`anyhow::Result<()>` is used throughout. The `.context()` method wraps low-level errors with human-readable descriptions, and `anyhow::bail!` handles non-2xx HTTP responses:

```rust
anyhow::bail!("Server returned an error: {}", resp.status());
```

### 7. File Saving

`save_to_file()` creates the output file and writes the fully buffered byte vector to disk:

```rust
fn save_to_file(buf: &[u8], fname: &str) -> Result<()> {
    let mut file = File::create(fname)?;
    file.write_all(buf)?;
    Ok(())
}
```

---

## 🧪 What Changed Between `main.rs.bak` and `main.rs`

The `.bak` file is the original draft before improvements were applied. Comparing the two is a great learning exercise:

| Area | `main.rs.bak` (old) | `main.rs` (current) |
|---|---|---|
| Error handling | `Box<dyn Error>` + `.unwrap()` | `anyhow::Result` + `.context()` |
| Function scope | Nested functions inside `main()` | Top-level functions |
| `-O` output flag | Not implemented | ✅ Implemented |
| HTTP error handling | No check for non-2xx | `anyhow::bail!` for failed status |
| Chunk size floor | Could be 0 | `std::cmp::max(chunk_size, 1024)` |
| API compatibility | Used outdated `reqwest` types | Updated to modern `0.11` API |

---

## 🎯 Learning Goals

This project covers the following Rust concepts in a practical setting:

- **Ownership & Borrowing**: Passing `&str` vs `String`, working with `&[u8]`
- **Error Propagation**: Using `?` operator, `Result<T, E>`, `anyhow`
- **Pattern Matching**: `match` on `Option<u64>` for content length
- **Traits**: `Read` and `Write` from `std::io`
- **Iterators**: `split('/')`, `.last()`, `.unwrap_or()`
- **Closures**: Used internally in `indicatif` style templates
- **Crate ecosystem**: Integrating multiple community crates together

---

## 👤 Author

**Pav Khemerak** — [pavkhemerak.official@gmail.com](mailto:pavkhemerak.official@gmail.com)

---

## 📄 License

This project is intended for educational purposes. No formal license is specified.
