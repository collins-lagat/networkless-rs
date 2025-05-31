#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Unknown,
    Off,
    Busy,
    Disconnected,
    AirplaneMode,
    Limited,
    Vpn,
    Ethernet,
    Wifi(u8),

    Shutdown,
}

