# {{project-name}}

A bare-metal Rust application for the HiSilicon **{{chip}}** RISC-V SoC, built on
the [ws63-rs](https://github.com/hispark-rs/hisi-riscv-rs) ecosystem
(`hisi-riscv-hal` + `hisi-riscv-rt` + {% if chip == "ws63" %}`ws63-pac`{% else %}`bs2x-pac`{% endif %}).

Starter: **{{starter}}**.

## Prerequisites

1. **The `hisi-riscv` toolchain** — a stable rustc with the `riscv32imfc-unknown-none-elf`
   target baked in as a builtin (hardware single-float, no atomics). It is *not* a
   normal rustup channel; install + link it once:

   ```bash
   curl -LO https://github.com/hispark-rs/hisi-riscv-rust-toolchain/releases/download/v1.96.0-2/hisi-riscv-rust-1.96.0-x86_64-unknown-linux-gnu.tar.gz
   tar xzf hisi-riscv-rust-1.96.0-x86_64-unknown-linux-gnu.tar.gz
   rustup toolchain link hisi-riscv "$PWD/stage2"
   ```

   `rust-toolchain.toml` already pins `channel = "hisi-riscv"`, so cargo picks it up here.

2. **QEMU** (for `cargo run`) — the [hisi-riscv-qemu](https://github.com/hispark-rs/hisi-riscv-qemu)
   fork, which adds the `-M {{chip}}` machine. Build it and put its
   `qemu-system-riscv32` on your `PATH`:

   ```bash
   git clone https://github.com/hispark-rs/hisi-riscv-qemu && cd hisi-riscv-qemu
   ./scripts/build.sh
   ```

## Build

```bash
cargo build --release
```

The target (`riscv32imfc-unknown-none-elf`) and the linker wiring (`-Tws63-link.x`)
are configured in `.cargo/config.toml` + `build.rs`, so a plain `cargo build` works.

## Run (QEMU)

```bash
cargo run --release
```

This boots the firmware on the `-M {{chip}}` machine via `qemu-system-riscv32`
(see the `runner` in `.cargo/config.toml`). UART0 is on stdio — quit with
**Ctrl-A** then **X**.
{% if starter == "blinky" %}
Expected: GPIO0 toggles every 500 ms (visible as pin-state changes in the machine
trace).
{% elsif starter == "uart_hello" %}
Expected output:

```
Hello from {{chip}} on QEMU!
UART0 @ {% if chip == "ws63" %}0x44010000{% else %}0x52081000{% endif %} is alive.
tick 0
tick 1
...
```
{% else %}
Expected: two async tasks interleave on the embassy executor:

```
[fast] tick 1
[fast] tick 2
[slow] tick 1
...
```
{% endif %}
## Flash to hardware

On-silicon flashing uses the vendor `burntool` / `loaderboot` flow (or the
in-progress [probe-rs](https://github.com/hispark-rs/probe-rs/tree/add-hisilicon-ws63-bs21)
support). Build a release binary, convert to the vendor image format, and flash
per the [hisi-riscv-rs HIL guide](https://github.com/hispark-rs/hisi-riscv-rs/tree/main/hil).

## Layout

| File | Purpose |
|------|---------|
| `src/main.rs` | The `{{starter}}` application. |
| `Cargo.toml` | Depends on `hisi-riscv-hal` / `hisi-riscv-rt`{% if starter == "async" %} + embassy{% endif %} from crates.io. |
| `.cargo/config.toml` | RISC-V target + the `cargo run` QEMU runner. |
| `rust-toolchain.toml` | Pins the custom `hisi-riscv` toolchain. |
| `build.rs` | Opts into hisi-riscv-rt's linker scripts. |
{% if chip != "ws63" %}| `memory.x` | The {{chip}} memory map (BS2X ships its own; WS63 uses the bundled one). |
{% endif %}

---

Generated from [hisi-rs-template](https://github.com/hispark-rs/hisi-rs-template)
via [`cargo generate`](https://cargo-generate.github.io/cargo-generate/).
