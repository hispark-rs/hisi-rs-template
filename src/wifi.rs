// WS63 Wi-Fi happy path through the public hisi-rf facade.

use core::fmt;
use core::future::Future;
use core::num::NonZeroU32;
use core::task::{Context, Poll, Waker};

use hisi_hal::Peripherals;
use hisi_hal::delay::Delay;
use hisi_hal::interrupt;
use hisi_hal::rf_power::RfPower;
use hisi_hal::software_interrupt::SoftwareInterrupt0;
use hisi_hal::time::Instant as HalInstant;
use hisi_hal::timer::TimerAlarm0;
use hisi_hal::uart::{Config as UartConfig, Uart, UartClock};
use hisi_hal::wdt::Watchdog;
use hisi_panic_handler as _;
use hisi_rf::{
    Error as RadioError, Passphrase, RadioConfig, ScanConfig, ScanResult, StationConfig,
};
use hisi_riscv_rt::entry;
use smoltcp::iface::{Config as InterfaceConfig, Interface, SocketSet, SocketStorage};
use smoltcp::socket::dhcpv4;
use smoltcp::time::Instant as NetworkInstant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpCidr};

const EVENT_CAPACITY: usize = 8;
const SCAN_CAPACITY: usize = 16;

const WIFI_SSID: &[u8] = match option_env!("WS63_WIFI_SSID") {
    Some(value) => value.as_bytes(),
    None => b"",
};
const WIFI_PASSPHRASE: &[u8] = match option_env!("WS63_WIFI_PASSPHRASE") {
    Some(value) => value.as_bytes(),
    None => b"",
};

type Uart0<'a> = Uart<'a, hisi_hal::peripherals::Uart0<'a>>;

struct DiagnosticWriter<'a, 'd>(&'a Uart0<'d>);

impl fmt::Write for DiagnosticWriter<'_, '_> {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.0.write(value.as_bytes());
        Ok(())
    }
}

fn fail_with_radio_diagnostic(uart: &Uart0<'_>, diagnostic: hisi_rf::Diagnostic) -> ! {
    uart.write(b"WIFI_ERROR ");
    let mut writer = DiagnosticWriter(uart);
    let _ = diagnostic.write_json(&mut writer);
    uart.write(b"\r\n");
    panic!("Wi-Fi operation failed")
}

static RADIO_STORAGE: hisi_rf::ws63::Storage<hisi_rf::ws63::SelectedProfile, EVENT_CAPACITY> =
    hisi_rf::ws63::Storage::new();
hisi_rf::ws63::declare_radio_arena!(static RADIO_ARENA);

unsafe fn rtos_allocate(size: usize) -> *mut u8 {
    // SAFETY: hisi-rtos releases this allocation through `rtos_deallocate`.
    unsafe { hisi_rf::ws63::InstalledRadioArena::<hisi_rf::ws63::SelectedProfile>::allocate(size) }
}

unsafe fn rtos_deallocate(pointer: *mut u8) {
    // SAFETY: hisi-rtos only returns pointers obtained through rtos_allocate.
    unsafe {
        hisi_rf::ws63::InstalledRadioArena::<hisi_rf::ws63::SelectedProfile>::deallocate(pointer)
    };
}

fn monotonic_ms() -> u64 {
    HalInstant::now().raw() / (u64::from(hisi_hal::soc::chip::TCXO_HZ) / 1_000)
}

fn network_now() -> NetworkInstant {
    NetworkInstant::from_millis(monotonic_ms().min(i64::MAX as u64) as i64)
}

fn contract_violation(_: hisi_rtos::ContractViolation) -> ! {
    panic!("hisi-rtos scheduler contract violation")
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

fn block_on_radio<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
            return output;
        }
        // The application thread is an adopted cooperative RTOS task. Each
        // pending poll explicitly gives the radio runner and vendor workers a
        // scheduling point without importing the runtime-driver crate.
        hisi_rtos::request_reschedule();
    }
}

fn select_network<'a>(results: &'a [ScanResult], ssid: &[u8]) -> Option<&'a ScanResult> {
    results.iter().find(|result| result.ssid.as_bytes() == ssid)
}

fn write_ipv4(uart: &Uart0<'_>, address: [u8; 4]) {
    for (index, octet) in address.iter().enumerate() {
        if index != 0 {
            uart.write(b".");
        }
        let mut digits = [0_u8; 3];
        let mut value = *octet;
        let mut used = 0;
        loop {
            digits[used] = b'0' + value % 10;
            used += 1;
            value /= 10;
            if value == 0 {
                break;
            }
        }
        for digit in digits[..used].iter().rev() {
            uart.write(core::slice::from_ref(digit));
        }
    }
}

