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
- macOS, Linux, or WSL

### Build

From inside the crate:

```bash
cd rmfeeder-rs/rmfeeder
cargo build --release
```

---

## 🚀 Usage

### **Single Article**

```bash
cargo run -- "https://en.wikipedia.org/wiki/Rust_(programming_language)"
```

Produces:

```
output.pdf
```

---

### **Multi-Article Bundle**

```bash
cargo run --   "https://example.com/article1"   "https://example.com/article2"
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

## 🛣 Roadmap

- CLI flags (output filename, title override, disable cover page, etc.)
- EPUB output (via Pandoc)
- Automatic send-to-reMarkable via MailHop alias
- RSS/OPML feed ingestion
- Daily digest mode
- Improved styling presets (reMarkable, Kindle, desktop)

---

## 📄 License

MIT

---

