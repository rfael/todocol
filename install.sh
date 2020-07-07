#!/bin/bash

readonly MY_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
readonly CONFIG_DIR="${HOME}/.config/todocol"
readonly BIN_DIR="${HOME}/.cargo/bin"

set -e

main() {
    cd ${MY_DIR}

    cargo build --release || exit 1

    echo "Copying binary file"
    mkdir -p "${BIN_DIR}"
    cp -v "${MY_DIR}/target/release/todocol" "${BIN_DIR}/todocol"

    echo "Copying setings file"
    mkdir -p "${CONFIG_DIR}"
    cp -v "${MY_DIR}/settings.json" "${CONFIG_DIR}/settings.json"

    # TODO: add option for installing zsh completion for todocol
}

main
