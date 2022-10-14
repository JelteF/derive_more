#!/bin/bash
set -euxo pipefail

for feature in $(tomljson Cargo.toml | jq --raw-output '.features | keys[]' | grep -v 'default\|generate-parsing-rs'); do
    cargo test --tests --no-default-features --features "$feature,testing-helpers";
done
