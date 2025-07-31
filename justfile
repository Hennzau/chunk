check:
    cargo check

build:
    cargo build

test:
    cargo test

publish-chii:
    cargo publish --package chii

publish-hej:
    cargo publish --package hej

publish-ulis:
    cargo publish --package ulis

publish-nux:
    cargo publish --package nux

publish-kyo:
    cargo publish --package kyo

run example:
    cargo run --example {{example}}
