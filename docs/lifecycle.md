# hisi-rs-template lifecycle

`hisi-rs-template` is a project template, not a crates.io package. Its release
unit is a Git tag and GitHub Release in the template repository.

## Versioning

- Template tags use the ecosystem line they target, for example
  `v0.6.0-alpha.1`.
- The generated application's `Cargo.toml` keeps `version = "0.1.0"` because it
  belongs to the new user project.
- A template release is cut after the referenced crates are already available on
  crates.io.

## Compatibility matrix

| Template tag | HAL | Runtime | WS63 PAC | BS2X PAC | Image tool |
| --- | --- | --- | --- | --- | --- |
| `v0.6.0-alpha.1` | `0.6.0-alpha.1` | `0.5.1` | `0.2.1` | `0.1.2` | `hisi-fwpkg-cli 0.3.0` |

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

