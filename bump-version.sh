#!/bin/bash

# Check if version type is provided
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <major|minor|patch>"
    exit 1
fi

TYPE=$1
CURRENT_VERSION=$(grep '^version =' Cargo.toml | head -n 1 | cut -d '"' -f 2)

# Split version into components
IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
MAJOR=${VERSION_PARTS[0]}
MINOR=${VERSION_PARTS[1]}
PATCH=${VERSION_PARTS[2]}

# Update version based on type
if [ "$TYPE" = "major" ]; then
    MAJOR=$((MAJOR + 1))
    MINOR=0
    PATCH=0
elif [ "$TYPE" = "minor" ]; then
    MINOR=$((MINOR + 1))
    PATCH=0
elif [ "$TYPE" = "patch" ]; then
    PATCH=$((PATCH + 1))
else
    echo "Invalid version type. Use major, minor, or patch"
    exit 1
fi

NEW_VERSION="$MAJOR.$MINOR.$PATCH"

# Update Cargo.toml
sed -i.bak "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Run cargo check to update Cargo.lock
echo "Updating Cargo.lock..."
cargo check --quiet

# Commit the change
git add Cargo.toml Cargo.lock
git commit -m "v$NEW_VERSION"

# Create and push tag
git tag -a "v$NEW_VERSION" -m "Version $NEW_VERSION"

echo "Version bumped from $CURRENT_VERSION to $NEW_VERSION"
echo "Run 'git push && git push --tags' to push changes"