fn run_smoltcp(uart: &Uart0<'_>, mut device: hisi_rf::ws63::WifiDevice, mac: [u8; 6]) -> ! {
    let mut config = InterfaceConfig::new(HardwareAddress::Ethernet(EthernetAddress(mac)));
    config.random_seed = monotonic_ms();
    let mut interface = Interface::new(config, &mut device, network_now());
    let mut socket_storage = [SocketStorage::EMPTY; 1];
    let mut sockets = SocketSet::new(&mut socket_storage[..]);
    let dhcp = sockets.add(dhcpv4::Socket::new());

    uart.write(b"WIFI_IP_STACK_BEGIN stack=smoltcp dhcp=1\r\n");
    loop {
        let _ = interface.poll(network_now(), &mut device, &mut sockets);
        match sockets.get_mut::<dhcpv4::Socket>(dhcp).poll() {
            Some(dhcpv4::Event::Configured(config)) => {
                interface.update_ip_addrs(|addresses| {
                    addresses.clear();
                    addresses
                        .push(IpCidr::Ipv4(config.address))
                        .expect("one DHCP address fits");
                });
                if let Some(router) = config.router {
                    interface
                        .routes_mut()
                        .add_default_ipv4_route(router)
                        .expect("one default route fits");
                }
                uart.write(b"WIFI_DHCP_OK addr=");
                write_ipv4(uart, config.address.address().octets());
                uart.write(b"\r\n");
            }
            Some(dhcpv4::Event::Deconfigured) => {
                interface.update_ip_addrs(|addresses| addresses.clear());
                interface.routes_mut().remove_default_ipv4_route();
                uart.write(b"WIFI_DHCP_DOWN\r\n");
            }
            None => {}
        }
        hisi_rtos::request_reschedule();
    }
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

    if WIFI_SSID.is_empty() || WIFI_PASSPHRASE.is_empty() {
        panic!("set WS63_WIFI_SSID and WS63_WIFI_PASSPHRASE before building");
    }

    let mut tcxo = hisi_hal::tcxo::TcxoDriver::new(p.TCXO);
    tcxo.enable();

    let radio_arena = RADIO_ARENA
        .claim_for::<hisi_rf::ws63::SelectedProfile>()
        .and_then(|arena| arena.install())
        .expect("install shared RF arena");

    let mut delay = Delay::new();
    let rf_ready = RfPower::new(p.CMU, p.CLDO_CRG).enable(p.EFUSE, &mut delay);
    let (_cldo_crg, efuse) = rf_ready.into_parts();

    let _timer = TimerAlarm0::new(p.TIMER);
    let _software_interrupt = SoftwareInterrupt0::new(p.SYS_CTL1);
    let _runtime = hisi_rtos::start_with_port(
        hisi_rtos::PortedConfig {
            radio_task_policy: hisi_rtos::RunPolicy::Cooperative,
            ..hisi_rtos::PortedConfig::default()
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
    .expect("start hisi-rtos");

    // The ported scheduler completes handoffs through the software-interrupt
    // trap path, so global interrupts must be enabled before radio tasks spawn.
    unsafe { interrupt::enable_global() };

    let resources =
        hisi_rf::ws63::Resources::<hisi_rf::ws63::SelectedProfile>::builder(efuse, radio_arena)
            .crypto(p.KM, p.SPACC, p.TRNG)
            .build();
    let controller = match hisi_rf::ws63::init(RadioConfig::default(), resources, &RADIO_STORAGE) {
        Ok(controller) => controller,
        Err(error) => fail_with_radio_diagnostic(&uart, error.diagnostic()),
    };
    let mut wifi = match controller.start_runner() {
        Ok(wifi) => wifi,
        Err(error) => fail_with_radio_diagnostic(&uart, error.diagnostic()),
    };

    let result = block_on_radio(async {
        wifi.controller.initialize().await?;
        uart.write(b"WIFI_INIT_OK\r\n");

        let mut results = [ScanResult::empty(); SCAN_CAPACITY];
        let outcome = wifi
            .controller
            .scan(
                ScanConfig::try_from_timeout_ms(15_000).unwrap(),
                &mut results,
            )
            .await?;
        uart.write(b"WIFI_SCAN_OK\r\n");

        let selected =
            select_network(&results[..outcome.count], WIFI_SSID).ok_or(RadioError::Protocol)?;
        let passphrase = Passphrase::try_from_ascii(WIFI_PASSPHRASE).ok_or(RadioError::Protocol)?;
        let station = StationConfig::wpa2_personal(selected, passphrase, 60_000)
            .ok_or(RadioError::Protocol)?;
        wifi.controller.connect(station).await?;
        uart.write(b"WIFI_CONNECT_OK\r\n");
        Ok::<(), RadioError>(())
    });
    if let Err(error) = result {
        fail_with_radio_diagnostic(&uart, error.diagnostic());
    }

    let mac = hisi_rf::ws63::station_mac_address().expect("station MAC unavailable after init");
    run_smoltcp(&uart, wifi.device, mac)
}
