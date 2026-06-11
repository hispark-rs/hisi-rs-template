//! Build script: opt this binary into hisi-riscv-rt's linker scripts.
//!
//! The linker scripts live in hisi-riscv-rt. A library dependency's
//! `cargo:rustc-link-arg` does NOT propagate to a downstream binary, so the
//! binary must opt in itself with `-Tws63-link.x`. hisi-riscv-rt exposes its
//! OUT_DIR (containing `ws63-link.x` + the scripts it INCLUDEs) via
//! `cargo:rustc-link-search`, which DOES propagate, so this `-T` resolves.
{%- if chip != "ws63" %}
//!
//! {{chip}} has its own memory map, so this crate ships its own `memory.x`
//! (hisi-riscv-rt's bundled WS63 one is off via `default-features = false`).
//! Copy ours into OUT_DIR and add that dir to the link search path, so
//! `ws63-link.x`'s `INCLUDE memory.x` resolves to THIS file (exactly one on the
//! path → no ambiguity).
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");

    println!("cargo:rustc-link-arg=-Tws63-link.x");
    println!("cargo:rerun-if-changed=build.rs");
}
{%- else %}

fn main() {
    println!("cargo:rustc-link-arg=-Tws63-link.x");
    println!("cargo:rerun-if-changed=build.rs");
}
{%- endif %}
