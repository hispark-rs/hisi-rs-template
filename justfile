# {{project-name}} — build / package / flash recipes for the HiSilicon {{chip}}.
#
# `just` is https://github.com/casey/just (`cargo install just`). These recipes
# wrap the hardware-validated WS63 flash flow (validated on silicon 2026-06-14):
#
#     cargo build --release
#     hisi-fwpkg image  -> app.img   (adds the 0x300 HiSilicon header)
#     probe-rs download -> flash @ the app partition
#     probe-rs reset    -> run
#
# Prereqs for the flashing recipes (the build/run ones need none of these):
#   * hisi-fwpkg  — cargo install --git https://github.com/hispark-rs/hisi-fwpkg
#   * the PATCHED probe-rs FORK (upstream probe-rs does NOT know {{chip}} yet):
#       cargo install --git https://github.com/hispark-rs/probe-rs \
#           --branch add-hisilicon-ws63-bs21 probe-rs-tools
#   * its chip description YAML (HiSilicon_WS63.yaml), shipped in that fork.

# The release ELF cargo produces (cargo names the artifact after [[bin]] = crate_name).
elf := "target/riscv32imfc-unknown-none-elf/release/{{crate_name}}"
# The packaged, bootable image (0x300 HiSilicon header || body).
img := "{{crate_name}}.img"
# The single-partition vendor package (the `fwpkg` recipe / hisiflash path).
fwpkg_out := "{{crate_name}}.fwpkg"

# probe-rs fork chip name + the path to its chip-description YAML. Override on the
# CLI, e.g.  `just CHIP_DESC=~/probe-rs/HiSilicon_WS63.yaml flash`
CHIP        := {% if chip == "ws63" %}"WS63"{% else %}"{{chip | upcase}}"{% endif %}
CHIP_DESC   := "HiSilicon_WS63.yaml"
# Flash address of the `app` partition flashboot loads the image from.
APP_ADDR    := "{{app_partition_addr}}"

{% raw %}_default:
    @just --list

# Build the release firmware ELF.
build:
    cargo build --release

# Run in QEMU (the hisi-riscv-qemu fork; Ctrl-A then X to quit).
run:
    cargo run --release

# flashboot jumps unconditionally to (app partition + 0x300), so a bare ELF/bin
# does NOT boot — this 0x300 HiSilicon header is what makes the image bootable.
#
# Build + wrap the ELF in the 0x300 HiSilicon app-image header -> {{img}}.
image: build
    hisi-fwpkg image -o {{img}} {{elf}}

# Secure boot is disabled on the dev chip (efuse SEC_VERIFY_ENABLE == 0), so the
# dummy-signature image hisi-fwpkg produces is sufficient — no real signing key.
#
# Flash {{img}} to the app partition via the PATCHED probe-rs fork, then reset.
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
    -rm -f {{img}} {{fwpkg_out}}
{% endraw %}