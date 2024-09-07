#!/usr/bin/env nu

def "main docs" [] {
    cargo +nightly docs-rs
    if $env.LAST_EXIT_CODE != 0 {
        print "docs-rs build failed"
        exit 1
    } else {
        cargo doc --no-deps --all-features
        cd fixcol-derive
        cargo doc --no-deps --document-private-items
    }
}

def "main test" [] {
    cargo +nightly docs-rs
    cargo test --workspace -- --include-ignored
    cargo test --workspace --features experimental-write -- --include-ignored
}

def main [] {}
