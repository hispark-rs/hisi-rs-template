# {{project-name}} — build / package / flash recipes for the HiSilicon {{chip}}.
#
# `just` is https://github.com/casey/just (`cargo install just`). These recipes
# wrap the hardware-validated WS63 flash flow (validated on silicon 2026-06-14):
#
#     cargo build -Zbuild-std=core,alloc --release
#     hisi-fwpkg plan   -> app.img + app.plan.json
#     probe-rs download --binary-format bin -> flash @ plan.base_addr
#     probe-rs reset    -> run
#
# Prereqs for the flashing recipes (the build/run ones need none of these):
#   * hisi-fwpkg  — cargo +stable install hisi-fwpkg-cli --version 0.3.2
#   * the PATCHED probe-rs FORK (upstream probe-rs does NOT know {{chip}} yet):
#       cargo install --git https://github.com/hispark-rs/probe-rs \
#           --branch add-hisilicon-ws63-bs21-hil-baseline probe-rs-tools
#   * its chip description YAML (HiSilicon_WS63.yaml), shipped in that fork.

# The release ELF cargo produces (cargo names the artifact after [[bin]] = crate_name).
elf := "target/riscv32imfc-unknown-none-elf/release/{{crate_name}}"
# The packaged, bootable image (0x300 HiSilicon header || body).
img := "{{crate_name}}.img"
# The machine-readable flash plan produced by hisi-fwpkg.
plan := "{{crate_name}}.plan.json"
# The single-partition vendor package (the `fwpkg` recipe / hisiflash path).
fwpkg_out := "{{crate_name}}.fwpkg"
# Official upstream rustc has the target built in; rustup does not ship prebuilt
# rust-std for it yet, so firmware builds compile core/alloc from rust-src.
build_std := "-Zbuild-std=core,alloc"

# probe-rs fork chip name + the path to its chip-description YAML. Override on the
# CLI, e.g.  `just CHIP_DESC=~/probe-rs/HiSilicon_WS63.yaml flash`
CHIP        := {% if chip == "ws63" %}"WS63"{% else %}"{{chip | upcase}}"{% endif %}
CHIP_DESC   := "HiSilicon_WS63.yaml"
# Flash address override. hisi-fwpkg knows the default; this value is passed only
# when you choose to override the generated plan.
APP_ADDR    := "{{app_partition_addr}}"

{% raw %}_default:
    @just --list

# Build the release firmware ELF.
build:
    cargo build {{build_std}} --release

# Run in QEMU (the hisi-riscv-qemu fork; Ctrl-A then X to quit).
{% endraw %}{% if starter == "wifi" %}{% raw %}
run: build
    @echo "Wi-Fi needs a real WS63; use 'just flash'"
{% endraw %}{% else %}{% raw %}
run:
    cargo run {{build_std}} --release
{% endraw %}{% endif %}{% raw %}

{% endraw %}{% if chip == "ws63" %}{% raw %}
# With the `boot-header` feature the 0x300 HiSilicon header is baked into the ELF
# at link time. This recipe is only for ELF-based `probe-rs run` / embedded-test
# paths that need symbols and semihosting metadata. Normal flashing uses `image`.
patch: build
    hisi-fwpkg patch-hash {{elf}}

# The ELF-based hardware run/debug path. Use `flash` for smoke/download.
# Ctrl-C to stop.
run-hw: patch
    probe-rs run --chip {{CHIP}} --chip-description-path {{CHIP_DESC}} {{elf}}
{% endraw %}{% else %}{% raw %}
# The hardware equivalent of `just run`: flash, then stream UART0 (CH340, not the
# J-Link VCOM). Ctrl-C to stop.
run-hw PORT='/dev/ttyUSB0' BAUD='115200': flash
    @echo "streaming {{PORT}} @ {{BAUD}} 8N1 (Ctrl-C to stop)"
    @stty -F {{PORT}} {{BAUD}} raw -echo
    @cat {{PORT}}
{% endraw %}{% endif %}{% raw %}

# Build the complete flash image and machine-readable plan. hisi-fwpkg owns the
# header/hash/body-range semantics; probe-rs only writes the resulting bin.
image: build
    hisi-fwpkg plan {{elf}} --chip {% endraw %}{% if chip == "ws63" %}ws63{% else %}bs21{% endif %}{% raw %} --app-addr {{APP_ADDR}} --image-output {{img}} > {{plan}}

# Flash {{img}} to the plan base address via the PATCHED probe-rs fork, then reset.
flash: image
    probe-rs download --chip {{CHIP}} --chip-description-path {{CHIP_DESC}} \
        --binary-format bin --base-address {{APP_ADDR}} {{img}}
    probe-rs reset --chip {{CHIP}} --chip-description-path {{CHIP_DESC}}

# hisi-fwpkg picks the per-chip app address itself; no probe-rs fork needed.
#
# Vendor path: build a single-partition .fwpkg to flash with `hisiflash`.
fwpkg: build
    hisi-fwpkg pack --chip {% endraw %}{% if chip == "ws63" %}ws63{% else %}bs21{% endif %}{% raw %} -o {{fwpkg_out}} {{elf}}
    @echo "Now flash with: hisiflash flash {{fwpkg_out}}"

# Remove build + packaging artifacts.
clean:
    cargo clean
    -rm -f {{img}} {{plan}} {{fwpkg_out}}
{% endraw %}
