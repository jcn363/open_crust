#!/usr/bin/env bash
# Generate search index for Zola multilingual site
# This script runs after `zola build` to create a unified search index

set -euo pipefail

SITE_DIR="site"
PUBLIC_DIR="${SITE_DIR}/public"
SEARCH_INDEX="${PUBLIC_DIR}/search_index.json"

echo "Generating search index..."

# Check if zola build output exists
if [[ ! -d "${PUBLIC_DIR}" ]]; then
    echo "Error: ${PUBLIC_DIR} not found. Run 'zola build' first."
    exit 1
fi

# Zola already generates search_index.json when build_search_index = true
# This script can be extended for custom search index processing if needed
# For now, we just verify the index exists and is valid

if [[ -f "${SEARCH_INDEX}" ]]; then
    echo "Search index found at ${SEARCH_INDEX}"
    
    # Validate JSON
    if command -v jq &> /dev/null; then
        jq empty "${SEARCH_INDEX}" && echo "Search index JSON is valid"
    else
        echo "jq not installed, skipping JSON validation"
    fi
    
    # Show stats
    if command -v jq &> /dev/null; then
        DOC_COUNT=$(jq '.docs | length' "${SEARCH_INDEX}")
        echo "Indexed documents: ${DOC_COUNT}"
    fi
else
    echo "Warning: Search index not found at ${SEARCH_INDEX}"
    echo "Make sure build_search_index = true in config.toml"
    exit 1
fi

echo "Search index generation complete!"