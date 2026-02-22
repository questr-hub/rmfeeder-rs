# rmfeeder-rs

`rmfeeder` is a Rust-based reading-bundle tool for reMarkable and other PDF readers.
It supports three core workflows with a shared visual output:
- direct URLs
- RSS/OPML feed workflows
- YouTube Watch Later summaries

All workflows render through the same PDF engine and styling system (cover page, TOC, navigation links, and typography).

---

## ✨ Features

### ✔ URL Reader to PDF
- Single URL mode
- Multi-URL bundle mode
- Optional summary mode with `fabric-ai` (`--summarize`, `--pattern`)

### ✔ Feed Workflow (OPML + State)
- `rmfeeder --feeds` extracts recent article URLs from feeds in an OPML file
- Local SQLite state avoids re-processing already-seen entries
- Supports state reset (`--clear-state`)

### ✔ YouTube Watch Later Workflow
- `rmfeeder --yt-watchlist` reads Watch Later via `yt-dlp`
- Summarizes videos with `fabric-ai` patterns (default `youtube_summary`)
- Builds a single reading bundle PDF from summaries
- Local SQLite state dedupes already-processed videos

### ✔ Shared Reading-Bundle UX
- Auto cover page with date
- Hyperlinked table of contents
- Per-item sections with “Back to TOC” links
- Consistent typography and layout via `styles.css`
- Selectable page size (`letter` default, `rm2` option)

### ✔ WeasyPrint Rendering Pipeline
- HTML + CSS to high-quality PDF
- Common renderer across URL, OPML, and YouTube flows

---

## 📦 Installation

### Requirements

- Rust (`rustup`)
- WeasyPrint (`brew install weasyprint`)
- Python 3 and GTK libraries (automatically installed by brew)
- Optional for build, required for summary workflows: `fabric-ai`
- Optional for build, required for YouTube workflow: `yt-dlp`
- macOS, Linux, or WSL

### Summary Workflow Dependencies

`fabric-ai` is not required to compile `rmfeeder`, but it is required for:

- `rmfeeder --summarize`
- `rmfeeder --yt-watchlist` (YouTube Watch Later summaries)

Install with Homebrew:

```bash
brew install fabric-ai
```

Verify installation:

```bash
fabric-ai --version
```

If `fabric-ai` is missing, summary commands will fail at runtime with a process/command-not-found style error.

### YouTube Workflow Dependencies

`yt-dlp` is not required to compile `rmfeeder`, but it is required for:

- `rmfeeder --yt-watchlist`

Install with Homebrew:

```bash
brew install yt-dlp
```

Verify installation:

```bash
yt-dlp --version
```

If `yt-dlp` is missing, `yt_helper` cannot fetch Watch Later entries.

### Build

From inside the crate:

```bash
cd rmfeeder-rs/rmfeeder
cargo build --release
```

---

## 🚀 Usage

Primary entrypoint:

- `rmfeeder` (unified URL/file/feeds/YouTube bundle generation)

Compatibility helpers are still available:

- `opml_helper`
- `yt_helper`

### **Configuration**

If a `rmfeeder.toml` file is present, its values become defaults (CLI flags override).
By default, the app looks for config in:
- `$XDG_CONFIG_HOME/rmfeeder/rmfeeder.toml` (when `XDG_CONFIG_HOME` is set)
- otherwise `~/.config/rmfeeder/rmfeeder.toml`

If neither environment path is available, it falls back to `rmfeeder.toml` in the current working directory.
Use `--config` to override the path explicitly.

```toml
state_db_path = "~/.local/share/rmfeeder/rmfeeder_state.sqlite"
feeds_opml_path = "~/.config/rmfeeder/feeds.opml"
urls_path = "urls.txt"
output_dir = "output"
page_size = "letter"
limit = 3
delay = 2
summarize = true
pattern = "summarize"
yt_limit = 10
yt_pattern = "youtube_summary"
yt_delay = 0
yt_cookies_browser = "chrome"
yt_mark_watched_on_success = true
```

Use a different config path:

```bash
cargo run --bin rmfeeder -- --config ~/.config/rmfeeder/custom.toml --file urls.txt
cargo run --bin rmfeeder -- --config ~/.config/rmfeeder/custom.toml --feeds --limit 5
```

Page size can be selected per-run:

```bash
cargo run --bin rmfeeder -- --page-size rm2 --file urls.txt
cargo run --bin rmfeeder -- --yt-watchlist --page-size rm2
```

