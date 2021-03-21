#!/bin/bash
set -euxo pipefail

rm -rf target/debug
cargo +nightly build
cargo +nightly external-doc
cd gh-pages
git add .
git commit -m 'Updated docs to latest version'
git push
