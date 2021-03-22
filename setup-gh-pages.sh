#!/bin/bash
set -euxo pipefail

mkdir -p target
rm -rf target/doc
ln -s ../gh-pages target/doc
test -d gh-pages || git clone git@github.com:JelteF/derive_more --branch gh-pages gh-pages
cargo install --git https://github.com/JelteF/cargo-external-doc --force
