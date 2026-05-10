#!/usr/bin/env bash
# Validate internal markdown links in the repository.
# Fails with non‑zero exit code if any link is broken.

set -e

echo "🔍 Checking markdown links..."

# Find all markdown files (excluding ignored paths)
md_files=$(find . -name "*.md" -type f | grep -v "\.git")

broken=0

for file in $md_files; do
    # Extract relative links: [text](./path) or [text](path)
    links=$(grep -oE '\]\([^\)]+\)' "$file" | sed 's/^\](/ /;s/)$//' | grep -v "^http" | grep -v "^#" | grep -v "^mailto")
    for link in $links; do
        # Resolve relative to the file's directory
        target="${file%/*}/$link"
        # Strip anchors
        target="${target%%#*}"
        if [[ ! -e "$target" ]]; then
            echo "❌ Broken link in $file → $link"
            broken=1
        fi
    done
done

if [[ $broken -eq 0 ]]; then
    echo "✅ All internal links are valid"
else
    echo "⚠️ Fix broken links before committing"
    exit 1
fi
