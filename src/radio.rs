// WS63 BLE/SLE starter through the public hisi-rf composition root.

use core::num::{NonZeroU32, NonZeroUsize};

use hisi_hal::Peripherals;
use hisi_hal::delay::Delay;
use hisi_hal::interrupt;
use hisi_hal::rf_power::RfPower;
use hisi_hal::software_interrupt::SoftwareInterrupt0;
use hisi_hal::time::Instant;
use hisi_hal::timer::TimerAlarm0;
use hisi_hal::uart::{Config as UartConfig, Uart, UartClock};
use hisi_hal::wdt::Watchdog;
use hisi_panic_handler as _;
use hisi_riscv_rt::entry;

hisi_rf::declare_radio_storage!(static RADIO_STORAGE);

type Uart0 = Uart<'static, hisi_hal::peripherals::Uart0<'static>>;

fn fail(uart: &Uart0, marker: &[u8]) -> ! {
    uart.write(marker);
    loop {
        core::hint::spin_loop();
    }
}

fn monotonic_ms() -> u64 {
    Instant::now().raw() / (u64::from(hisi_hal::soc::chip::TCXO_HZ) / 1_000)
}

fn contract_violation(_: hisi_rtos::ContractViolation) -> ! {
    panic!("hisi-rtos scheduler contract violation")
}

unsafe fn rtos_allocate(size: usize) -> *mut u8 {
    // SAFETY: the process-lifetime radio storage is the RTOS allocator owner.
    unsafe { hisi_rf::ws63::InstalledRadioStorage::allocate(size) }
}

unsafe fn rtos_deallocate(pointer: *mut u8) {
    // SAFETY: `pointer` came from `rtos_allocate` in this runtime instance.
    unsafe { hisi_rf::ws63::InstalledRadioStorage::deallocate(pointer) };
}

#[unsafe(no_mangle)]
extern "C" fn TIMER_INT0() {
    TimerAlarm0::clear_interrupt();
    hisi_rtos::interrupt_enter();
    hisi_rtos::on_timer_interrupt();
    hisi_rtos::interrupt_exit();
}

#[unsafe(no_mangle)]
extern "C" fn SOFT_INT0() {
    SoftwareInterrupt0::clear_interrupt();
    hisi_rtos::interrupt_enter();
    hisi_rtos::on_software_interrupt();
    hisi_rtos::interrupt_exit();
}

#[entry]
fn main() -> ! {
    let p = Peripherals::take().expect("peripherals already taken");
    let uart = Uart::new_uart0(
        p.UART0,
        UartConfig {
            clock: UartClock::Boot,
            ..UartConfig::default()
        },
    );
    Watchdog::new(p.WDT).disable();

    let report = RADIO_STORAGE.report();
    let storage = RADIO_STORAGE
        .install()
        .unwrap_or_else(|_| fail(&uart, b"RADIO_STORAGE_ERR\r\n"));

    let mut delay = Delay::new();
    let rf_ready = RfPower::new(p.CMU, p.CLDO_CRG).enable(p.EFUSE, &mut delay);
    let (_cldo_crg, efuse) = rf_ready.into_parts();

    let _timer = TimerAlarm0::new(p.TIMER);
    let _software_interrupt = SoftwareInterrupt0::new(p.SYS_CTL1);
    let runtime = hisi_rtos::start_with_port(
        hisi_rtos::PortedConfig {
            minimum_stack_size: NonZeroUsize::new(report.minimum_task_stack_bytes)
                .expect("profile stack size is non-zero"),
            radio_task_policy: hisi_rtos::RunPolicy::Cooperative,
            max_scheduler_lock_duration: NonZeroU32::new(5_000).unwrap(),
        },
        hisi_rtos::Resources {
            allocate: rtos_allocate,
            deallocate: rtos_deallocate,
            monotonic_ms,
        },
        hisi_rtos::SchedulerPort {
            max_timer_delay: NonZeroU32::new(TimerAlarm0::MAX_DELAY_MS).unwrap(),
            arm_timer: TimerAlarm0::arm_millis,
            disarm_timer: TimerAlarm0::disarm,
            pend_reschedule: SoftwareInterrupt0::pend_interrupt,
            contract_violation,
        },
    )
    .unwrap_or_else(|_| fail(&uart, b"RADIO_RTOS_ERR\r\n"));
    let main_task = runtime
        .current_task()
        .unwrap_or_else(|_| fail(&uart, b"RADIO_MAIN_TASK_ERR\r\n"));
    runtime
        .set_task_run_policy(
            main_task,
            hisi_rtos::RunPolicy::Preemptive {
                time_slice: NonZeroU32::new(5).unwrap(),
            },
        )
        .unwrap_or_else(|_| fail(&uart, b"RADIO_POLICY_ERR\r\n"));

    // SAFETY: both RTOS interrupt handlers and the scheduler port are installed.
    unsafe { interrupt::enable_global() };
    hisi_rtos::request_reschedule();

{% if radio_profile == "ble-peripheral" or radio_profile == "ble-central" or radio_profile == "ble-dual-role" -%}
    let resources = hisi_rf::ws63::Resources::new(efuse, p.KM, p.SPACC, p.PKE, p.TRNG);
{% else -%}
    let resources = hisi_rf::ws63::Resources::new(efuse, p.KM, p.SPACC, p.TRNG);
{%- endif %}
    let controller = hisi_rf::ws63::init(resources, storage)
        .unwrap_or_else(|_| fail(&uart, b"RADIO_INIT_ERR\r\n"));
    uart.write(b"RADIO_INIT_OK profile={{radio_profile}}\r\n");
    run_profile(controller.split(), &uart)
}

