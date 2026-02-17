# Page Size Option (v1: letter + rm2)

## Goal
Add a page-size option to PDF-producing binaries while keeping current behavior stable.

## Scope
- `rmfeeder`:
  - single URL output
  - URL bundle output (`--file` or multiple URLs)
- `yt_helper`:
  - Watch Later summary bundle output
- Config file support via `rmfeeder.toml`

Out of scope for this iteration:
- `opml_helper` PDF output (it does not generate PDFs)
- Additional sizes beyond `letter` and `rm2`

## CLI/API
- New flag:
  - `--page-size <letter|rm2>`
- Default:
  - `letter`
- Config key:
  - `page_size = "letter"` or `page_size = "rm2"`
- Precedence:
  - CLI flag overrides config value
  - config value overrides built-in default

## Rendering Behavior
- Keep `styles.css` as-is.
- Apply page size at render-time by injecting a small CSS override:
  - `@page { size: letter; }`
  - `@page { size: 157.8mm 210.4mm; }` for RM2
- This affects both single-article and bundle rendering paths.

## Validation
- Reject invalid `--page-size` values with a clear error.
- Reject invalid config `page_size` values with the same validation path.

## Non-Goals
- Do not change filename defaults.
- Do not auto-detect device.
- Do not alter typography/margins in this change.
