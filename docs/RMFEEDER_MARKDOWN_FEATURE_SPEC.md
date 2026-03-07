# rmfeeder: Markdown & Stdin Ingestion — Feature Specification

**Status:** Specification complete. Not yet implemented.
**Target:** rmfeeder-rs (`/Users/smck/.cargo/bin/rmfeeder`)
**Author:** PAI (Sean implements)
**Date:** 2026-03-05

---

## Motivation

rmfeeder currently requires a URL or feed source. PAI generates markdown constantly — research
summaries, TELOS notes, GOALS, CHALLENGES, etc. — and has no way to convert those into
reMarkable-optimized PDFs. Additionally, Fabric outputs markdown to stdout with no way to pipe
it directly into rmfeeder.

This feature closes that gap by adding three complementary input modes: single file, directory,
and stdin. Together they enable the full PAI → PDF pipeline.

---

## Recommended Implementation Approach

Add three new mutually exclusive source flags alongside the existing URL, `--file`, `--feeds`,
and `--yt-watchlist` sources. Markdown input is rendered to the same HTML+CSS pipeline that
URL-sourced content uses, with an additional pre-processing step to strip YAML frontmatter and
apply a markdown-to-HTML conversion.

---

## New Flags

### `--markdown <path>`

Reads a single local markdown file and converts it to a single-entry PDF bundle.

**Behavior:**
- Reads the file at `<path>`
- Strips YAML frontmatter (any `---`-delimited block at the top of the file)
- Converts markdown to HTML using the existing rendering pipeline
- Bundle title: first `# H1` heading in the file; if no H1, use the filename without extension
- Cover page: displays bundle title, source path, and timestamp
- No TOC (single entry — TOC would be redundant)
- Output: timestamped default filename or `--output` value
- Error if `<path>` does not exist: print `error: file not found: <path>` to stderr, exit 1

### `--markdown-dir <path>`

Reads all `.md` files in the directory (flat, non-recursive) and produces a multi-entry PDF
bundle with a navigable table of contents.

**Behavior:**
- Scans `<path>` for files matching `*.md` (case-insensitive extension match)
- File ordering: alphabetical by filename, case-insensitive (e.g., `a.md` < `B.md` < `c.md`)
- Excludes subdirectories (flat scan only — no recursion)
- Each file processed as a standalone entry:
  - Strip YAML frontmatter
  - Convert markdown to HTML
  - Entry title: first `# H1` in the file; fallback to filename without extension
- Bundle structure: cover page → TOC → entry-1 → entry-2 → ... (same as URL bundles)
- TOC entry label: entry title (H1 or filename, as above)
- Each entry includes "Back to TOC" link at the end (same implementation as URL bundles)
- Output: timestamped default filename or `--output` value
- Error if `<path>` does not exist or is not a directory: stderr + exit 1
- Error if directory contains zero `.md` files: `error: no markdown files found in <path>` + exit 1

### `--stdin`

Reads markdown from stdin until EOF and produces a single-entry PDF bundle. Required explicit
flag — do not auto-detect stdin to avoid TTY ambiguity in Rust.

**Behavior:**
- Reads all bytes from stdin until EOF (`BufReader::read_to_string` pattern)
- Strip YAML frontmatter if present
- Convert markdown to HTML
- Bundle title: first `# H1` in the content; fallback to `"stdin-bundle"`
- Cover page: displays bundle title, source `"<stdin>"`, and timestamp
- No TOC (single entry)
- Output: timestamped default filename or `--output` value
- Error on empty stdin (zero bytes or whitespace-only): `error: stdin produced no content` + exit 1

---

## Markdown Rendering

Use an existing Rust markdown crate. Recommended: `pulldown-cmark` (CommonMark compliant, widely
used in the Rust ecosystem, no system dependencies).

Processing pipeline per file/entry:
1. Read raw content as UTF-8 string
2. Strip YAML frontmatter: if content starts with `---\n`, strip everything up to and including
   the closing `---\n` (first occurrence only)
3. Parse markdown to HTML via pulldown-cmark with these extensions enabled:
   - Tables
   - Strikethrough
   - Task lists
   - Footnotes
4. Wrap in the same HTML template used for URL-sourced articles (apply existing CSS)
5. Pass through WeasyPrint for PDF rendering (existing pipeline, no changes needed)

---

## Cover Page

For all three modes, the cover page follows the existing URL bundle format with these fields:

