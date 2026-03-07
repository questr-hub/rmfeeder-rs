# Output Target Expansion Spec

## Developer Note: Existing Architecture

Current page-size handling is centralized in `PageSize` (`rmfeeder/src/lib.rs`).

- CLI accepts `--page-size <value>` and validates through `PageSize::parse`.
- Renderers (`pdf.rs`, `multipdf.rs`) do not pick sizes directly; they consume `page_size.page_override_css()`.
- Final page dimensions are applied by injecting `@page { size: ... }` into HTML before WeasyPrint conversion.

This build keeps that architecture intact and extends it additively.

## Safety Constraint

Existing stable targets keep their current CSS size values exactly:

- `letter` -> `letter`
- `rm1` -> `157.8mm 210.4mm`
- `rm2` -> `157.8mm 210.4mm`
- `rmpp` -> `179.6mm 239.5mm`
- `rmpp-move` -> `179.6mm 239.5mm`

No renderer-path redesign is introduced.

## Target Matrix

| Flag | Aliases | Width (px) | Height (px) | DPI basis | Description |
|---|---|---:|---:|---:|---|
| `letter` | - | 2550 | 3300 | 300 | US Letter |
| `rm1` | `remarkable1`, `remarkable-1` | 1404 | 1872 | 226 | reMarkable 1 |
| `rm2` | - | 1404 | 1872 | 226 | reMarkable 2 |
| `rmpp` | `rpp`, `paperpro`, `paper-pro`, `remarkable-paper-pro` | 1620 | 2160 | 229 | reMarkable Paper Pro |
| `rmpp-move` | `rmppm`, `rpp-move`, `paperpro-move`, `paper-pro-move`, `remarkable-paper-pro-move` | 1620 | 2160 | 229 | reMarkable Paper Pro Move |
| `scribe` | - | 1860 | 2480 | 300 | Kindle Scribe |
| `supernote-a5x` | - | 1920 | 2560 | 226 | Supernote A5X |
| `supernote-a5x2` | - | 1920 | 2560 | 226 | Supernote A5X2 |
| `supernote-a6x` | - | 1404 | 1872 | 226 | Supernote A6X |
| `supernote-a6x2` | - | 1404 | 1872 | 226 | Supernote A6X2 |
| `boox-go103` | - | 1860 | 2480 | 300 | Boox Go 10.3 |
| `boox-noteair` | - | 1860 | 2480 | 300 | Boox Note Air |
| `boox-noteair4c` | - | 1860 | 2480 | 300 | Boox Note Air4 C |
| `boox-noteair4c-color` | - | 930 | 1240 | 150 | Boox Note Air4 C Color Layer |
| `boox-notemax` | - | 2400 | 3200 | 300 | Boox Note Max |
| `a6` | - | 1240 | 1748 | 300 | ISO A6 |
| `a5` | - | 1748 | 2480 | 300 | ISO A5 |
| `a4` | - | 2480 | 3508 | 300 | ISO A4 |
| `ipad11` | - | 1668 | 2420 | 264 | iPad Pro 11-inch |
| `ipad13` | - | 2064 | 2752 | 264 | iPad Pro 13-inch |

## Listing Command

`rmfeeder --list-targets` prints:

- `flag,width,height,description`
- deterministic order
- one row per canonical target

## Review Artifacts

Use the script below to render one sample PDF per target:

```bash
./scripts/render-target-samples.sh
```

Outputs are written to:

- `test-output/targets/<flag>.pdf`
