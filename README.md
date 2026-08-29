# {{project-name}}

A bare-metal Rust application for the HiSilicon **{{chip}}** RISC-V SoC, built on
the [ws63-rs](https://github.com/hispark-rs/hisi-riscv-rs) ecosystem
(`hisi-hal` + `hisi-riscv-rt` + {% if chip == "ws63" %}`ws63-pac`{% else %}`bs2x-pac`{% endif %}).

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

{% if starter != "wifi" and starter != "radio" %}2. **QEMU** (for `cargo run`) — the [hisi-riscv-qemu](https://github.com/hispark-rs/hisi-riscv-qemu)
   fork, which adds the `-M {{chip}}` machine. Build it and put its
   `qemu-system-riscv32` on your `PATH`:

   ```bash
   git clone https://github.com/hispark-rs/hisi-riscv-qemu && cd hisi-riscv-qemu
   ./scripts/build.sh
   ```
{% endif %}

## Build

```bash
{% if starter == "wifi" %}WS63_WIFI_SSID='your-network' WS63_WIFI_PASSPHRASE='your-passphrase' \
    cargo build -Zbuild-std=core,alloc --release
{% else %}
cargo build -Zbuild-std=core,alloc --release
{% endif %}
```

The target (`riscv32imfc-unknown-none-elf`) and the linker wiring (`-Thisi-riscv-link.x`)
are configured in `.cargo/config.toml` + `build.rs`. Use `just build` if you
prefer not to type the build-std flag.

{% if starter == "wifi" %}
## Run on WS63

The RF payload calls the WS63 mask ROM, so QEMU can compile this starter but
cannot execute its radio path. Build credentials are embedded only in the final
firmware image through temporary environment variables; they are not stored in
the generated source:

```bash
WS63_WIFI_SSID='your-network' WS63_WIFI_PASSPHRASE='your-passphrase' just flash
```

The application starts the native RTOS port, claims the named
`profile-wifi-wpa2-smoltcp` resources, and runs the storage-bound `RadioRunner`
beside the Wi-Fi control and smoltcp tasks on one Embassy executor. The control
task performs initialize, scan and connect before handing `WifiDevice` to a
long-lived smoltcp DHCP loop. Expected UART markers are `WIFI_INIT_OK`,
`WIFI_SCAN_OK`, `WIFI_CONNECT_OK`, and finally `WIFI_DHCP_OK`.

`just image` also emits `{{crate_name}}.resource.json` from the selected public
`hisi-rf` profile. The host-only helper uses the same event capacity as the
firmware and depends on no WS63 sys/blob/RTOS-driver crate directly. Review this
resource contract together with `{{crate_name}}.plan.json` before changing the
profile or memory layout.
{% elsif starter == "radio" %}
## Run on WS63

This project selects the named `profile-{{radio_profile}}` composition. It owns
one static radio arena, starts the WS63 RTOS port, initializes the shared radio
controller, then exercises the profile's first typed operation:

| Profile | Starter operation |
|---|---|
| `ble-peripheral` | BLE advertising |
| `ble-central` | BLE passive scan |
| `ble-dual-role` | BLE advertising; central methods remain available |
| `sle-announce` | SLE announce |
| `sle-seek` | SLE seek |
| `sle-ssap` | SLE announce; SSAP registration remains available |

Build and flash with `just flash`. Successful initialization and command
acceptance emit `RADIO_INIT_OK profile={{radio_profile}}` and
`RADIO_COMMAND_OK profile={{radio_profile}}`. These markers prove the selected
controller/host accepted the operation; peer-visible behavior still requires a
second-board HIL.

`just resource-report` writes `{{crate_name}}.resource.json` from the same
caller-owned `RadioStorage` used by the firmware. It is the machine-readable
source for arena bytes, control bytes, task slots/stacks, and event capacities;
the generated project does not duplicate those numbers.
{% else %}
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
{% endif %}
## Flash to hardware

{% if starter == "wifi" %}> The generated project's build and image contracts are CI-validated. On real WS63
> silicon, treat the four UART markers above as the end-to-end HIL acceptance
> gate; do not infer radio success from a successful build alone.
{% elsif starter == "radio" %}> The generated project's build and image contracts are CI-validated. On real WS63
> silicon, require the two `RADIO_*_OK` markers above and a peer-visible
> profile-specific HIL before treating the radio operation as end-to-end proven.
{% else %}> Validated end-to-end on real {% if chip == "ws63" %}WS63{% else %}{{chip}}{% endif %} silicon
> (blinky boots + blinks, 2026-06-14){% if chip != "ws63" %} for WS63; the BS2X address
> below is `hisi-fwpkg`'s default and is **not yet HIL-validated** — confirm it
> against your `fbb_bs2x` partition table first{% endif %}.
{% endif %}

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
  cargo +stable install hisi-fwpkg-cli --version 0.3.2
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

probe-rs download --chip {{chip | upcase}} \
    --chip-description-path HiSilicon_WS63.yaml \
    --binary-format bin --base-address {{app_partition_addr}} {{crate_name}}.img
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
{% if starter == "wifi" %}| `src/wifi.rs` | The WS63 radio/runtime/smoltcp happy path; credentials come only from build environment variables. |
{% elsif starter == "radio" %}| `src/radio.rs` | The WS63 `profile-{{radio_profile}}` composition, typed operation, runner, and RTOS wiring. |
{% endif %}| `Cargo.toml` | Depends on `hisi-hal` / `hisi-riscv-rt`{% if starter == "async" %} + embassy{% elsif starter == "wifi" %} + `hisi-rf` / `hisi-rtos` / smoltcp{% elsif starter == "radio" %} + `hisi-rf` / `hisi-rtos`{% endif %} from crates.io. |
| `.cargo/config.toml` | RISC-V target + the `cargo run` QEMU runner. |
| `justfile` | `just build` / `run` / `image` / `flash`{% if chip == "ws63" %} / `patch` / `run-hw`{% endif %} / `fwpkg` convenience recipes. |
| `rust-toolchain.toml` | Pins the official nightly used by this project. |
| `build.rs` | Opts into hisi-riscv-rt's linker scripts. |
{% if chip != "ws63" %}| `memory.x` | The {{chip}} memory map (BS2X ships its own; WS63 uses the bundled one). |
{% endif %}

---

Generated from [hisi-rs-template](https://github.com/hispark-rs/hisi-rs-template)
via [`cargo generate`](https://cargo-generate.github.io/cargo-generate/).
