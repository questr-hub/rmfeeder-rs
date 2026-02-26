# Tufte-Inspired Typography Spec for reMarkable

## Status
Archived experiment. We tested this theme and decided not to adopt it as the default; current production preference remains the Charter/Avenir styling on `main`.

## Goal
Adopt a more Tufte-like reading style (bookish serif voice, restrained hierarchy, calmer spacing) while keeping text comfortably readable on reMarkable devices.

## Inputs Reviewed
Gemini baseline proposal:

- Document Title / `h1`: `24pt` regular
- Major Section / `h2`: `20pt` regular
- Sub-section / `h3`: `17pt` bold
- Detailed Topic / `h4`: `14pt` bold
- Minor/Narrative / `h5`: `14pt` italic
- Body / normal: `14pt` regular
- Marginalia: `11pt` regular

## Constraints and Observations
- Current CSS baseline is tuned around Charter/Hoefler with smaller nominal sizes and relative scaling.
- On this machine:
  - `Hoefler Text` is installed.
  - `Charter` is installed.
  - `Menlo` and `PT Mono` are installed.
  - `Source Code Mono` is not installed.

## Typography Profile
Single serif profile: `Hoefler Text` (with Charter fallback).

## reMarkable Type Scale (Recommended)
These are target rendered sizes (pt), tuned for rm1/rm2/rmpp reading comfort.

| Role | Selector | Gemini | Hoefler Target | Style |
|---|---|---:|---:|---|
| Document Title | `h1` | 24 | 27.5 | Regular |
| Major Section | `h2` | 20 | 22 | Regular |
| Sub-section | `h3` | 17 | 18 | Regular |
| Detailed Topic | `h4` | 14 | 15 | Regular |
| Minor/Narrative | `h5` | 14 | 14 | Italic |
| Body Text | `p`, `li` | 14 | 14 | Regular |
| Marginalia/Notes | `.marginalia`, `figcaption`, metadata | 11 | 11.5 | Regular |
| Monospace | `code`, `pre` | - | 12.5 | Regular |

## Rhythm and Spacing
- Body `line-height`: `1.38` to `1.44` (target `1.4`)
- Headings `line-height`: `1.12` to `1.2`
- Paragraph spacing:
  - Keep vertical spacing for now: `margin-bottom: 0.75em` to `0.9em`
  - Optional later experiment: Tufte-style paragraph indent with reduced vertical gaps
- Heading spacing:
  - Top margin: `1.0em` to `1.2em`
  - Bottom margin: `0.25em` to `0.4em`

## Font Stacks
### Serif
- `"Hoefler Text", "Charter", "Iowan Old Style", "Palatino", "Times New Roman", serif`

### Monospace
Use `Menlo` as the primary monospaced face for this Tufte-inspired profile:

- `Menlo, "PT Mono", Consolas, monospace`

Rationale:
- `Menlo` is visually quieter and more book-compatible next to Hoefler than `PT Mono`.
- It preserves strong code legibility on e-ink at moderate sizes.
- It is installed and discovered by fontconfig/WeasyPrint in this environment.

## Implementation Notes
- Use CSS custom properties to make profile switching and tuning simple:
  - `--font-serif-body`
  - `--font-mono`
  - `--size-body`, `--size-h1` ... `--size-h5`, `--size-note`, `--size-code`
  - `--leading-body`, `--leading-headings`
- Keep existing page-size selection behavior unchanged.
- Apply this spec first to article content; cover and TOC can be aligned in a follow-up pass.

## Acceptance Criteria
- Reading comfort on rm2 is improved versus current default (larger apparent body size and cleaner hierarchy).
- `h1`/`h2` feel prominent without looking display-heavy.
- Long-form paragraphs remain easy to scan on e-ink with no cramped lines.
- Monospace blocks are clearly legible and visually subordinate to body text.
- Font fallback behavior is deterministic when a preferred mono face is unavailable.

## Rollout Plan
1. Implement the Hoefler profile with the scale above.
2. Render and visually review at least one long article on `rm1`, `rm2`, and `rmpp` page sizes.
3. Fine-tune by global `0.5pt` increments if pagination or density is off.
