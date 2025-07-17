// From: https://github.com/pop-os/dbus-settings-bindings/blob/3b86984332be2c930a3536ab714b843c851fa8ca/networkmanager/src/interface/enums.rs

use bitflags::bitflags;

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

// Values from https://github.com/travier/nmstate/blob/47c6092388784bf0edb86fb05c4c9bed891f9fdc/rust/src/libnm_dbus/device.rs#L418
impl From<String> for DeviceType {
    fn from(device_type: String) -> Self {
        match &device_type[..] {
            "802-3-ethernet" => DeviceType::Ethernet,
            "802-11-wireless" => DeviceType::Wifi,
            "bluetooth" => DeviceType::Bluetooth,
            "modem" => DeviceType::Modem,
            "generic" => DeviceType::Generic,
            "tun" => DeviceType::TunTap,
            "wireguard" => DeviceType::WireGuard,
            _ => DeviceType::Unknown,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    Unmanaged,
    Unavailable,
    Disconnected,
    Prepare,
    Config,
    NeedAuth,
    IpConfig,
    IpCheck,
    Secondaries,
    Activated,
    Deactivating,
    Failed,
    #[default]
    Unknown,
}

impl From<u32> for DeviceState {
    fn from(device_state: u32) -> Self {
        match device_state {
            10 => DeviceState::Unmanaged,
            20 => DeviceState::Unavailable,
            30 => DeviceState::Disconnected,
            40 => DeviceState::Prepare,
            50 => DeviceState::Config,
            60 => DeviceState::NeedAuth,
            70 => DeviceState::IpConfig,
            80 => DeviceState::IpCheck,
            90 => DeviceState::Secondaries,
            100 => DeviceState::Activated,
            110 => DeviceState::Deactivating,
            120 => DeviceState::Failed,
            _ => DeviceState::Unknown,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveConnectionState {
    #[default]
    Unknown,
    Activating,
    Activated,
    Deactivating,
    Deactivated,
}

impl From<u32> for ActiveConnectionState {
    fn from(device_state: u32) -> Self {
        match device_state {
            1 => ActiveConnectionState::Activating,
            2 => ActiveConnectionState::Activated,
            3 => ActiveConnectionState::Deactivating,
            4 => ActiveConnectionState::Deactivated,
            _ => ActiveConnectionState::Unknown,
        }
    }
}

bitflags! {
    pub struct ApFlags: u32 {
        const NONE = 0x0;
        const PRIVACY = 0x1;
        const WPS = 0x2;
        const WPS_PBC = 0x4;
        const WPS_PIN = 0x8;
    }
}

bitflags! {
    pub struct ApSecurityFlags: u32 {
        const NONE = 0x0;
        const WEP40 = 0x1;
        const WEP104 = 0x2;
        const TKIP = 0x4;
        const CCMP = 0x8;
        const GROUP_WEP40 = 0x10;
        const GROUP_WEP104 = 0x20;
        const GROUP_TKIP = 0x40;
        const GROUP_CCMP = 0x80;
        const KEY_MGMTPSK = 0x100;
        const KEY_MGMT_802_1X = 0x200;
        const KEY_MGMT_SAE = 0x400;
        const KEY_MGMT_OWE = 0x800;
        const KEY_MGMT_OWE_TM = 0x1000;
        const KEY_MGMT_EAP_SUITE_B_192 = 0x2000;
    }
}
