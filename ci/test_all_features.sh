#!/usr/bin/env bash
set -euxo pipefail

for feature in $(tomljson Cargo.toml | jq --raw-output '.features | keys[]' | grep -v 'default\|std\|full\|testing-helpers'); do
    if [ "${1:-}" = 'std' ]; then
        cargo +nightly test -p derive_more --tests --no-default-features --features "$feature,std,testing-helpers";
    else
        cargo +nightly test -p derive_more --tests --no-default-features --features "$feature,testing-helpers";
    fi
done
