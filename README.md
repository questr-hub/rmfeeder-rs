# rmfeeder-rs

`rmfeeder` is a Rust-based tool that fetches readable web articles, extracts clean content,
and formats them into beautiful PDFs — perfect for the reMarkable tablet or any PDF reader.

This project now supports **single-article mode**, **multi-article bundles**, **cover pages**,  
**table of contents (TOC)**, **navigation links**, and full **CSS-driven typography**.

---

## ✨ Features

### ✔ Single Article PDF
- Fetch a URL
- Extract readable content using `dom_smoothie`
- Apply clean typography defined in `styles.css`
- Convert to PDF with WeasyPrint

### ✔ Multi-Article PDF Bundles
When multiple URLs are passed:

1. **Cover Page**
   - Automatically generated
   - Includes date
   - Styled with large title fonts

2. **Table of Contents**
   - Hyperlinks to each article section
   - Built dynamically from article titles
   - Clean layout through CSS

3. **Per-Article Sections**
   - Each section gets:
     - A header with the article title
     - Extracted readable HTML content
     - A “📄 Back to TOC” navigation link
   - Proper page breaks, margins, spacing

4. **Consistent Styling**
   - All typography and layout comes from `styles.css`
   - Easy to modify to adjust margins, fonts, or reMarkable optimization

### ✔ WeasyPrint Rendering
- HTML + CSS → high-quality PDF
- Supports page size controls, margins, and custom fonts

---

## 📦 Installation

### Requirements

- Rust (`rustup`)
- WeasyPrint (`brew install weasyprint`)
- Python 3 and GTK libraries (automatically installed by brew)
- Optional: fabric (`brew install fabric-ai`) for `--summarize`
- macOS, Linux, or WSL

### Build

From inside the crate:

```bash
cd rmfeeder-rs/rmfeeder
cargo build --release
```

---

## 🚀 Usage

This crate provides two binaries:

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

### **OPML Helper**

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
2025-01-11-08-45-30.pdf
```

To set a custom filename:

```bash
cargo run --bin rmfeeder -- --output article.pdf "https://en.wikipedia.org/wiki/Rust_(programming_language)"
```

To generate a summary instead of the full article:

```bash
cargo run --bin rmfeeder -- --summarize "https://en.wikipedia.org/wiki/Rust_(programming_language)"
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
