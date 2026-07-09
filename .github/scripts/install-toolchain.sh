#!/usr/bin/env bash
set -euo pipefail

TOOLCHAIN="${RUST_TOOLCHAIN:-nightly-2026-07-09}"

rustup toolchain install "$TOOLCHAIN" \
  --profile minimal \
  --component rust-src \
  --component clippy \
  --component rustfmt \
  --component llvm-tools-preview

rustc +"$TOOLCHAIN" --version
rustc +"$TOOLCHAIN" --print target-list | grep -qx 'riscv32imfc-unknown-none-elf'
rustup target list --toolchain "$TOOLCHAIN" | grep -Eq '^riscv32imfc-unknown-none-elf([[:space:]]|$)' || \
  echo "rustup has no prebuilt rust-std for riscv32imfc-unknown-none-elf yet; use -Zbuild-std=core,alloc"
