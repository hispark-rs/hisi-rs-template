# Changelog

All notable changes to `hisi-rs-template` are documented here.

This template is versioned by Git tags and GitHub Releases. The generated
application's `Cargo.toml` keeps `version = "0.1.0"` because that is the user's
application version, not the template version.

## [Unreleased]

## [0.7.0-alpha.19] - 2026-07-30

### Changed

- Updated the WS63 Wi-Fi starter to `hisi-rf 0.1.0-alpha.67` and
  `hisi-rtos 0.1.0-alpha.16`.
- Generated Wi-Fi projects now declare caller-owned scheduler stack storage
  and pass its installed capability to the native WS63 RTOS facade. Application
  allocator callbacks are no longer part of the happy path.

## [0.7.0-alpha.18] - 2026-07-30

### Changed

- The WS63 Wi-Fi starter now reports recoverable configuration, resource,
  runtime-start, operation, and application-deadline failures as versioned,
  actionable diagnostics before halting. Panic remains reserved for broken
  static or unsafe contracts.
- Updated to `hisi-hal 0.7.0-alpha.6` and `hisi-rtos 0.1.0-alpha.15`.
  Generated Wi-Fi projects use the native `hisi_rtos::ws63::start` facade and
  no longer define their own TIMER_INT0/SOFT_INT0 handlers, scheduler port, or
  global-interrupt startup sequence.

## [0.7.0-alpha.17] - 2026-07-29

### Changed

- Updated the WS63 Wi-Fi starter to `hisi-rf 0.1.0-alpha.57`. The generated
  application keeps all Wi-Fi timeouts, capacities, credentials, and
  `RadioConfig` construction in `src/wifi_config.rs`; it does not duplicate
  those values in the runner or application entry point.

## [0.7.0-alpha.16] - 2026-07-29

### Changed

- Updated the WS63 Wi-Fi starter to `hisi-rf 0.1.0-alpha.55`.
- The generated smoltcp runner now obtains the station MAC from its initialized
  `WifiDevice`; the generated project no longer depends on process-global
  netif state.
- Updated the resource-report contract for the per-radio L2 capability state
  added by `hisi-rf 0.1.0-alpha.55`.

## [0.7.0-alpha.15] - 2026-07-29

### Changed

- Updated the WS63 Wi-Fi starter to `hisi-rf 0.1.0-alpha.54`.
- Moved the generated application's credentials, scan capacity, typed
  operation/backend timeouts, and application wait deadlines into
  `src/wifi_config.rs`, keeping policy visible and separate from the radio
  workflow.
- Added bounded outer wait deadlines for initialize, scan, and connect. A
  dropped operation future now enters the facade's cancellation path, while
  application, operation, and backend timeout diagnostics remain distinct.
- Updated the generated resource-report contract for the cancellation channel
  added to the fixed eight-event profile.

## [0.7.0-alpha.14] - 2026-07-29

### Changed

- Updated the WS63 Wi-Fi starter to `hisi-rf 0.1.0-alpha.53`. Event capacity
  remains part of the selected profile's machine-readable resource report, but
  no longer appears in application control types or storage declarations.

## [0.7.0-alpha.13] - 2026-07-29

### Changed

- Updated the WS63 Wi-Fi starter to `hisi-rf 0.1.0-alpha.52` and replaced its
  separate control-storage and shared-arena statics with one
  `declare_radio_storage!` composition and pre-RTOS admission step.
- Updated generated resource-report tooling and CI to
  `hisi-rf-resource-report/v6`, including the control, composition-handle, and
  shared-arena ownership split measured for the WS63 RV32 target rather than
  the host running the report helper.

## [0.7.0-alpha.12] - 2026-07-29

### Changed

- Updated the WS63 Wi-Fi starter to `hisi-rf 0.1.0-alpha.49` and moved
  profile-specific hardware ownership into the starter's resource builder.
  The default WPA2 project no longer consumes an unused PKE token.

