#!/bin/bash

readonly MY_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
readonly CONFIG_DIR=$HOME/.config/todocol
readonly BIN_DIR=$HOME/.cargo/bin

main() {
    cd $MY_DIR || exit 1

    cargo build --release || exit 1

    [ -d $BIN_DIR ] && cp $MY_DIR/target/release/todocol $BIN_DIR/todocol

    mkdir -p $CONFIG_DIR

    cp -v $MY_DIR/settings.json $CONFIG_DIR/settings.json
}

main
