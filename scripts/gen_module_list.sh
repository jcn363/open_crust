#!/bin/bash
# Generate a markdown list of all Rust source modules in src/

OUTPUT="docs/MODULES.md"

echo "# Source Modules" > "$OUTPUT"
echo "" >> "$OUTPUT"
echo "This file is auto‑generated. Do not edit manually." >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "## Module List" >> "$OUTPUT"
echo "" >> "$OUTPUT"

git ls-files "src/**/*.rs" | sed 's|^src/||' | sort | sed 's|^|- |' >> "$OUTPUT"

echo "Generated $OUTPUT with $(git ls-files "src/**/*.rs" | wc -l) modules"