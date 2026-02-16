# YouTube Watch Later Helper v1 Spec

## What "v1 spec" means
This document defines the first shippable version of the YouTube watchlist workflow with no ambiguity:
- exact CLI and flags
- default values
- output naming
- success and failure behavior
- what is intentionally out of scope

If implementation matches this spec, behavior should be predictable and testable.

## Goal
Provide a single command that:
1. pulls videos from YouTube Watch Later,
2. summarizes each video with `fabric-ai`,
3. builds one PDF reading bundle with the existing rmfeeder cover/TOC/article-section UX.

## Binary
Add a new helper binary:
- `yt_helper`

This keeps risk low and avoids destabilizing existing `rmfeeder`/`opml_helper` flows.

## CLI (v1)
```
yt_helper --watch-later [options]
```

### Required mode flag
- `--watch-later`
  - uses playlist URL `https://www.youtube.com/playlist?list=WL`

### Options
- `--output <file.pdf>`
  - optional explicit output path
- `--limit <N>`
  - default: `10`
  - max videos to include in output bundle (after local-state skips/failures)
- `--pattern <name>`
  - default: `youtube_summary`
  - passed to `fabric-ai -p <name>`
- `--delay <seconds>`
  - default: `0`
  - sleep between video processing attempts
- `--cookies-from-browser <name>`
  - default: `chrome`
  - passed to `yt-dlp --cookies-from-browser <name>`
- `--dry-run`
  - optional
  - still generates PDF output
  - disables side effects:
    - no local state reads/writes
    - no YouTube mark-watched updates
- `--clear-state`
  - optional; clear local YouTube helper seen-state before run
- `--config <path>`
  - optional; same config behavior as other binaries

## Default output filename
If `--output` is not provided:
- `yt-watchlist-YYYY-MM-DD-HH-MM-SS.pdf`

Example:
- `yt-watchlist-2026-02-16-13-42-09.pdf`

## Config integration
Use existing `rmfeeder.toml` loading and CLI override precedence.

### New config keys for v1
- `yt_limit = 10`
- `yt_pattern = "youtube_summary"`
- `yt_delay = 0`
- `yt_cookies_browser = "chrome"`
- `yt_mark_watched_on_success = true`

### Existing config keys reused
- `output_dir`
- `state_db_path` (or separate yt state path in v2)

## Data flow
1. `yt-dlp` fetches Watch Later metadata (title + URL) using:
   - `--flat-playlist`
   - `--print "%(title)s|%(webpage_url)s"`
2. Helper normalizes and sanitizes title for display and IDs.
3. For each URL:
   - check state unless `--dry-run`
   - run `fabric-ai -y <url> -p <pattern>`
   - parse markdown output to HTML via existing markdown pipeline (`pulldown-cmark`)
   - build item model: title, source URL, rendered HTML body
4. Hand collected items to existing bundle renderer path (cover + TOC + section blocks).
5. Write PDF to selected output path.
6. Unless `--dry-run` is set, mark successful items watched with `yt-dlp --mark-watched --skip-download`.
7. Unless `--dry-run` is set, update state for successfully included URLs.

## State behavior (v1)
State is URL-based dedupe.

Default path:
- `~/.local/share/rmfeeder/rmfeeder_state.sqlite`

Table can reuse existing `seen(url, seen_at)` in v1, with helper prefixing logical source in code if needed for disambiguation later.

### Flags
- default behavior:
  - local state ON
  - mark watched ON (after successful inclusion in bundle)
- `--dry-run`:
  - do not read or write local state
  - do not mark watched on YouTube
  - still generate PDF
- `--clear-state`:
  - clear local state table before run
  - does not alter YouTube watched history

## Failure behavior
- Per-item failures do not abort entire run.
- Failed item logs warning and continues.
- If all items fail or are filtered, command exits non-zero with a clear message.
- By default, mark-watched applies only to items that reached bundle inclusion.
- In `--dry-run`, no mark-watched updates occur.

## Logging behavior
Print progress to stderr:
- fetching watch later list
- processing each URL
- skipped/failed reasons
- final summary: attempted / included / skipped / failed

CLI help text for `--dry-run` should explicitly state:
- "Generate PDF without side effects (no local state read/write, no mark-watched updates)."

## UX constraints
- PDF output must match existing rmfeeder visual style and TOC navigation.
- TOC entries should remain link-clickable with current dotted underline style.
- Cover page title for this mode should be:
  - `rmfeeder ::`
  - `YouTube Watchlist`

## Out of scope for v1
- custom playlist IDs beyond Watch Later
- manual YouTube URL list input for `yt_helper` (for example `urls.txt`)
- auto-detecting YouTube URLs inside generic URL flows (`rmfeeder --file`, direct URL args, RSS/OPML) and routing them through YouTube summarization automatically
- transcript language options
- retries/backoff tuning
- parallel processing
- separate yt-specific SQLite schema
- in-app OAuth/cookie management

## Acceptance criteria
1. Running with `--watch-later` generates a PDF with at least one entry when items are available.
2. Default filename follows `yt-watchlist-YYYY-MM-DD-HH-MM-SS.pdf`.
3. `--output` overrides filename.
4. `--pattern` affects summary generation.
5. By default, successful items are marked watched after bundle inclusion.
6. `--dry-run` still writes PDF and causes no local/YouTube state changes.
7. `--clear-state` resets local state only.
8. Existing `rmfeeder` and `opml_helper` behavior remains unchanged.
