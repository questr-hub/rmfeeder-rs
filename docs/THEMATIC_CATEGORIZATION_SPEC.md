# Thematic Categorization for yt_helper

## Goal

After collecting fabric summaries for `--watch-later`, run a single LLM pass to group
articles into named thematic sections. The PDF TOC and body become two-level: section
headers containing grouped articles. Categorization is on by default; `--no-categories`
disables it.

## CLI

| Flag | Behavior |
|---|---|
| *(none, default)* | Categorization enabled |
| `--no-categories` | Skip categorization; flat list as today |

Config file: `categorize = false` in `~/.config/rmfeeder/config.toml` as a global default.
Flag overrides config.

## Data Flow

1. Fetch Watch Later playlist (unchanged)
2. Filter seen items via state DB (unchanged)
3. For each video: run fabric, retain raw markdown summary alongside rendered HTML
4. **Categorization pass** -- single Claude API call with all collected items
5. **Restructure** flat articles into `Vec<BundleArticle>` with `.section` set
6. Generate PDF bundle via existing `generate_pdf_bundle_with_sections`

## Categorization API Call

- Model: `claude-haiku-4-5-20251001`
- Auth: `ANTHROPIC_API_KEY` env var; clear error if missing when categorization is on
- Input to LLM: JSON array of `{ index, title, channel, summary }` -- full summaries for best signal
- Output format:
  ```json
  {
    "categories": [
      { "name": "AI & Machine Learning", "ordered_items": [2, 0, 5] },
      { "name": "Homelab & Self-Hosting", "ordered_items": [3, 7] }
    ],
    "other": [1, 4]
  }
  ```
- LLM instructions: 3-6 named categories; concise names; LLM orders items within each for
  reading flow; items in `other` get a catch-all "Other" section at the end

## Failure Handling

If the API call fails, times out, or returns unparseable output:
- Print warning to stderr: `Warning: categorization failed (<reason>); falling back to flat list`
- Continue with flat uncategorized bundle -- no PDF lost

## Edge Cases

| Scenario | Behavior |
|---|---|
| 0 articles | Unchanged -- exits with error before categorization |
| 1 article | Skip categorization; flat output |
| All items land in "Other" | Warn to stderr; produce PDF with just the "Other" section |
| `--dry-run` | Implies `--no-categories`; no API cost in dry runs |
| `other` vec is empty | No "Other" section rendered |

## Implementation Notes

- `BundleArticle.section` already exists in `multipdf.rs`
- `generate_pdf_bundle_with_sections` already renders section headers in TOC
- No new multipdf variant needed -- just populate `.section` per article
- `reqwest::blocking` already a dep; no new crates required
- LLM response JSON extraction handles markdown-fenced output gracefully
- Out-of-range indices in LLM response are silently dropped with a warning
