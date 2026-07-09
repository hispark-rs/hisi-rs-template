#!/usr/bin/env bash
set -euo pipefail

HOST="${HOST_TRIPLE:-x86_64-unknown-linux-gnu}"
VERSION="${HISI_RISCV_TOOLCHAIN_VERSION:-1.96.0}"
RELEASE="${HISI_RISCV_TOOLCHAIN_RELEASE:-v1.96.0-2}"
URL="https://github.com/hispark-rs/hisi-riscv-rust-toolchain/releases/download/${RELEASE}/hisi-riscv-rust-${VERSION}-${HOST}.tar.gz"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

curl -fL "$URL" -o "$tmp/toolchain.tar.gz"
mkdir -p "$HOME/.rustup/toolchains/hisi-riscv"
tar xzf "$tmp/toolchain.tar.gz" --strip-components=1 -C "$HOME/.rustup/toolchains/hisi-riscv"
rustup toolchain list | grep -q '^hisi-riscv'

