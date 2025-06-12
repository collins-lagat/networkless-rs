use std::sync::LazyLock;

use image::GenericImageView;

use crate::APP_ID;

enum Icon {
    Unknown,
    Off,
    Busy,
    Disconnected,
    AirplaneMode,
    Limited,
    Vpn,
    Ethernet,
    Wifi(u8),
}

pub struct Tray {
    icon: Option<Icon>,
    pub wifi_state: Option<WifiState>,
    pub wired_state: Option<WiredState>,
    pub bluetooth_state: Option<BluetoothState>,
    pub vpn_state: Option<VPNState>,
    pub airplane_mode_state: Option<AirplaneModeState>,
}

impl Tray {
    pub fn new() -> Self {
        Self {
            icon: None,
            wifi_state: None,
            wired_state: None,
            bluetooth_state: None,
            vpn_state: None,
            airplane_mode_state: None,
        }
    }
}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        APP_ID.into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let mut icons = vec![];

        static UNKNOWN_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../assets/unknown.png")));

        static OFF_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../assets/wifi-off.png")));

        static BUSY_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../assets/busy.png")));

        static DISCONNECTED_ICON: LazyLock<ksni::Icon> = LazyLock::new(|| {
            get_icon_from_image_bytes(include_bytes!("../assets/disconnected.png"))
        });

        static AIRPLANE_MODE_ICON: LazyLock<ksni::Icon> = LazyLock::new(|| {
            get_icon_from_image_bytes(include_bytes!("../assets/airplane_mode.png"))
        });

        static LIMITED_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../assets/limited.png")));

        static VPN_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../assets/vpn.png")));

        static ETHERNET_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../assets/ethernet.png")));

        static WIFI_100_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../assets/wifi-100.png")));

        static WIFI_75_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../assets/wifi-75.png")));

        static WIFI_50_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../assets/wifi-50.png")));

        static WIFI_25_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../assets/wifi-25.png")));

        static WIFI_0_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../assets/wifi-0.png")));

        static BLUETOOTH_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../assets/bluetooth.png")));

        static VIRTUAL_VPN_ICON: LazyLock<ksni::Icon> = LazyLock::new(|| {
            get_icon_from_image_bytes(include_bytes!("../assets/virtual-vpn.png"))
        });

        match self.icon {
            Some(Icon::Unknown) => icons.push(UNKNOWN_ICON.clone()),
            Some(Icon::Off) => icons.push(OFF_ICON.clone()),
            Some(Icon::Busy) => icons.push(BUSY_ICON.clone()),
            Some(Icon::Disconnected) => icons.push(DISCONNECTED_ICON.clone()),
            Some(Icon::AirplaneMode) => icons.push(AIRPLANE_MODE_ICON.clone()),
            Some(Icon::Limited) => icons.push(LIMITED_ICON.clone()),
            Some(Icon::Vpn) => icons.push(VPN_ICON.clone()),
            Some(Icon::Ethernet) => icons.push(ETHERNET_ICON.clone()),
            Some(Icon::Wifi(0..=19)) => icons.push(WIFI_0_ICON.clone()),
            Some(Icon::Wifi(20..=39)) => icons.push(WIFI_25_ICON.clone()),
            Some(Icon::Wifi(40..=49)) => icons.push(WIFI_50_ICON.clone()),
            Some(Icon::Wifi(50..=79)) => icons.push(WIFI_75_ICON.clone()),
            Some(Icon::Wifi(80..=100)) => icons.push(WIFI_100_ICON.clone()),
            Some(Icon::Wifi(_)) => unreachable!(),
            None => {}
        };

        if self.bluetooth_state.is_some() {
            icons.push(BLUETOOTH_ICON.clone());
        }

        if self.vpn_state.is_some() {
            icons.push(VIRTUAL_VPN_ICON.clone());
        }

        icons
    }

    fn title(&self) -> String {
        match self.icon {
            Some(Icon::Unknown) => "Unknown".into(),
            Some(Icon::Off) => "Off".into(),
            Some(Icon::Busy) => "Busy".into(),
            Some(Icon::Disconnected) => "Disconnected".into(),
            Some(Icon::AirplaneMode) => "Airplane Mode".into(),
            Some(Icon::Limited) => "Limited".into(),
            Some(Icon::Vpn) => "VPN".into(),
            Some(Icon::Ethernet) => "Ethernet".into(),
            Some(Icon::Wifi(_)) => "Wifi".into(),
            None => "Wireless".into(),
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::{CheckmarkItem, RadioGroup, RadioItem, StandardItem, SubMenu};

        let mut menu = vec![];

        if let Some(wifi_state) = &self.wifi_state {
            let connections = wifi_state
                .connections
                .iter()
                .map(|connection| RadioItem {
                    label: connection.into(),
                    ..Default::default()
                })
                .collect::<Vec<RadioItem>>();

            menu.push(
                SubMenu {
                    label: "Wifi".into(),
                    submenu: vec![
                        CheckmarkItem {
                            label: "On".into(),
                            checked: wifi_state.on,
                            ..Default::default()
                        }
                        .into(),
                        RadioGroup {
                            selected: 0,
                            options: connections,
                            ..Default::default()
                        }
                        .into(),
                    ],
                    ..Default::default()
                }
                .into(),
            );
        }

        if let Some(wired_state) = &self.wired_state {
            menu.push(
                SubMenu {
                    label: "Wired".into(),
                    submenu: vec![
                        CheckmarkItem {
                            label: "On".into(),
                            checked: wired_state.on,
                            ..Default::default()
                        }
                        .into(),
                        StandardItem {
                            label: wired_state.speed.to_string(),
                            ..Default::default()
                        }
                        .into(),
                    ],
                    ..Default::default()
                }
                .into(),
            );
        }

        if let Some(bluetooth_state) = &self.bluetooth_state {
            let mut submenu = vec![
                CheckmarkItem {
                    label: "On".into(),
                    checked: bluetooth_state.on,
                    ..Default::default()
                }
                .into(),
            ];

            for device in bluetooth_state.devices.iter() {
                submenu.push(
                    StandardItem {
                        label: device.clone(),
                        ..Default::default()
                    }
                    .into(),
                );
            }

            menu.push(
                SubMenu {
                    label: "Bluetooth".into(),
                    submenu,
                    ..Default::default()
                }
                .into(),
            );
        }

        if let Some(airplane_mode_state) = &self.airplane_mode_state {
            menu.push(
                CheckmarkItem {
                    label: "Airplane Mode".into(),
                    checked: airplane_mode_state.on,
                    ..Default::default()
                }
                .into(),
            );
        }

        if let Some(vpn_state) = &self.vpn_state {
            menu.push(
                SubMenu {
                    label: "VPN".into(),
                    submenu: vec![
                        CheckmarkItem {
                            label: "On".into(),
                            checked: vpn_state.on,
                            ..Default::default()
                        }
                        .into(),
                        StandardItem {
                            label: vpn_state.active_connection.clone(),
                            ..Default::default()
                        }
                        .into(),
                    ],
                    ..Default::default()
                }
                .into(),
            );
        }

        menu
    }
}

pub struct WifiState {
    on: bool,
    strength: u8,
    active_connection: String,
    connections: Vec<String>,
}

pub struct WiredState {
    on: bool,
    speed: u8,
}

pub struct BluetoothState {
    on: bool,
    devices: Vec<String>,
}

pub struct VPNState {
    on: bool,
    active_connection: String,
}

pub struct AirplaneModeState {
    on: bool,
}

fn get_icon_from_image_bytes(image_bytes: &[u8]) -> ksni::Icon {
    let img = image::load_from_memory_with_format(image_bytes, image::ImageFormat::Png)
        .expect("valid image");
    let (width, height) = img.dimensions();
    let mut data = img.into_rgba8().into_vec();
    assert_eq!(data.len() % 4, 0);
    for pixel in data.chunks_exact_mut(4) {
        pixel.rotate_right(1) // rgba to argb
    }
    ksni::Icon {
        width: width as i32,
        height: height as i32,
        data,
    }
}
