#!/bin/bash
set -euxo pipefail

# Go to latest release
latest_tag_hash=$(git rev-list --tags --max-count=1)
latest_tag=$(git describe "$latest_tag_hash")
git checkout "$latest_tag"

# Make sure we have gh-pages directory
if [ ! -d gh-pages ]; then
    git clone git@github.com:JelteF/derive_more --branch gh-pages gh-pages
fi

# Make sure the gh-pages directory has the latest commits
(
    cd gh-pages
    git fetch
    git checkout gh-pages
    git reset --hard origin/gh-pages
)

# Remove old docs
rm -rf gh-pages/*
# build docs
rm -rf target/debug
cargo +nightly build
cargo +nightly external-doc
# go back to old branch
git checkout -

# Push doc changes
(
    cd gh-pages
    git add .
    git commit -m "Update docs for $latest_tag release"
    git push
)

# Go back to previous commit instead of latest tag
git checkout -
