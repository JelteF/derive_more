#!/bin/bash

shopt -s globstar
set -x
cargo external-doc

while true
do
    inotifywait -e move_self -e modify **/*.rs **/*.md
    cargo external-doc
    sleep .2
done
