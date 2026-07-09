# {{project-name}}

A bare-metal Rust application for the HiSilicon **{{chip}}** RISC-V SoC, built on
the [ws63-rs](https://github.com/hispark-rs/hisi-riscv-rs) ecosystem
(`hisi-riscv-hal` + `hisi-riscv-rt` + {% if chip == "ws63" %}`ws63-pac`{% else %}`bs2x-pac`{% endif %}).

Starter: **{{starter}}**.

## Prerequisites

1. **Official Rust nightly** — `rustc` has the
   `riscv32imfc-unknown-none-elf` target built in (hardware single-float, no
   atomics). rustup does not ship a prebuilt `rust-std` component for this target
   yet, so the project pins nightly and builds `core`/`alloc` from `rust-src`:

   ```bash
   rustup toolchain install nightly-2026-07-09 \
       --profile minimal \
       --component rust-src \
       --component clippy \
       --component rustfmt \
       --component llvm-tools-preview
   ```

   `rust-toolchain.toml` already pins this nightly, so cargo picks it up here.

2. **QEMU** (for `cargo run`) — the [hisi-riscv-qemu](https://github.com/hispark-rs/hisi-riscv-qemu)
   fork, which adds the `-M {{chip}}` machine. Build it and put its
   `qemu-system-riscv32` on your `PATH`:

   ```bash
   git clone https://github.com/hispark-rs/hisi-riscv-qemu && cd hisi-riscv-qemu
   ./scripts/build.sh
   ```

## Build

```bash
cargo build -Zbuild-std=core,alloc --release
```

The target (`riscv32imfc-unknown-none-elf`) and the linker wiring (`-Thisi-riscv-link.x`)
are configured in `.cargo/config.toml` + `build.rs`. Use `just build` if you
prefer not to type the build-std flag.

## Run (QEMU)

```bash
cargo run -Zbuild-std=core,alloc --release
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
`hisi-fwpkg plan` turns the ELF into a complete flash image and a machine-readable
plan. The plan owns the header/hash/body-range/base-address semantics; probe-rs
only writes the resulting binary image.

No real signing key is required for the dev flow: the dev chips ship with secure
boot **disabled** (efuse `SEC_VERIFY_ENABLE == 0`), so flashboot's `verify_image_*`
short-circuits to success before it ever reads the signature — a structurally
correct header with a **zeroed ("dummy") signature** is enough. flashboot does
still check the **body hash** even with secure-verify off, so the header's
`code_area_hash` must match the code. `hisi-fwpkg plan` computes it while building
the image. On WS63, `patch-hash` is only for ELF-based `probe-rs run` /
embedded-test paths that need symbols and semihosting metadata.

> ⚠ If you ever **enable** secure boot (burn `SEC_VERIFY_ENABLE`), this
> dummy-signature flow stops booting — you then need the vendor signing tools and
> your provisioned keys (ECC/RSA), which are out of scope for this template.

### Tools

* **`hisi-fwpkg`** — packs the ELF/bin into a bootable image (or a `.fwpkg`):
  ```bash
  cargo +stable install hisi-fwpkg-cli --version 0.3.0
  ```
* **The patched `probe-rs` FORK** — *upstream probe-rs does not support {{chip}} yet.*
  Install the fork (it also ships the `HiSilicon_WS63.yaml` chip description):
  ```bash
  cargo install --git https://github.com/hispark-rs/probe-rs \
      --branch add-hisilicon-ws63-bs21-hil-baseline probe-rs-tools
  ```

### One command (via `just`)

This project ships a [`justfile`](https://github.com/casey/just):

```bash
just flash    # build -> hisi-fwpkg plan image -> probe-rs bin download @ plan.base_addr -> reset
```

Point it at the fork's YAML if it isn't in the cwd:

```bash
just CHIP_DESC=/path/to/HiSilicon_WS63.yaml flash
```

### Manual (what `just flash` runs)

```bash
cargo build -Zbuild-std=core,alloc --release

hisi-fwpkg plan target/riscv32imfc-unknown-none-elf/release/{{crate_name}} \
    --chip {% if chip == "ws63" %}ws63{% else %}bs21{% endif %} \
    --app-addr {{app_partition_addr}} \
    --image-output {{crate_name}}.img \
    > {{crate_name}}.plan.json

BASE_ADDR=$(python3 -c 'import json; print(json.load(open("{{crate_name}}.plan.json"))["base_addr"])')
probe-rs download --chip {{chip | upcase}} \
    --chip-description-path HiSilicon_WS63.yaml \
    --binary-format bin --base-address "$BASE_ADDR" {{crate_name}}.img
probe-rs reset --chip {{chip | upcase}} \
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
| `justfile` | `just build` / `run` / `image` / `flash`{% if chip == "ws63" %} / `patch` / `run-hw`{% endif %} / `fwpkg` convenience recipes. |
| `rust-toolchain.toml` | Pins the official nightly used by this project. |
| `build.rs` | Opts into hisi-riscv-rt's linker scripts. |
{% if chip != "ws63" %}| `memory.x` | The {{chip}} memory map (BS2X ships its own; WS63 uses the bundled one). |
{% endif %}

---

Generated from [hisi-rs-template](https://github.com/hispark-rs/hisi-rs-template)
via [`cargo generate`](https://cargo-generate.github.io/cargo-generate/).