Supported values:
- `letter` (default)
- `rm2`

### **Default Output Filenames**

If `--output` is not provided, filenames are mode-based and timestamped:

- `rmfeeder` single URL: `single-YYYY-MM-DD-HH-MM-SS.pdf`
- `rmfeeder` single URL + `--summarize`: `single-summary-YYYY-MM-DD-HH-MM-SS.pdf`
- `rmfeeder` bundle (multiple URLs or any `--file`): `bundle-YYYY-MM-DD-HH-MM-SS.pdf`
- `rmfeeder` bundle + `--summarize`: `bundle-summary-YYYY-MM-DD-HH-MM-SS.pdf`
- `rmfeeder` with `--yt-watchlist`: `bundle-summary-YYYY-MM-DD-HH-MM-SS.pdf`
- `yt_helper` watch later (legacy helper): `yt-watchlist-YYYY-MM-DD-HH-MM-SS.pdf`

`--output` always overrides default naming.

### **Unified Source Selection**

Use one command to combine sources:

```bash
# Direct URLs
cargo run --bin rmfeeder -- "https://example.com/a" "https://example.com/b"

# URLs from file
cargo run --bin rmfeeder -- --file urls.txt

# OPML feeds (defaults to ~/.config/rmfeeder/feeds.opml)
cargo run --bin rmfeeder -- --feeds

# YouTube Watch Later summaries
cargo run --bin rmfeeder -- --yt-watchlist

# Combined run (feeds + YouTube + direct URL)
cargo run --bin rmfeeder -- --feeds --yt-watchlist "https://example.com/c"
```

Source selectors are additive:

- `--feeds`
- `--feeds-file <feeds.opml>`
- `--yt-watchlist`
- `--file <path>`
- direct URL args (`<url1> [url2] ...`)

Source-specific options:

- `--limit N` sets both feeds and YouTube limits together
- `--yt-limit N` overrides only YouTube limit
- `--yt-pattern <name>` sets YouTube summary pattern (default `youtube_summary`)
- `--cookies-from-browser <name>` selects browser/profile for YouTube auth
- `--no-mark-watched` disables YouTube mark-watched side effects

State behavior:

- `--clear-state` resets `~/.local/share/rmfeeder/rmfeeder_state.sqlite`
- seen-item skips log as `already seen, skipping item: ...`
- feed items retain OPML section grouping in the TOC
- YouTube items are grouped under `YouTube Watchlist`

### **Single Article**

```bash
cargo run --bin rmfeeder -- "https://en.wikipedia.org/wiki/Rust_(programming_language)"
```

Produces a timestamped PDF filename like:

```
single-2025-01-11-08-45-30.pdf
```

To set a custom filename:

```bash
cargo run --bin rmfeeder -- --output article.pdf "https://en.wikipedia.org/wiki/Rust_(programming_language)"
```

To generate a summary instead of the full article:

```bash
cargo run --bin rmfeeder -- --summarize "https://en.wikipedia.org/wiki/Rust_(programming_language)"
```

Note: `--summarize` requires `fabric-ai` to be installed and available on your `PATH`.

Single-article summary defaults to:

```text
single-summary-YYYY-MM-DD-HH-MM-SS.pdf
```

Use a different fabric pattern:

```bash
cargo run --bin rmfeeder -- --summarize --pattern extract_wisdom "https://en.wikipedia.org/wiki/Rust_(programming_language)"
```

---

### **Multi-Article Bundle**

```bash
cargo run --bin rmfeeder -- "https://example.com/article1" "https://example.com/article2"
```

Optional delay between fetches (in seconds):

```bash
cargo run --bin rmfeeder -- --delay 2 "https://example.com/article1" "https://example.com/article2"
```

Default filename behavior for bundles:

- no summarize: `bundle-YYYY-MM-DD-HH-MM-SS.pdf`
- with summarize: `bundle-summary-YYYY-MM-DD-HH-MM-SS.pdf`
- `--file` input is always treated as bundle mode for naming.

`--output` always overrides default naming.

Produces a multi-page PDF with:

- Cover page  
- Table of contents  
- Article #1  
- Article #2  
- Navigation links  

---

## 🗂 Project Structure

```
rmfeeder/
  src/
    lib.rs
    main.rs
    fetcher.rs
    extractor.rs
    multipdf.rs
    pdf.rs
    epub.rs            (unused for now)
    xhtml.rs
    xhtml_sanitize.rs
  styles.css
  Cargo.toml
```

---

## 📄 License

MIT

---
