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
   # Release v1.96.0-2 ships four hosts — swap the triple below to match yours:
   #   x86_64-unknown-linux-gnu · aarch64-unknown-linux-gnu · aarch64-apple-darwin · x86_64-pc-windows-msvc
   HOST=x86_64-unknown-linux-gnu
   curl -LO https://github.com/hispark-rs/hisi-riscv-rust-toolchain/releases/download/v1.96.0-2/hisi-riscv-rust-1.96.0-$HOST.tar.gz
   tar xzf hisi-riscv-rust-1.96.0-$HOST.tar.gz
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

> Validated end-to-end on real {% if chip == "ws63" %}WS63{% else %}{{chip}}{% endif %} silicon
> (blinky boots + blinks, 2026-06-14){% if chip != "ws63" %} for WS63; the BS2X address
> below is `hisi-fwpkg`'s default and is **not yet HIL-validated** — confirm it
> against your `fbb_bs2x` partition table first{% endif %}.

A bare ELF/`.bin` does **not** boot. flashboot loads the app partition at flash
`{{app_partition_addr}}` and jumps **unconditionally** to `{{app_partition_addr}} + 0x300`,
so the image needs the fixed 0x300-byte HiSilicon header in front of the code.
`hisi-fwpkg image` adds exactly that.

No real signing key is required for the dev flow: the dev chips ship with secure
boot **disabled** (efuse `SEC_VERIFY_ENABLE == 0`), so flashboot's `verify_image_*`
short-circuits to success before it ever reads the signature. `hisi-fwpkg` writes
a structurally-correct header with **zeroed ("dummy") signature** fields, which is
sufficient on those boards.

### Tools

* **`hisi-fwpkg`** — packs the ELF/bin into a bootable image (or a `.fwpkg`):
  ```bash
  cargo install --git https://github.com/hispark-rs/hisi-fwpkg
  ```
* **The patched `probe-rs` FORK** — *upstream probe-rs does not support {{chip}} yet.*
  Install the fork (it also ships the `HiSilicon_WS63.yaml` chip description):
  ```bash
  cargo install --git https://github.com/hispark-rs/probe-rs \
      --branch add-hisilicon-ws63-bs21 probe-rs-tools
  ```

### One command (via `just`)

This project ships a [`justfile`](https://github.com/casey/just):

```bash
just flash    # build -> hisi-fwpkg image -> probe-rs download @ app partition -> reset
```

Point it at the fork's YAML if it isn't in the cwd:

```bash
just CHIP_DESC=/path/to/HiSilicon_WS63.yaml flash
```

### Manual (what `just flash` runs)

```bash
cargo build --release
hisi-fwpkg image -o {{crate_name}}.img \
    target/riscv32imfc-unknown-none-elf/release/{{crate_name}}

# Flash via the patched probe-rs FORK (the validated path):
probe-rs download --chip {% if chip == "ws63" %}WS63{% else %}{{chip | upcase}}{% endif %} \
    --chip-description-path HiSilicon_WS63.yaml \
    --binary-format bin --base-address {{app_partition_addr}} {{crate_name}}.img
probe-rs reset --chip {% if chip == "ws63" %}WS63{% else %}{{chip | upcase}}{% endif %} \
    --chip-description-path HiSilicon_WS63.yaml
```

### Vendor alternative (`.fwpkg` + `hisiflash`)

No probe-rs fork needed — build a single-partition package and flash it with the
vendor tool (`hisi-fwpkg` picks the per-chip app address itself):

```bash
just fwpkg                       # -> {{crate_name}}.fwpkg
hisiflash flash {{crate_name}}.fwpkg
```

See the [hisi-riscv-rs HIL guide](https://github.com/hispark-rs/hisi-riscv-rs/tree/main/hil)
for wiring and the vendor `burntool` / `loaderboot` UART flow.

## Layout

| File | Purpose |
|------|---------|
| `src/main.rs` | The `{{starter}}` application. |
| `Cargo.toml` | Depends on `hisi-riscv-hal` / `hisi-riscv-rt`{% if starter == "async" %} + embassy{% endif %} from crates.io. |
| `.cargo/config.toml` | RISC-V target + the `cargo run` QEMU runner. |
| `justfile` | `just build` / `run` / `image` / `flash` / `fwpkg` convenience recipes. |
| `rust-toolchain.toml` | Pins the custom `hisi-riscv` toolchain. |
| `build.rs` | Opts into hisi-riscv-rt's linker scripts. |
{% if chip != "ws63" %}| `memory.x` | The {{chip}} memory map (BS2X ships its own; WS63 uses the bundled one). |
{% endif %}

---

Generated from [hisi-rs-template](https://github.com/hispark-rs/hisi-rs-template)
via [`cargo generate`](https://cargo-generate.github.io/cargo-generate/).
