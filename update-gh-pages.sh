#!/bin/bash
set -euxo pipefail

# Go to latest release
latest_tag_hash=$(git rev-list --tags --max-count=1)
latest_tag=$(git describe "$latest_tag_hash")
git checkout "$latest_tag"

# build docs
rm -rf target/debug
cargo +nightly build
cargo +nightly external-doc
# go back to old branch
git checkout -

# Push doc changes
cd gh-pages
git add .
git commit -m "Update docs for $latest_tag release"
git push
