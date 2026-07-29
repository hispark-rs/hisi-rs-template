// WS63 Wi-Fi happy path through the public hisi-rf facade.

use core::fmt;
use core::future::Future;
use core::task::{Context, Poll, Waker};

use hisi_hal::Peripherals;
use hisi_hal::delay::Delay;
use hisi_hal::rf_power::RfPower;
use hisi_hal::time::Instant as HalInstant;
use hisi_hal::uart::{Config as UartConfig, Uart, UartClock};
use hisi_hal::wdt::Watchdog;
use hisi_panic_handler as _;
use hisi_rf::{Passphrase, ScanConfig, ScanResult, StationConfig};
use hisi_riscv_rt::entry;
use smoltcp::iface::{Config as InterfaceConfig, Interface, SocketSet, SocketStorage};
use smoltcp::socket::dhcpv4;
use smoltcp::time::Instant as NetworkInstant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpCidr};

#[path = "wifi_config.rs"]
mod wifi_config;

use wifi_config::{
    ApplicationWaitDeadline, CONNECT_OPERATION_TIMEOUT, CONNECT_WAIT_DEADLINE,
    INITIALIZE_WAIT_DEADLINE, SCAN_CAPACITY, SCAN_OPERATION_TIMEOUT, SCAN_WAIT_DEADLINE,
    WIFI_PASSPHRASE, WIFI_SSID,
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
    halt()
}

fn fail_with_configuration(uart: &Uart0<'_>, code: &[u8], action: &[u8]) -> ! {
    uart.write(b"WIFI_ERROR {\"schema\":\"hisi-rf-application-config/v1\",\"code\":\"");
    uart.write(code);
    uart.write(b"\",\"action\":\"");
    uart.write(action);
    uart.write(b"\"}\r\n");
    halt()
}

fn fail_with_rtos_start(uart: &Uart0<'_>, error: hisi_rtos::StartError) -> ! {
    let code = match error {
        hisi_rtos::StartError::AlreadyStarted => b"runtime.already-started".as_slice(),
        hisi_rtos::StartError::Driver(_) => b"runtime.driver-registration".as_slice(),
    };
    uart.write(b"WIFI_ERROR {\"schema\":\"hisi-rtos-start/v1\",\"code\":\"");
    uart.write(code);
    uart.write(b"\",\"action\":\"restart with exactly one RTOS runtime\"}\r\n");
    halt()
}

hisi_rf::ws63::declare_radio_storage!(static RADIO_STORAGE);
static RTOS_STORAGE: hisi_rtos::SchedulerStorage<15> = hisi_rtos::SchedulerStorage::new();
#[cfg_attr(target_arch = "riscv32", unsafe(link_section = ".hisi.shared-arena"))]
static RTOS_ARENA: hisi_rtos::SchedulerArena<
    { hisi_rf::ws63::SELECTED_RUNTIME_ARENA_BYTES },
> = hisi_rtos::SchedulerArena::new();

hisi_rtos::bind_interrupts!(struct RtosIrqs {
    TIMER_INT0 => hisi_rtos::ws63::TimerInterrupt;
    SOFT_INT0 => hisi_rtos::ws63::SoftwareInterrupt;
});

fn monotonic_ms() -> u64 {
    HalInstant::now().raw() / (u64::from(hisi_hal::soc::chip::TCXO_HZ) / 1_000)
}

fn network_now() -> NetworkInstant {
    NetworkInstant::from_millis(monotonic_ms().min(i64::MAX as u64) as i64)
}

fn contract_violation(_: hisi_rtos::ContractViolation) -> ! {
    panic!("hisi-rtos scheduler contract violation")
}

struct ApplicationDeadlineElapsed;

fn block_on_radio<F: Future>(
    future: F,
    deadline: ApplicationWaitDeadline,
) -> Result<F::Output, ApplicationDeadlineElapsed> {
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    let started = monotonic_ms();
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
            return Ok(output);
        }
        if monotonic_ms().wrapping_sub(started) >= deadline.as_millis() {
            return Err(ApplicationDeadlineElapsed);
        }
        // The application thread is an adopted cooperative RTOS task. Each
        // pending poll explicitly gives the radio runner and vendor workers a
        // scheduling point without importing the runtime-driver crate.
        hisi_rtos::request_reschedule();
    }
}

fn fail_with_application_deadline(uart: &Uart0<'_>, stage: &[u8]) -> ! {
    uart.write(b"WIFI_ERROR {\"schema\":\"hisi-rf-application-wait/v1\",");
    uart.write(b"\"code\":\"application.deadline\",\"stage\":\"");
    uart.write(stage);
    uart.write(b"\"}\r\n");
    halt()
}

