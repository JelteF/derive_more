#!/bin/bash
set -ex

for feature in $(tomljson Cargo.toml | jq --raw-output '.features | keys[]' | grep -v 'default\|nightly\|generate-parsing-rs'); do
    cargo test --tests --no-default-features --features  "$feature";
done

