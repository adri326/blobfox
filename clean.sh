#!/bin/sh

cargo run --bin clean -- "$1"
xmllint "$1" --format --output "$1"
