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
- `opml_helper` extracts recent article URLs from feeds in an OPML file
- Local SQLite state avoids re-processing already-seen entries
- Supports stateless runs (`--no-state`) and state reset (`--clear-state`)

### ✔ YouTube Watch Later Workflow
- `yt_helper` reads Watch Later via `yt-dlp`
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
- `yt_helper` (YouTube Watch Later summaries)

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

- `yt_helper --watch-later`

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

This crate provides three binaries:

- `rmfeeder` (fetch URLs and generate PDF)
- `opml_helper` (read feeds OPML and emit article URLs)
- `yt_helper` (read YouTube Watch Later, summarize, and build PDF bundle)

### **Configuration**

If a `rmfeeder.toml` file is present, its values become defaults (CLI flags override).
By default, the app looks for `rmfeeder.toml` in the current working directory unless `--config` is passed.

```toml
state_db_path = "~/.local/share/rmfeeder/rmfeeder_state.sqlite"
feeds_opml_path = "feeds.opml"
urls_path = "urls.txt"
output_dir = "output"
page_size = "letter"
limit = 3
delay = 2
summarize = true
pattern = "summarize"
```

Use a different config path:

```bash
cargo run --bin rmfeeder -- --config ~/.config/rmfeeder/custom.toml --file urls.txt
cargo run --bin opml_helper -- --config ~/.config/rmfeeder/custom.toml --limit 5 feeds.opml
```

Page size can be selected per-run:

```bash
cargo run --bin rmfeeder -- --page-size rm2 --file urls.txt
cargo run --bin yt_helper -- --watch-later --page-size rm2
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
- `yt_helper` watch later: `yt-watchlist-YYYY-MM-DD-HH-MM-SS.pdf`

`--output` always overrides default naming.

### **OPML Helper**

Current version note: OPML is a two-step workflow.
There is no `rmfeeder --opml` flag yet.

Generate a URL list from an OPML file (default 3 per feed), then feed it into rmfeeder:

```bash
cargo run --bin opml_helper -- --limit 3 --output urls.txt feeds.opml
cargo run --bin rmfeeder -- --file urls.txt
```

Write URLs to stdout (no `--output`):

```bash
cargo run --bin opml_helper -- --limit 5 feeds.opml
```

State behavior (default is stateful):

```bash
cargo run --bin opml_helper -- --no-state feeds.opml
cargo run --bin opml_helper -- --clear-state feeds.opml
```

Default state DB path:

```text
~/.local/share/rmfeeder/rmfeeder_state.sqlite
```

### **YouTube Helper**

`yt_helper` requires `fabric-ai` to generate summaries.

Build a Watch Later summary bundle PDF:

```bash
cargo run --bin yt_helper -- --watch-later
```

Default output filename:

```text
yt-watchlist-YYYY-MM-DD-HH-MM-SS.pdf
```

Common options:

```bash
cargo run --bin yt_helper -- --watch-later --output yt-bundle.pdf --limit 8 --pattern youtube_summary --delay 2
```

`--limit` is the number of videos included in the PDF (after local-state skips/failures).

Cookie profile behavior:

- By default, `yt_helper` uses `--cookies-from-browser chrome`, which means Chrome's default profile.
- If your YouTube account is in a different Chrome profile, pass it explicitly:

```bash
cargo run --bin yt_helper -- --watch-later --cookies-from-browser "chrome:Profile 4"
```

- You can also set this in `rmfeeder.toml`:

```toml
yt_cookies_browser = "chrome:Profile 4"
```

Dry-run mode still generates the PDF, but does not update local state or YouTube watched status:

```bash
cargo run --bin yt_helper -- --watch-later --dry-run
```

Expected YouTube workflow:

- `yt_helper` does not filter by YouTube watched state when reading Watch Later.
- `yt_helper` uses local SQLite state (`~/.local/share/rmfeeder/rmfeeder_state.sqlite`) as the source of truth for "already processed".
- Marking watched is a YouTube side effect only; videos may still remain in Watch Later until you remove them in YouTube's web/app UI.
- Typical flow:
  - run `yt_helper` to generate reading bundles
  - keep items in Watch Later while deciding whether to watch full videos
  - manually remove watched items in YouTube when done

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
