// From: https://github.com/pop-os/dbus-settings-bindings/blob/3b86984332be2c930a3536ab714b843c851fa8ca/networkmanager/src/interface/enums.rs

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NmState {
    Asleep,
    Disconnected,
    Disconnecting,
    Connecting,
    ConnectedLocal,
    ConnectedSite,
    ConnectedGlobal,
    #[default]
    Unknown,
}

impl From<u32> for NmState {
    fn from(state: u32) -> NmState {
        match state {
            10 => NmState::Asleep,
            20 => NmState::Disconnected,
            30 => NmState::Disconnecting,
            40 => NmState::Connecting,
            50 => NmState::ConnectedLocal,
            60 => NmState::ConnectedSite,
            70 => NmState::ConnectedGlobal,
            _ => NmState::Unknown,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NmConnectivityState {
    None,
    Portal,
    Loss,
    Full,
    #[default]
    Unknown,
}

impl From<u32> for NmConnectivityState {
    fn from(state: u32) -> NmConnectivityState {
        match state {
            1 => NmConnectivityState::None,
            2 => NmConnectivityState::Portal,
            3 => NmConnectivityState::Loss,
            4 => NmConnectivityState::Full,
            _ => NmConnectivityState::Unknown,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Ethernet,
    Wifi,
    Bluetooth,
    Modem,
    TunTap,
    WireGuard,
    Generic,
    Other,
    #[default]
    Unknown,
}

impl From<u32> for DeviceType {
    fn from(device_type: u32) -> DeviceType {
        match device_type {
            1 => DeviceType::Ethernet,
            2 => DeviceType::Wifi,
            5 => DeviceType::Bluetooth,
            8 => DeviceType::Modem,
            14 => DeviceType::Generic,
            16 => DeviceType::TunTap,
            29 => DeviceType::WireGuard,
            3..=32 => DeviceType::Other,
            _ => DeviceType::Unknown,
        }
    }
}
