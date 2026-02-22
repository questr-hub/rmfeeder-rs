# Unified `rmfeeder` Workflow Spec

## Goal
Make `rmfeeder` the single entrypoint for all collection and PDF-generation workflows while preserving current config compatibility.

## Scope
- Unify URL, file, OPML feeds, and YouTube Watch Later into `rmfeeder`.
- Keep seen-state dedupe and extend it consistently across all sources.
- Support combined-source runs (for example feeds + YouTube in one bundle).
- Keep current helper functionality available internally via library modules.

## CLI Surface (Target)
- `rmfeeder --yt-watchlist`
- `rmfeeder --feeds`
- `rmfeeder --feeds --yt-watchlist`
- `rmfeeder --file <path>`
- `rmfeeder <url1> [url2 ...]`

### Existing options retained
- `--config <path>`
- `--output <file.pdf>`
- `--delay <n>`
- `--page-size <letter|rm2>`
- `--summarize`
- `--pattern <name>`
- `--clear-state`

## Behavior Rules
1. Source selectors are additive.
2. Inputs from all selected sources are merged into one processing queue.
3. `--summarize` applies to article sources (URL/file/feeds), not YouTube Watch Later items (already summary-driven).
4. Dedupe is applied before rendering.
5. State DB is updated only for successfully processed items.
6. If at least one item succeeds, produce a PDF and report partial failures separately.

## Seen/Dedupe Logging Requirement
When an item is skipped due to seen-state, terminal output MUST include a consistent message regardless of source type.

Required log phrase:
- `already seen, skipping item`

Applies to:
- OPML/feed-originated items
- YouTube Watch Later items
- Any future source plugged into unified queue flow

Recommended format (non-breaking):
- `already seen, skipping item: <url-or-id> [source=<feeds|yt|file|arg>]`

## State and Queue Strategy
Use the existing SQLite state DB as the canonical store for dedupe and per-run queue state.

### Why not temp file
- Better crash recovery and observability
- No temp-file cleanup issues
- Single source of truth for skip/success/failure history

### Schema additions (proposed)
- `runs(run_id TEXT PRIMARY KEY, started_at INTEGER, finished_at INTEGER, mode TEXT, summarize INTEGER, page_size TEXT)`
- `run_items(run_id TEXT, item_key TEXT, source_type TEXT, url TEXT, title TEXT, status TEXT, error TEXT, created_at INTEGER, PRIMARY KEY(run_id, item_key))`

Existing `seen` table remains and continues to gate dedupe.

## Config Defaults
- Config file default:
  - `$XDG_CONFIG_HOME/rmfeeder/rmfeeder.toml`
  - fallback `~/.config/rmfeeder/rmfeeder.toml`
- Default feeds file:
  - `$XDG_CONFIG_HOME/rmfeeder/feeds.opml`
  - fallback `~/.config/rmfeeder/feeds.opml`

Relevant keys:
- `feeds_opml_path`
- `urls_path`
- `output_dir`
- `page_size`
- `limit` (or future split: `feeds_limit`, `yt_limit`)
- `state_db_path`

## Refactor Plan
1. Extract OPML and YouTube collection logic into library modules used by `rmfeeder`.
2. Add unified source-selection and merged-queue orchestration in `rmfeeder`.
3. Introduce run metadata/queue state tables in SQLite.
4. Normalize dedupe keys by source.
5. Standardize skip logging using the required phrase.
6. Upgrade argument parsing to `clap` for proper `--help`/validation.
7. Keep legacy helper binaries as wrappers (optional deprecation later).

## Error Handling
- Continue on per-item failures.
- Emit summary counts: attempted, included, skipped, failed.
- Exit nonzero only if no items were successfully included.

## Acceptance Criteria
1. One command covers feeds, YT watchlist, file, and direct URLs.
2. `--feeds --yt-watchlist` yields one merged bundle.
3. Seen-item skips are logged with `already seen, skipping item` for all sources.
4. `--summarize` behavior is source-aware (no YT re-summary).
5. `--help` outputs complete option docs.
6. Existing config defaults and path behavior remain compatible.
