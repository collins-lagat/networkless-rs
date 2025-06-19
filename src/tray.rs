use std::sync::LazyLock;

use image::GenericImageView;

use crate::{
    APP_ID,
    app::{Action, App},
};

#[derive(Debug, Clone)]
pub enum Icon {
    Unknown,
    Off,
    Busy,
    Disconnected,
    AirplaneMode,
    Limited,
    Tun,
    Ethernet,
    Wifi(u8),
}

#[derive(Debug, Clone)]
pub struct Tray {
    icon: Option<Icon>,
    app: Option<App>,
    pub wifi_state: Option<WifiState>,
    pub wired_state: Option<WiredState>,
    pub vpn_state: Option<VPNState>,
    pub airplane_mode_state: Option<AirplaneModeState>,
}

impl Tray {
    pub fn new() -> Self {
        Self {
            icon: None,
            app: None,
            wifi_state: None,
            wired_state: None,
            vpn_state: None,
            airplane_mode_state: None,
        }
    }

    pub fn set_app(&mut self, app: App) {
        self.app = Some(app);
    }

    pub fn set_icon(&mut self, icon: Icon) {
        self.icon = Some(icon);
    }

    pub fn set_wifi_state(&mut self, wifi_state: WifiState) {
        self.wifi_state = Some(wifi_state);
    }

    pub fn set_wired_state(&mut self, wired_state: WiredState) {
        self.wired_state = Some(wired_state);
    }

    pub fn set_vpn_state(&mut self, vpn_state: VPNState) {
        self.vpn_state = Some(vpn_state);
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
            Some(Icon::Tun) => icons.push(VPN_ICON.clone()),
            Some(Icon::Ethernet) => icons.push(ETHERNET_ICON.clone()),
            Some(Icon::Wifi(0..=19)) => icons.push(WIFI_0_ICON.clone()),
            Some(Icon::Wifi(20..=39)) => icons.push(WIFI_25_ICON.clone()),
            Some(Icon::Wifi(40..=49)) => icons.push(WIFI_50_ICON.clone()),
            Some(Icon::Wifi(50..=79)) => icons.push(WIFI_75_ICON.clone()),
            Some(Icon::Wifi(80..=100)) => icons.push(WIFI_100_ICON.clone()),
            Some(Icon::Wifi(_)) => unreachable!(),
            None => {}
        };

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
            Some(Icon::Tun) => "VPN".into(),
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
                            activate: Box::new(|this: &mut Self| {
                                if let Some(app) = this.app.as_ref() {
                                    app.send_action_blocking(Action::ToggleWifi);
                                }
                            }),
                            ..Default::default()
                        }
                        .into(),
                        RadioGroup {
                            selected: 0,
                            options: connections,
                            select: Box::new(|this: &mut Self, current| {
                                if let Some(app) = this.app.as_ref() {
                                    app.send_action_blocking(Action::ChangeAccessPoint(
                                        current.to_string(),
                                    ));
                                }
                            }),
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
                            activate: Box::new(|this: &mut Self| {
                                if let Some(app) = this.app.as_ref() {
                                    app.send_action_blocking(Action::ToggleWired);
                                }
                            }),
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

        if let Some(airplane_mode_state) = &self.airplane_mode_state {
            menu.push(
                CheckmarkItem {
                    label: "Airplane Mode".into(),
                    checked: airplane_mode_state.on,
                    activate: Box::new(|this: &mut Self| {
                        if let Some(app) = this.app.as_ref() {
                            app.send_action_blocking(Action::ToggleAirplaneMode);
                        }
                    }),
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
                            activate: Box::new(|this: &mut Self| {
                                if let Some(app) = this.app.as_ref() {
                                    app.send_action_blocking(Action::ToggleVPN);
                                }
                            }),
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

#[derive(Debug, Clone)]
pub struct WifiState {
    pub on: bool,
    pub connections: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WiredState {
    pub on: bool,
    pub speed: u8,
}

#[derive(Debug, Clone)]
pub struct VPNState {
    pub on: bool,
    pub active_connection: String,
}

#[derive(Debug, Clone)]
pub struct AirplaneModeState {
    pub on: bool,
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
