# `rmfeeder --help` UX Spec

## Goal
Make `rmfeeder --help` user-friendly by replacing the current single-line usage string with structured, contextual help similar to `mailhop`.

## Problem
Current behavior prints one dense line:

```text
Usage: rmfeeder [--config <path>] [--output <file.pdf>] ...
```

This makes discovery hard:
- flags are not described
- source-mode rules are not obvious
- no examples are shown
- no guidance on defaults/config paths is shown

## Target Help Output

### Top-level shape
Help should be organized into explicit sections:
- one-line description
- usage variants
- source input options
- output/render options
- summarization options
- YouTube-specific options
- maintenance/debug options
- examples
- config/defaults notes

### Proposed `--help` content (target)
```text
rmfeeder - Build article/notes bundles as device-friendly PDFs

Usage:
  rmfeeder [OPTIONS] <url1> [url2 ...]
  rmfeeder [OPTIONS] --file <path>
  rmfeeder [OPTIONS] --feeds [--feeds-file <feeds.opml>]
  rmfeeder [OPTIONS] --yt-watchlist
  rmfeeder [OPTIONS] --markdown <path>
  rmfeeder [OPTIONS] --markdown-dir <path>
  rmfeeder [OPTIONS] --stdin
  rmfeeder --clear-state
  rmfeeder --list-targets

Source Input (choose exactly one):
  <url...>                 One or more direct URLs
  --file <path>            Read URLs from a file (blank lines / # comments ignored)
  --feeds                  Use OPML feeds from default or configured path
  --feeds-file <path>      Use an explicit OPML file (implies --feeds)
  --yt-watchlist           Pull from YouTube Watch Later
  --markdown <path>        Convert one markdown file to a single PDF entry
  --markdown-dir <path>    Convert a directory of markdown files into one bundle
  --stdin                  Read markdown content from stdin

Output & Rendering:
  --output <file.pdf>      Output PDF path (overrides timestamp naming)
  --delay <seconds>        Delay between fetches
  --page-size <name>       Target device/page profile (run --list-targets)
  --limit <N>              Shared item limit (feeds, yt, markdown-dir)

Summarization:
  --summarize              Use fabric to summarize content before rendering
  --pattern <name>         fabric pattern to use (implies --summarize)

YouTube Options:
  --yt-limit <N>           Limit only YouTube items
  --yt-pattern <name>      YouTube summary pattern (default: youtube_summary)
  --cookies-from-browser <name>
                            Browser/profile for auth cookies (default: chrome)
  --no-mark-watched        Do not mark processed videos as watched

Maintenance:
  --config <path>          Config file path (default: ~/.config/rmfeeder/rmfeeder.toml)
  --clear-state            Clear seen-item state DB and exit
  --list-targets           Print page-size target table as CSV and exit
  -h, --help               Show this help

Examples:
  rmfeeder --feeds --limit 5
  rmfeeder --yt-watchlist --yt-limit 20 --page-size rmpp
  rmfeeder --markdown notes.md --summarize --pattern extract_wisdom
  cat notes.md | rmfeeder --stdin --output note.pdf
```

## Implementation Approach
Use `clap` for argument parsing/help generation instead of manual `env::args()` handling.

### Why `clap`
- built-in, readable `--help` formatting
- per-flag descriptions and defaults
- mutual-exclusion and dependency validation
- better error text for invalid/missing values

### CLI modeling plan
1. Create `Args` struct with `#[derive(Parser)]`.
2. Keep positional `urls: Vec<String>` and explicit source flags.
3. Enforce source exclusivity with an argument group:
   - sources: `urls`, `file`, `feeds`, `yt_watchlist`, `markdown`, `markdown_dir`, `stdin`.
4. Keep existing behavior:
   - `--pattern` implies summarize
   - `--feeds-file` implies feeds mode
   - `--limit` fans out to feeds + yt + markdown-dir defaults
   - `--clear-state` exits early when no source is selected
5. Keep `PageSize` validation and accepted values aligned with current parser.
6. Add `after_help` examples block and sectioned long help text.

## Compatibility Notes
- Functional behavior should remain unchanged.
- Error text may change to `clap`-style wording; this is acceptable if semantics stay the same.
- `--list-targets` remains supported.

## Test Plan
Add CLI-focused tests for help and validation.

### Required tests
- `rmfeeder --help` includes section headers:
  - `Source Input`
  - `Output & Rendering`
  - `Summarization`
  - `YouTube Options`
  - `Examples`
- `rmfeeder --help` contains a one-line description and multiple usage forms.
- conflicting source selectors fail with non-zero exit.
- missing values (for example `--file` without path) fail with clear message.
- invalid page size fails and prints allowed values.

### Suggested mechanics
- integration tests under `rmfeeder/tests/cli_help.rs`
- use `assert_cmd` + `predicates` for output assertions
- optional snapshot fixture for full help text (to catch regressions)

## Acceptance Criteria
- `rmfeeder --help` is multi-line and sectioned (not one dense line).
- every exposed flag has a short description.
- mutually exclusive source behavior is documented in help output.
- help includes at least four realistic command examples.
- existing workflows (`--feeds`, `--yt-watchlist`, `--markdown`, direct URLs) still run unchanged.
