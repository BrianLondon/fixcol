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

def tests [] {
    cargo +nightly docs-rs
    cargo test --workspace -- --include-ignored
    cargo test --workspace --features experimental-write -- --include-ignored
}

def "main test" [] {
    tests
}

def "main deploy" [] {
    # Confirm version consistency between all sub-crates
    let $ws_versions = cargo metadata --format-version 1 | from json | get workspace_members | each {|m| split row '#' | last }
    let $version = $ws_versions.0
    # assert every element of $ws_versions is equal to $version
    if ($ws_versions | any {|v| $v != $version}) {
        print 'Conflicting versions of crates to deploy.'
        exit 1
    }

    let $published_versions = http get https://crates.io/api/v1/crates/fixcol | get versions.num
    if ($published_versions | any { |v| $v == $version }) {
        print $'Version ($version) has already been released on crates.io.'
        exit 1
    }
    
    if (open CHANGELOG.md | lines | where $it =~ $version | is-empty) {
        print $'Version ($version) is not present in CHANGELOG. Please update.'
        exit 1
    }

    tests

    if (git -P tag --points-at HEAD | lines | any { |t| $t == $'v($version)' }) == false {
        # confirm tag does not exist and create it
        let $tagged_versions = git -P tag -l 'v*' | lines | each { |t| str substring 1..-1 }
        if ($tagged_versions | any { |t| $t == $version}) {
            print $'A git tag for version ($version) already exists.'
            exit 1
        }

        git tag $'v($version)'
        git push origin --tags
    }

    cd fixcol-derive
    cargo publish
    cd -
    cargo publish
}

def main [] {}
