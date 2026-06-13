#!/usr/bin/env bash
set -e

# Run tests
./scripts/test.sh

# Build release
./scripts/build.sh

# Get version from Cargo.toml if not passed as an argument
VERSION=$1
if [ -z "$VERSION" ]; then
    VERSION=$(grep '^version =' Cargo.toml | head -n 1 | cut -d '"' -f 2)
fi

if [ -z "$VERSION" ]; then
    echo "Error: Version could not be determined."
    exit 1
fi

TAG="v$VERSION"
echo "Creating and tagging Git release: $TAG"

# Check if tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Warning: Tag $TAG already exists."
else
    git tag -a "$TAG" -m "Release $TAG"
    echo "Tag $TAG created."
fi

# Push tag (if remote exists)
if git remote | grep -q 'origin'; then
    git push origin "$TAG"
else
    echo "No 'origin' remote found, skipping tag push."
fi
