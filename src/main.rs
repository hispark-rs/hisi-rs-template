{%- case starter -%}
{%- when "blinky" -%}
//! {{project-name}} — LED blinky on the HiSilicon {{chip}}.
//!
//! Toggles GPIO0 using the chip-neutral HAL GPIO path (the [`OutputConfig`]
//! builder + the type-erased [`Output`] driver). The busy-wait is sized for the
//! {{chip}} app core ({% if chip == "ws63" %}240 MHz{% else %}64 MHz{% endif %}); for an
//! interrupt- or async-timed delay see the `async` starter.

#![no_std]
#![no_main]

use hisi_riscv_hal::gpio::{AnyPin, OutputConfig};
use hisi_riscv_rt::entry;

/// Approximate busy-wait delay (~1 cycle/iter, calibrated for the {{chip}} CPU clock).
fn delay_ms(ms: u32) {
    for _ in 0..ms {
        for _ in 0..{% if chip == "ws63" %}240_000{% else %}64_000{% endif %} {
            core::hint::spin_loop();
        }
    }
}

#[entry]
fn main() -> ! {
    // GPIO0 as a push-pull output starting low, built via the OutputConfig builder.
    // SAFETY: GPIO0 is a valid {{chip}} pin and this example owns it exclusively.
    let mut led = unsafe { AnyPin::steal(0) }.init_output(OutputConfig::new().with_initial(false));

    loop {
        led.set_high();
        delay_ms(500);
        led.set_low();
        delay_ms(500);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
{%- when "uart_hello" -%}
//! {{project-name}} — UART hello-world on the HiSilicon {{chip}}.
//!
//! Prints a banner and a running tick counter over UART0 (115200 8N1) using the
//! HAL's chip-neutral `Uart` driver. It deliberately does NOT init the clock tree,
//! so it touches only UART0 registers — ideal for the `{{chip}}` QEMU machine,
//! whose UART model ignores the baud divisor. On QEMU the output appears on stdio
//! (`-nographic`); on silicon you would init the clocks first.

#![no_std]
#![no_main]

use hisi_riscv_hal::Peripherals;
use hisi_riscv_hal::uart::{Config, Uart};
use hisi_riscv_rt::entry;

/// Format a u32 as decimal into `buf`, returning the used slice.
fn u32_to_dec(mut n: u32, buf: &mut [u8; 10]) -> &[u8] {
    if n == 0 {
        buf[0] = b'0';
        return &buf[..1];
    }
    let mut i = buf.len();
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    &buf[i..]
}

#[entry]
fn main() -> ! {
    let p = Peripherals::take().unwrap();
    let uart = Uart::new_uart0(p.UART0, Config::default());

    uart.write(0, b"\r\nHello from {{chip}} on QEMU!\r\n");
    uart.write(0, b"UART0 @ {% if chip == "ws63" %}0x44010000{% else %}0x52081000{% endif %} is alive.\r\n");

    let mut tick: u32 = 0;
    loop {
        let mut buf = [0u8; 10];
        uart.write(0, b"tick ");
        uart.write(0, u32_to_dec(tick, &mut buf));
        uart.write(0, b"\r\n");
        tick = tick.wrapping_add(1);

        // Busy-wait between lines (~arbitrary at QEMU speed).
        for _ in 0..5_000_000 {
            core::hint::spin_loop();
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
{%- when "async" -%}
//! {{project-name}} — embassy multitask on the WS63.
//!
//! Runs `embassy-executor` (platform-riscv32, thread mode) with two async tasks
//! that `Timer::after_millis(..).await` at different rates. Time comes from
//! `hisi_riscv_hal::embassy` — the WS63 embassy-time `Driver` (now() via the TCXO
//! 64-bit counter, alarms via a TIMER channel). Proves full embassy adaptation on
//! the single-core, no-atomics WS63 (atomics via portable-atomic + critical-section).
//!
//! The HAL time-driver does not install a trap handler, so this app owns its
//! `mtvec` and routes the alarm channel's IRQ (TIMER_INT0 = 26) to
//! `hisi_riscv_hal::embassy::on_alarm_interrupt`.

#![no_std]
#![no_main]

use embassy_executor::{Executor, Spawner};
use embassy_time::Timer;
use hisi_riscv_hal::Peripherals;
use hisi_riscv_hal::interrupt;
use hisi_riscv_rt::entry;
use static_cell::StaticCell;

// ── raw UART0 output (avoids sharing a Uart handle across tasks) ──
const UART0_DATA: *mut u32 = 0x4401_0004 as *mut u32;
const UART0_FIFO: *const u32 = 0x4401_0044 as *const u32;

fn putc(b: u8) {
    unsafe {
        while core::ptr::read_volatile(UART0_FIFO) & 1 != 0 {} // TX FIFO full?
        core::ptr::write_volatile(UART0_DATA, b as u32);
    }
}
fn puts(s: &[u8]) {
    for &b in s {
        putc(b);
    }
}
fn putnum(mut n: u32) {
    if n == 0 {
        putc(b'0');
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = buf.len();
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    puts(&buf[i..]);
}

// ── alarm-channel trap (direct mode) ──
core::arch::global_asm!(
    ".section .text.atrap, \"ax\"",
    ".balign 4",
    ".global atrap",
    "atrap:",
    "    addi sp, sp, -64",
    "    sw ra,0(sp)\n sw t0,4(sp)\n sw t1,8(sp)\n sw t2,12(sp)\n sw t3,16(sp)",
    "    sw t4,20(sp)\n sw t5,24(sp)\n sw t6,28(sp)\n sw a0,32(sp)\n sw a1,36(sp)",
    "    sw a2,40(sp)\n sw a3,44(sp)\n sw a4,48(sp)\n sw a5,52(sp)\n sw a6,56(sp)\n sw a7,60(sp)",
    "    call atrap_handle",
    "    lw ra,0(sp)\n lw t0,4(sp)\n lw t1,8(sp)\n lw t2,12(sp)\n lw t3,16(sp)",
    "    lw t4,20(sp)\n lw t5,24(sp)\n lw t6,28(sp)\n lw a0,32(sp)\n lw a1,36(sp)",
    "    lw a2,40(sp)\n lw a3,44(sp)\n lw a4,48(sp)\n lw a5,52(sp)\n lw a6,56(sp)\n lw a7,60(sp)",
    "    addi sp, sp, 64",
    "    mret",
);

unsafe extern "C" {
    fn atrap();
}

#[unsafe(no_mangle)]
extern "C" fn atrap_handle() {
    let mcause: u32;
    unsafe { core::arch::asm!("csrr {0}, mcause", out(reg) mcause) };
    if (mcause & 0x8000_0000) != 0 && (mcause & 0xFFF) == 26 {
        hisi_riscv_hal::embassy::on_alarm_interrupt(); // embassy-time alarm fired
    }
}

#[embassy_executor::task]
async fn fast() {
    let mut n = 0u32;
    loop {
        Timer::after_millis(10).await;
        n += 1;
        puts(b"[fast] tick ");
        putnum(n);
        puts(b"\r\n");
    }
}

#[embassy_executor::task]
async fn slow() {
    let mut n = 0u32;
    loop {
        Timer::after_millis(25).await;
        n += 1;
        puts(b"[slow] tick ");
        putnum(n);
        puts(b"\r\n");
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[entry]
fn main() -> ! {
    let p = Peripherals::take().unwrap();
    // Start the TCXO free-running counter (the embassy-time `now()` source).
    let mut tcxo = hisi_riscv_hal::tcxo::TcxoDriver::new(p.TCXO);
    tcxo.enable();

    puts(b"\r\n{{project-name}}: embassy multitask (TCXO now() + TIMER alarm)\r\n");

    unsafe {
        core::arch::asm!("csrw mtvec, {0}", in(reg) atrap as *const () as usize); // direct mode
        interrupt::init();
        interrupt::enable_global();
    }

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner: Spawner| {
        spawner.spawn(fast().unwrap());
        spawner.spawn(slow().unwrap());
    });
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
{%- endcase -%}
