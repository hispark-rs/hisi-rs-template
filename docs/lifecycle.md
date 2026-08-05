# hisi-rs-template lifecycle

`hisi-rs-template` is a project template, not a crates.io package. Its release
unit is a Git tag and GitHub Release in the template repository.

## Versioning

- Template tags use the ecosystem line they target, for example
  `v0.7.0-alpha.3`.
- The generated application's `Cargo.toml` keeps `version = "0.1.0"` because it
  belongs to the new user project.
- A template release is cut after the referenced crates are already available on
  crates.io.

## Compatibility matrix

| Template tag | HAL package | HAL version | Runtime | WS63 PAC | BS2X PAC | RF facade | Image tool |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `Unreleased` | `hisi-hal` | `0.7.0-alpha.6` | `0.5.7` + `hisi-rtos 0.1.0-alpha.23` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.83` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.28` | `hisi-hal` | `0.7.0-alpha.6` | `0.5.7` + `hisi-rtos 0.1.0-alpha.23` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.83` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.27` | `hisi-hal` | `0.7.0-alpha.6` | `0.5.7` + `hisi-rtos 0.1.0-alpha.18` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.75` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.26` | `hisi-hal` | `0.7.0-alpha.6` | `0.5.7` + `hisi-rtos 0.1.0-alpha.18` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.75` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.25` | `hisi-hal` | `0.7.0-alpha.6` | `0.5.7` + `hisi-rtos 0.1.0-alpha.18` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.74` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.24` | `hisi-hal` | `0.7.0-alpha.6` | `0.5.7` + `hisi-rtos 0.1.0-alpha.18` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.73` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.23` | `hisi-hal` | `0.7.0-alpha.6` | `0.5.7` + `hisi-rtos 0.1.0-alpha.17` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.71` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.22` | `hisi-hal` | `0.7.0-alpha.6` | `0.5.7` + `hisi-rtos 0.1.0-alpha.17` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.70` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.21` | `hisi-hal` | `0.7.0-alpha.6` | `0.5.7` + `hisi-rtos 0.1.0-alpha.17` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.69` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.20` | `hisi-hal` | `0.7.0-alpha.6` | `0.5.7` + `hisi-rtos 0.1.0-alpha.17` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.68` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.19` | `hisi-hal` | `0.7.0-alpha.6` | `0.5.7` + `hisi-rtos 0.1.0-alpha.16` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.67` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.18` | `hisi-hal` | `0.7.0-alpha.6` | `0.5.7` + `hisi-rtos 0.1.0-alpha.15` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.57` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.17` | `hisi-hal` | `0.7.0-alpha.3` | `0.5.7` + `hisi-rtos 0.1.0-alpha.14` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.57` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.16` | `hisi-hal` | `0.7.0-alpha.3` | `0.5.7` + `hisi-rtos 0.1.0-alpha.14` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.55` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.15` | `hisi-hal` | `0.7.0-alpha.3` | `0.5.7` + `hisi-rtos 0.1.0-alpha.14` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.54` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.14` | `hisi-hal` | `0.7.0-alpha.3` | `0.5.7` + `hisi-rtos 0.1.0-alpha.14` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.53` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.13` | `hisi-hal` | `0.7.0-alpha.3` | `0.5.7` + `hisi-rtos 0.1.0-alpha.14` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.52` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.12` | `hisi-hal` | `0.7.0-alpha.3` | `0.5.7` + `hisi-rtos 0.1.0-alpha.14` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.49` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.11` | `hisi-hal` | `0.7.0-alpha.3` | `0.5.7` + `hisi-rtos 0.1.0-alpha.14` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.48` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.10` | `hisi-hal` | `0.7.0-alpha.3` | `0.5.7` + `hisi-rtos 0.1.0-alpha.13` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.42` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.9` | `hisi-hal` | `0.7.0-alpha.3` | `0.5.7` + `hisi-rtos 0.1.0-alpha.13` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.41` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.8` | `hisi-hal` | `0.7.0-alpha.3` | `0.5.7` + `hisi-rtos 0.1.0-alpha.13` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.38` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.7` | `hisi-hal` | `0.7.0-alpha.3` | `0.5.7` + `hisi-rtos 0.1.0-alpha.13` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.37` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.6` | `hisi-hal` | `0.7.0-alpha.3` | `0.5.5` + `hisi-rtos 0.1.0-alpha.10` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.17` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.5` | `hisi-hal` | `0.7.0-alpha.3` | `0.5.5` + `hisi-rtos 0.1.0-alpha.10` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.16` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.4` | `hisi-hal` | `0.7.0-alpha.3` | `0.5.5` + `hisi-rtos 0.1.0-alpha.10` | `0.4.0` | `0.1.2` | `hisi-rf 0.1.0-alpha.14` | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.3` | `hisi-hal` | `0.7.0-alpha.3` | `0.5.5` | `0.4.0` | `0.1.2` | - | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.2` | `hisi-hal` | `0.7.0-alpha.2` | `0.5.4` | `0.3.0` | `0.1.2` | - | `hisi-fwpkg-cli 0.3.2` |
| `v0.7.0-alpha.1` | `hisi-hal` | `0.7.0-alpha.1` | `0.5.3` | `0.2.2` | `0.1.2` | - | `hisi-fwpkg-cli 0.3.2` |
| `v0.6.0` | `hisi-riscv-hal` | `0.6.0` | `0.5.3` | `0.2.2` | `0.1.2` | - | `hisi-fwpkg-cli 0.3.2` |
| `v0.6.0-rc.3` | `hisi-riscv-hal` | `0.6.0-rc.1` | `0.5.3` | `0.2.2` | `0.1.2` | - | `hisi-fwpkg-cli 0.3.2` |
| `v0.6.0-rc.2` | `hisi-riscv-hal` | `0.6.0-rc.1` | `0.5.1` | `0.2.2` | `0.1.2` | - | `hisi-fwpkg-cli 0.3.2` |
| `v0.6.0-rc.1` | `hisi-riscv-hal` | `0.6.0-rc.1` | `0.5.1` | `0.2.2` | `0.1.2` | - | `hisi-fwpkg-cli 0.3.0` |
| `v0.6.0-alpha.1` | `hisi-riscv-hal` | `0.6.0-alpha.1` | `0.5.1` | `0.2.1` | `0.1.2` | - | `hisi-fwpkg-cli 0.3.0` |

## Release checklist

1. Publish dependency crates first: PAC, runtime, HAL, and `hisi-fwpkg-cli`.
2. Update this template's generated dependency versions and the compatibility
   matrix.
3. Run template CI locally or in GitHub Actions; it must generate the supported
   starter projects and run their build/image contracts.
4. Tag the template repository with `vX.Y.Z[-pre.N]`.
5. Update the parent `hisi-riscv-rs` submodule pointer and docs happy-path
   checks.

## Integration rule

The parent repo pins this repository as a submodule so docs and smoke scripts can
test the same template users generate. Do not copy template commands into parent
docs without also adding them to the parent happy-path smoke script.
