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

### **Configuration**

If a `rmfeeder.toml` file is present, its values become defaults (CLI flags override):

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

### **Single Article**

```bash
cargo run -- "https://en.wikipedia.org/wiki/Rust_(programming_language)"
```

Produces a timestamped PDF filename like:

```
2025-01-11-08-45-30.pdf
```

To set a custom filename:

```bash
cargo run -- --output article.pdf "https://en.wikipedia.org/wiki/Rust_(programming_language)"
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
cargo run -- "https://example.com/article1" "https://example.com/article2"
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
