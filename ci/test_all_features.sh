#!/usr/bin/env bash
set -euxo pipefail

for feature in $(tomljson Cargo.toml | jq --raw-output '.features | keys[]' | grep -v 'default\|std\|testing-helpers'); do
    cargo test -p derive_more --tests --no-default-features --features "$feature,testing-helpers";
done
