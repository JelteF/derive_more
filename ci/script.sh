# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    export RUSTFLAGS="-D warnings"
    cross build --target "$TARGET"
    cross build --target "$TARGET" --release

    if [ -n "$DISABLE_TESTS" ]; then
        return
    fi

    cross test --target "$TARGET"
    cross test --target "$TARGET" --release
}

# we don't run the "test phase" when doing deploys
main