## [0.7.0-alpha.11] - 2026-07-29

### Changed

- Updated the WS63 Wi-Fi starter to `hisi-rf 0.1.0-alpha.48` and
  `hisi-rtos 0.1.0-alpha.14`, and fixed release builds to one codegen unit so
  generated projects retain the validated WS63 size/link layout independently
  of a parent workspace.

### Added

- Tag pushes now run the complete generated-project and cross-platform resource
  report matrices before creating a GitHub Release.

## [0.7.0-alpha.10] - 2026-07-28

### Changed

- Updated the WS63 Wi-Fi starter to `hisi-rf 0.1.0-alpha.42`, carrying the
  incremental local-continuation fix through the public facade and keeping the
  caller-owned arena composition contract covered by the generated project.

## [0.7.0-alpha.9] - 2026-07-28

### Added

- The WS63 Wi-Fi starter now emits a deterministic
  `*.resource.json` alongside its image and FlashPlan. A host-only helper reads
  the selected profile through the public `hisi-rf` facade and uses the same
  bounded event capacity as the firmware.

### Changed

- Updated the WS63 Wi-Fi starter to `hisi-rf 0.1.0-alpha.41`.

### Fixed

- The host-only resource-report helper now installs the standard host
  `critical-section` implementation, so its public-facade dependency graph
  links on native Windows as well as macOS and Linux.

## [0.7.0-alpha.8] - 2026-07-28

### Changed

- Updated the WS63 Wi-Fi starter to `hisi-rf 0.1.0-alpha.38`, carrying
  actionable `hisi-rf-error/v2` diagnostics for caller-owned RF arena
  admission failures.

## [0.7.0-alpha.7] - 2026-07-28

### Changed

- Updated generated projects to `hisi-riscv-rt 0.5.7`.
- Updated the WS63 Wi-Fi starter to `hisi-rf 0.1.0-alpha.37` and
  `hisi-rtos 0.1.0-alpha.13`. The application now declares and installs the
  profile-selected caller-owned RF arena through the public facade before
  starting the runtime.
- Enabled the fixed 32 KiB radio main-stack linker profile for Wi-Fi firmware
  and removed the application's obsolete direct `hisi-alloc` dependency.

## [0.7.0-alpha.6] - 2026-07-23

### Changed

- Updated the WS63 Wi-Fi starter to `hisi-rf 0.1.0-alpha.17` and
  `hisi-alloc 0.1.0-alpha.2`.
- Wi-Fi initialization, runner startup, and control-plane failures now emit the
  stable `hisi-rf-error/v2` JSON diagnostic before entering the panic handler.

## [0.7.0-alpha.5] - 2026-07-23

### Changed

- Updated the WS63 Wi-Fi starter to `hisi-rf 0.1.0-alpha.16`, including
  source-aware diagnostics and checked aligned RF heap allocations.

## [0.7.0-alpha.4] - 2026-07-23

### Added

- Added a WS63 `wifi` starter using only the public `hisi-rf` facade for radio
  composition. It starts the caller-owned runner, performs async
  initialize/scan/WPA2 connect, and hands the L2 device to smoltcp DHCP.

### Fixed

- Removed the generated `python3` dependency from `just flash`; the download
  address now uses the same explicit app address passed into `hisi-fwpkg plan`.

## [0.7.0-alpha.3] - 2026-07-17

### Changed

- Generated projects now use `hisi-hal 0.7.0-alpha.3`, `hisi-riscv-rt 0.5.5`,
  and `ws63-pac 0.4.0`, preserving the single-PAC dependency contract after the
  audited WS63 register-model release.

## [0.7.0-alpha.2] - 2026-07-14

### Changed

- Generated projects now use `hisi-hal 0.7.0-alpha.2`, `hisi-riscv-rt 0.5.4`,
  and `ws63-pac 0.3.0` so every WS63 firmware resolves one PAC major version.

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
