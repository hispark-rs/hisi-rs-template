// Pre-generation guard.
//
// The `async` (embassy) starter relies on `hisi_riscv_hal::embassy`, which is
// gated `#[cfg(all(feature = "chip-ws63", feature = "embassy"))]` in the HAL —
// the embassy-time driver is WS63-only today (the BS2X timer IRQ differs and the
// driver is not yet wired for it). Reject the combination early with a clear
// message instead of letting it fail later with a confusing build error.
let chip = variable::get("chip");
let starter = variable::get("starter");

if starter == "async" && chip != "ws63" {
    abort("The `async` (embassy) starter is currently WS63-only. Choose `chip = ws63`, or pick the `blinky` / `uart_hello` starter for " + chip + ".");
}
