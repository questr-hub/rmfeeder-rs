#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST_PATH="$ROOT_DIR/rmfeeder/Cargo.toml"
SAMPLE_INPUT="$ROOT_DIR/testdata/target-sample.md"
OUTPUT_DIR="$ROOT_DIR/test-output/targets"

mkdir -p "$OUTPUT_DIR"

TARGET_FLAGS=$(cargo run --quiet --manifest-path "$MANIFEST_PATH" --bin rmfeeder -- --list-targets | tail -n +2 | cut -d',' -f1)

while IFS= read -r flag; do
  [[ -z "$flag" ]] && continue
  output_file="$OUTPUT_DIR/${flag}.pdf"
  echo "Rendering $flag -> $output_file"
  cargo run --quiet --manifest-path "$MANIFEST_PATH" --bin rmfeeder -- \
    --markdown "$SAMPLE_INPUT" \
    --page-size "$flag" \
    --output "$output_file"
done <<< "$TARGET_FLAGS"

echo "Done. Wrote target samples to $OUTPUT_DIR"
