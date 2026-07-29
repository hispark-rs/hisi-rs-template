use hisi_rf::{BackendTimeout, OperationTimeout, RadioConfig, WifiConfig};

pub const SCAN_CAPACITY: usize = 16;

#[derive(Clone, Copy)]
pub struct ApplicationWaitDeadline(u64);

impl ApplicationWaitDeadline {
    pub const fn from_millis(milliseconds: u64) -> Self {
        assert!(milliseconds != 0);
        Self(milliseconds)
    }

    pub const fn as_millis(self) -> u64 {
        self.0
    }
}

pub const SCAN_OPERATION_TIMEOUT: OperationTimeout =
    OperationTimeout::try_from_millis(15_000).expect("non-zero scan operation timeout");
pub const CONNECT_OPERATION_TIMEOUT: OperationTimeout =
    OperationTimeout::try_from_millis(60_000).expect("non-zero connect operation timeout");

pub const INITIALIZE_WAIT_DEADLINE: ApplicationWaitDeadline =
    ApplicationWaitDeadline::from_millis(35_000);
pub const SCAN_WAIT_DEADLINE: ApplicationWaitDeadline =
    ApplicationWaitDeadline::from_millis(30_000);
pub const CONNECT_WAIT_DEADLINE: ApplicationWaitDeadline =
    ApplicationWaitDeadline::from_millis(90_000);

pub const WIFI_SSID: &[u8] = match option_env!("WS63_WIFI_SSID") {
    Some(value) => value.as_bytes(),
    None => b"",
};
pub const WIFI_PASSPHRASE: &[u8] = match option_env!("WS63_WIFI_PASSPHRASE") {
    Some(value) => value.as_bytes(),
    None => b"",
};

pub fn radio_config() -> RadioConfig {
    let mut config = RadioConfig::default();
    config.wifi = WifiConfig {
        initialize_timeout: BackendTimeout::try_from_millis(30_000)
            .expect("non-zero backend initialize timeout"),
        disconnect_timeout: BackendTimeout::try_from_millis(10_000)
            .expect("non-zero backend disconnect timeout"),
    };
    config
}