fn halt() -> ! {
    loop {
        core::hint::spin_loop();
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
        fail_with_configuration(
            &uart,
            b"credentials.missing",
            b"set WS63_WIFI_SSID and WS63_WIFI_PASSPHRASE before building",
        );
    }

    let mut tcxo = hisi_hal::tcxo::TcxoDriver::new(p.TCXO);
    tcxo.enable();

    let installed_storage = match RADIO_STORAGE.install() {
        Ok(storage) => storage,
        Err(error) => fail_with_radio_diagnostic(&uart, error.diagnostic()),
    };
    let scheduler_storage = match RTOS_STORAGE.install(&RTOS_ARENA) {
        Ok(storage) => storage,
        Err(_) => fail_with_configuration(
            &uart,
            b"runtime.storage",
            b"declare one unclaimed scheduler storage arena for the selected profile",
        ),
    };

    let mut delay = Delay::new();
    let rf_ready = RfPower::new(p.CMU, p.CLDO_CRG).enable(p.EFUSE, &mut delay);
    let (_cldo_crg, efuse) = rf_ready.into_parts();

    let _runtime = match hisi_rtos::ws63::start(
        hisi_rtos::ws63::Config {
            radio_task_policy: hisi_rtos::RunPolicy::Cooperative,
            ..hisi_rtos::ws63::Config::default()
        },
        hisi_rtos::ws63::Resources {
            timer: p.TIMER,
            software_interrupt: p.SYS_CTL1,
            storage: scheduler_storage,
            contract_violation,
            irqs: RtosIrqs::new(),
        },
    ) {
        Ok(runtime) => runtime,
        Err(error) => fail_with_rtos_start(&uart, error),
    };

    let (control_storage, radio_arena) = installed_storage.into_init_parts();
    let resources =
        hisi_rf::ws63::Resources::<hisi_rf::ws63::SelectedProfile>::builder(efuse, radio_arena)
            .crypto(p.KM, p.SPACC, p.TRNG)
            .build();
    let controller =
        match hisi_rf::ws63::init(wifi_config::radio_config(), resources, control_storage) {
            Ok(controller) => controller,
            Err(error) => fail_with_radio_diagnostic(&uart, error.diagnostic()),
        };
    let mut wifi = match controller.start_runner() {
        Ok(wifi) => wifi,
        Err(error) => fail_with_radio_diagnostic(&uart, error.diagnostic()),
    };

    let initialized = block_on_radio(wifi.controller.initialize(), INITIALIZE_WAIT_DEADLINE)
        .unwrap_or_else(|_| fail_with_application_deadline(&uart, b"initialize"));
    if let Err(error) = initialized {
        fail_with_radio_diagnostic(&uart, error.diagnostic());
    }
    uart.write(b"WIFI_INIT_OK\r\n");

    let mut results = [ScanResult::empty(); SCAN_CAPACITY];
    let outcome = block_on_radio(
        wifi.controller
            .scan(ScanConfig::new(SCAN_OPERATION_TIMEOUT), &mut results),
        SCAN_WAIT_DEADLINE,
    )
    .unwrap_or_else(|_| fail_with_application_deadline(&uart, b"scan"))
    .unwrap_or_else(|error| fail_with_radio_diagnostic(&uart, error.diagnostic()));
    uart.write(b"WIFI_SCAN_OK\r\n");

    let selected = select_network(&results[..outcome.count], WIFI_SSID).unwrap_or_else(|| {
        fail_with_configuration(
            &uart,
            b"network.not-found",
            b"verify SSID visibility and run a new scan",
        )
    });
    let passphrase = Passphrase::try_from_ascii(WIFI_PASSPHRASE).unwrap_or_else(|| {
        fail_with_configuration(
            &uart,
            b"credentials.invalid",
            b"use an 8-63 byte WPA2 passphrase",
        )
    });
    let station = StationConfig::wpa2_personal(selected, passphrase, CONNECT_OPERATION_TIMEOUT)
        .unwrap_or_else(|| {
            fail_with_configuration(
                &uart,
                b"network.security-mismatch",
                b"select a WPA2-Personal network",
            )
        });
    let connected = block_on_radio(wifi.controller.connect(station), CONNECT_WAIT_DEADLINE)
        .unwrap_or_else(|_| fail_with_application_deadline(&uart, b"connect"));
    if let Err(error) = connected {
        fail_with_radio_diagnostic(&uart, error.diagnostic());
    }
    uart.write(b"WIFI_CONNECT_OK\r\n");

    let mac = wifi
        .device
        .station_mac_address()
        .expect("station MAC unavailable after init");
    run_smoltcp(&uart, wifi.device, mac)
}
