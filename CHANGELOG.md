# Changelog

All notable changes to `hisi-rs-template` are documented here.

This template is versioned by Git tags and GitHub Releases. The generated
application's `Cargo.toml` keeps `version = "0.1.0"` because that is the user's
application version, not the template version.

## [Unreleased]

## [0.7.0-alpha.1] - 2026-07-13

### Changed

- Generated projects now depend on the renamed `hisi-hal 0.7.0-alpha.1`
  package and import it as `hisi_hal`. The retired `hisi-riscv-hal 0.6.x`
  line remains available for existing applications but is no longer emitted by
  the template.

## [0.6.0] - 2026-07-12

### Changed

- Generated WS63 and BS2X projects now use the published
  `hisi-riscv-hal 0.6.0` stable release with `hisi-riscv-rt 0.5.3`.

## [0.6.0-rc.3] - 2026-07-12

### Changed

- Generated projects now depend on `hisi-riscv-rt 0.5.3`, which publishes the
  WS63 RF startup memory/ROM-patch support while keeping WS63-only relocation
  symbols out of linked BS2X images.

## [0.6.0-rc.2] - 2026-07-12

### Fixed

- Pin `hisi-fwpkg-cli 0.3.2`, whose FlashPlan includes linker-aligned erased
  tails and avoids double-wrapping an already-headered application image.

## [0.6.0-rc.1] - 2026-07-12

### Changed

- Generated projects now depend on `hisi-riscv-hal 0.6.0-rc.1` and
  `ws63-pac 0.2.2`, so the template exercises the frozen HAL stable surface and
  the PAC definitions required by the standalone HAL release.

## [0.6.0-alpha.1] - 2026-07-09

### Added
- Added a standalone lifecycle guide for template versioning, compatibility, and
  release integration.
- Added CI that generates representative WS63/BS2X projects and validates their
  build/image contracts.

### Changed
- Generated projects now depend on the published `0.6.0-alpha.1` HAL line,
  `hisi-riscv-rt 0.5.1`, `ws63-pac 0.2.1`, and `bs2x-pac 0.1.2`.
- Hardware packaging documentation now installs `hisi-fwpkg-cli 0.3.0` from
  crates.io instead of using a git install.
