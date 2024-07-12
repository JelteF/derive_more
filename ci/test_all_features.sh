#!/usr/bin/env bash

std=''
if [ "${1:-}" = 'std' ]; then
    std=',std'
fi

set -euxo pipefail

for feature in $(tomljson Cargo.toml | jq --raw-output '.features | keys[]' | grep -v 'default\|std\|full\|testing-helpers'); do
    RUSTFLAGS='-D warnings' cargo +nightly test -p derive_more --tests --no-default-features --features "$feature$std,testing-helpers"
done