fn run_profile(mut parts: hisi_rf::ws63::RadioParts, uart: &Uart0) -> ! {
{% if radio_profile == "ble-peripheral" or radio_profile == "ble-dual-role" -%}
    const INTERVAL_UNITS: u16 = 160; // 100 ms in Bluetooth 0.625 ms units.
    let interval = hisi_rf::ble::AdvertisingInterval::try_from_units(INTERVAL_UNITS).unwrap();
    let config = hisi_rf::ble::AdvertisingConfig::new(
        hisi_rf::ble::AdvertisingTiming::try_new(interval, interval).unwrap(),
        hisi_rf::ble::AdvertisingChannels::ALL,
        hisi_rf::ble::AdvertisingPayload::try_from_slice(b"\x02\x01\x06\x08\x09HISI-RS")
            .unwrap(),
    );
    let command = parts
        .ble
        .try_start_advertising(config)
        .unwrap_or_else(|_| fail(uart, b"RADIO_COMMAND_BUSY\r\n"));
{% elsif radio_profile == "ble-central" -%}
    const INTERVAL_UNITS: u16 = 160; // 100 ms in Bluetooth 0.625 ms units.
    let interval = hisi_rf::ble::ScanInterval::try_from_units(INTERVAL_UNITS).unwrap();
    let config = hisi_rf::ble::ScanConfig::new(
        hisi_rf::ble::ScanTiming::try_new(interval, interval).unwrap(),
        hisi_rf::ble::ScanMode::Passive,
        false,
    );
    let command = parts
        .ble
        .try_start_scanning(config)
        .unwrap_or_else(|_| fail(uart, b"RADIO_COMMAND_BUSY\r\n"));
{% elsif radio_profile == "sle-announce" or radio_profile == "sle-ssap" -%}
    const INTERVAL_UNITS: u32 = 800; // 100 ms in SLE 125 us units.
    let interval = hisi_rf::sle::AnnounceInterval::try_from_units(INTERVAL_UNITS).unwrap();
    let config = hisi_rf::sle::AnnounceConfig::new(
        hisi_rf::sle::AnnounceTiming::try_new(interval, interval).unwrap(),
        hisi_rf::sle::AnnounceChannels::ALL,
        hisi_rf::sle::AnnouncePayload::try_from_slice(b"HISI-RS").unwrap(),
        hisi_rf::sle::AnnouncePayload::try_from_slice(b"hisi-rf").unwrap(),
    );
    let command = parts
        .sle
        .try_start_announce(config)
        .unwrap_or_else(|_| fail(uart, b"RADIO_COMMAND_BUSY\r\n"));
{% elsif radio_profile == "sle-seek" -%}
    const INTERVAL_UNITS: u16 = 800; // 100 ms in SLE 125 us units.
    let interval = hisi_rf::sle::SeekInterval::try_from_units(INTERVAL_UNITS).unwrap();
    let config = hisi_rf::sle::SeekConfig::new(
        hisi_rf::sle::SeekTiming::try_new(interval, interval).unwrap(),
        false,
    );
    let command = parts
        .sle
        .try_start_seek(config)
        .unwrap_or_else(|_| fail(uart, b"RADIO_COMMAND_BUSY\r\n"));
{%- endif %}

    loop {
        parts
            .runner
            .run_once()
            .unwrap_or_else(|_| fail(uart, b"RADIO_RUNNER_ERR\r\n"));
        while parts.runner.run_event_once() {}
{% if radio_profile == "ble-peripheral" or radio_profile == "ble-central" or radio_profile == "ble-dual-role" -%}
        if let Some(completion) = parts
            .ble
            .try_take_completion()
            .unwrap_or_else(|_| fail(uart, b"RADIO_COMPLETION_ERR\r\n"))
        {
            if completion.id() != command || completion.into_result().is_err() {
                fail(uart, b"RADIO_COMMAND_ERR\r\n");
            }
            uart.write(b"RADIO_COMMAND_OK profile={{radio_profile}}\r\n");
        }
{% else -%}
        if let Some(completion) = parts
            .sle
            .try_take_completion()
            .unwrap_or_else(|_| fail(uart, b"RADIO_COMPLETION_ERR\r\n"))
        {
            if completion.id() != command || completion.into_result().is_err() {
                fail(uart, b"RADIO_COMMAND_ERR\r\n");
            }
            uart.write(b"RADIO_COMMAND_OK profile={{radio_profile}}\r\n");
        }
{% endif -%}
        if parts.runner.dropped_events() != 0 {
            fail(uart, b"RADIO_EVENT_DROP\r\n");
        }
        core::hint::spin_loop();
    }
}
