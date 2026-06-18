#!/bin/bash
# Release script for OpenCrust
# Following DeepSpeed's release automation pattern

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the version from version.txt
VERSION=$(cat version.txt | tr -d '\n')
TAG="v${VERSION}"

echo -e "${GREEN}=== OpenCrust Release Script ===${NC}"
echo -e "Version: ${VERSION}"
echo -e "Tag: ${TAG}"
echo ""

# Check if we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ] && [ "$CURRENT_BRANCH" != "master" ]; then
    echo -e "${RED}ERROR: Must be on main or master branch to release${NC}"
    exit 1
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}ERROR: Uncommitted changes detected. Commit or stash them first.${NC}"
    exit 1
fi

# Validate version
echo -e "${YELLOW}Validating version...${NC}"
python3 scripts/check_release_version.py "${VERSION}"

# Run tests
echo -e "${YELLOW}Running tests...${NC}"
cargo test

# Check formatting
echo -e "${YELLOW}Checking formatting...${NC}"
cargo fmt -- --check

# Run clippy
echo -e "${YELLOW}Running clippy...${NC}"
cargo clippy -- -D warnings

# Build release
echo -e "${YELLOW}Building release...${NC}"
cargo build --release

# Create git tag
echo -e "${YELLOW}Creating git tag...${NC}"
git tag -a "${TAG}" -m "Release ${VERSION}"

# Push tag
echo -e "${YELLOW}Pushing tag...${NC}"
git push origin "${TAG}"

# Bump patch version for next development cycle
echo -e "${YELLOW}Bumping patch version for next development cycle...${NC}"
python3 scripts/bump_patch_version.py

# Commit version bump
NEW_VERSION=$(cat version.txt | tr -d '\n')
git add version.txt Cargo.toml
git commit -m "chore: bump version to ${NEW_VERSION} for next development cycle"

# Push version bump
echo -e "${YELLOW}Pushing version bump...${NC}"
git push origin "${CURRENT_BRANCH}"

echo -e "${GREEN}=== Release ${VERSION} completed successfully! ===${NC}"
echo -e "Next development version: ${NEW_VERSION}"
echo ""
echo "Next steps:"
echo "1. Verify the release on GitHub: https://github.com/jcn363/open_crust/releases/tag/${TAG}"
echo "2. Verify the binary artifacts are uploaded"
echo "3. Update documentation if needed"