| Field | Value |
|-------|-------|
| Title | Bundle title (H1 or filename or "stdin-bundle") |
| Source | File path (`--markdown`, `--markdown-dir`) or `<stdin>` |
| Entry count | Number of entries (1 for `--markdown` / `--stdin`) |
| Timestamp | Generation timestamp (existing format) |

---

## YAML Frontmatter Stripping

Definition: YAML frontmatter is present if and only if the content begins with exactly `---`
followed immediately by a newline (`\n`). Strip everything from byte 0 through and including
the first subsequent `---\n` sequence. If no closing `---\n` is found, treat the entire content
as non-frontmatter (do not strip anything).

This handles:
- PAI markdown files (all have frontmatter)
- Files with no frontmatter (no-op)
- Files where `---` appears mid-document (not stripped — only leading frontmatter is targeted)

---

## Flag Interactions

### `--output`

Works identically for all three new modes. Default timestamped filename if not specified.

### `--page-size`

Works identically for all three new modes. Applies to the entire bundle.

### `--limit`

For `--markdown-dir`: limits the number of files included. Files are selected in alphabetical
order up to `--limit N`. Ignored for `--markdown` and `--stdin` (single-entry).

### `--summarize` / `--pattern`

For `--markdown` and `--markdown-dir`: after rendering each entry's markdown to HTML, apply the
specified Fabric pattern to the text content before rendering to PDF. This allows:
```bash
rmfeeder --markdown long-notes.md --summarize --pattern extract_wisdom
```
For `--stdin`: apply the Fabric pattern to the stdin content before rendering.

Note: this requires running `fabric` on the extracted text content — same approach as URL
summarization.

### `--delay`

Not applicable to `--markdown`, `--markdown-dir`, or `--stdin`. Silently ignored (no local
network requests are made).

---

## Mutual Exclusivity

The following source modes are mutually exclusive. rmfeeder must emit an error and exit 1 if
more than one is specified together:

```
URL args | --file | --feeds | --feeds-file | --yt-watchlist | --markdown | --markdown-dir | --stdin
```

Error message format:
```
error: conflicting source flags: --markdown and --feeds cannot be used together
```

---

## PAI Use Cases (Reference)

### Bundle TELOS notes for reMarkable review

```bash
rmfeeder --markdown-dir ~/.claude/PAI/USER/TELOS/ \
  --page-size rm2 \
  --output telos-$(date +%Y%m%d).pdf
```

### Fabric wisdom extraction → PDF in one pipe

```bash
fabric -u "https://article.com" -p extract_wisdom | \
  rmfeeder --stdin --page-size rm2 --output wisdom-$(date +%Y%m%d).pdf
```

### Research output → PDF

```bash
# After Research skill outputs a markdown summary to file:
rmfeeder --markdown /tmp/research-output.md \
  --page-size rm2 \
  --output research-$(date +%Y%m%d).pdf
```

### Summarize a directory of notes before bundling

```bash
rmfeeder --markdown-dir ~/notes/ \
  --summarize --pattern extract_wisdom \
  --page-size rm2
```

---

## Suggested Cargo Dependencies

```toml
[dependencies]
pulldown-cmark = "0.12"   # Markdown → HTML (CommonMark + extensions)
```

No additional system dependencies required. WeasyPrint and the existing HTML template pipeline
handle the rest.

---

## Acceptance Criteria

- [ ] `rmfeeder --markdown file.md` produces a PDF with content from the file
- [ ] `rmfeeder --markdown file.md` strips YAML frontmatter before rendering
- [ ] `rmfeeder --markdown file.md` uses H1 as title, filename fallback
- [ ] `rmfeeder --markdown-dir ./dir/` bundles all .md files alphabetically
- [ ] `rmfeeder --markdown-dir ./dir/` generates TOC with per-file entries
- [ ] `rmfeeder --markdown-dir ./dir/` includes Back-to-TOC links per entry
- [ ] `rmfeeder --markdown-dir ./dir/ --limit 5` caps at 5 files
- [ ] `echo "# Hello" | rmfeeder --stdin` produces a PDF from stdin
- [ ] `rmfeeder --stdin` with empty stdin exits 1 with error message
- [ ] `rmfeeder --markdown missing.md` exits 1 with "file not found" error
- [ ] `rmfeeder --markdown-dir /empty/` exits 1 with "no markdown files found" error
- [ ] `rmfeeder --markdown file.md --feeds` exits 1 with "conflicting source flags" error
- [ ] `--output`, `--page-size`, `--limit` work as specified with all three modes
- [ ] `fabric -u URL -p extract_wisdom | rmfeeder --stdin` produces a valid PDF
