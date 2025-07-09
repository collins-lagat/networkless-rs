use std::sync::LazyLock;

use ksni::MenuItem;

use crate::{
    APP_ID,
    app::{Action, App},
    trays::get_icon_from_image_bytes,
};

#[derive(Debug, Clone)]
pub enum Icon {
    Unknown,
    Off,
    Busy,
    Disconnected,
    Limited,
    Tun,
    Ethernet,
    Wifi(u8),
}

#[derive(Debug, Clone)]
pub struct WifiState {
    pub on: bool,
    pub available_connections: Vec<String>,
    pub known_connections: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WiredState {
    pub on: bool,
}

#[derive(Debug, Clone)]
pub struct VPNConnection {
    pub name: String,
    pub on: bool,
}

#[derive(Debug, Clone)]
pub struct VPNState {
    pub connections: Vec<VPNConnection>,
}

#[derive(Debug, Clone)]
pub struct AirplaneModeState {
    pub on: bool,
}

#[derive(Debug, Clone)]
pub struct NetworkTray {
    app: App,
    icon: Option<Icon>,
    pub wifi_state: Option<WifiState>,
    pub wired_state: Option<WiredState>,
    pub vpn_state: Option<VPNState>,
    pub airplane_mode_state: Option<AirplaneModeState>,
}

impl NetworkTray {
    pub fn new(app: App) -> Self {
        Self {
            icon: None,
            app,
            wifi_state: None,
            wired_state: None,
            vpn_state: None,
            airplane_mode_state: None,
        }
    }

    pub fn set_icon(&mut self, icon: Icon) {
        self.icon = Some(icon);
    }

    pub fn set_wifi_state(&mut self, wifi_state: Option<WifiState>) {
        self.wifi_state = wifi_state;
    }

    pub fn set_wired_state(&mut self, wired_state: Option<WiredState>) {
        self.wired_state = wired_state;
    }

    pub fn set_airplane_mode_state(&mut self, airplane_mode_state: Option<AirplaneModeState>) {
        self.airplane_mode_state = airplane_mode_state;
    }

    pub fn set_vpn_state(&mut self, vpn_state: Option<VPNState>) {
        self.vpn_state = vpn_state;
    }
}

impl ksni::Tray for NetworkTray {
    fn id(&self) -> String {
        APP_ID.into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let mut icons = Vec::with_capacity(1);

        static UNKNOWN_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../../assets/unknown.png")));

        static OFF_ICON: LazyLock<ksni::Icon> = LazyLock::new(|| {
            get_icon_from_image_bytes(include_bytes!("../../assets/wifi-off.png"))
        });

        static BUSY_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../../assets/busy.png")));

        static DISCONNECTED_ICON: LazyLock<ksni::Icon> = LazyLock::new(|| {
            get_icon_from_image_bytes(include_bytes!("../../assets/disconnected.png"))
        });

        static LIMITED_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../../assets/limited.png")));

        static VPN_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../../assets/vpn.png")));

        static ETHERNET_ICON: LazyLock<ksni::Icon> = LazyLock::new(|| {
            get_icon_from_image_bytes(include_bytes!("../../assets/ethernet.png"))
        });

        static WIFI_100_ICON: LazyLock<ksni::Icon> = LazyLock::new(|| {
            get_icon_from_image_bytes(include_bytes!("../../assets/wifi-100.png"))
        });

        static WIFI_75_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../../assets/wifi-75.png")));

        static WIFI_50_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../../assets/wifi-50.png")));

        static WIFI_25_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../../assets/wifi-25.png")));

        static WIFI_0_ICON: LazyLock<ksni::Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../../assets/wifi-0.png")));

        match self.icon {
            Some(Icon::Unknown) => icons.push(UNKNOWN_ICON.clone()),
            Some(Icon::Off) => icons.push(OFF_ICON.clone()),
            Some(Icon::Busy) => icons.push(BUSY_ICON.clone()),
            Some(Icon::Disconnected) => icons.push(DISCONNECTED_ICON.clone()),
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

        icons
    }

    fn title(&self) -> String {
        match self.icon {
            Some(Icon::Unknown) => "Unknown".into(),
            Some(Icon::Off) => "Off".into(),
            Some(Icon::Busy) => "Busy".into(),
            Some(Icon::Disconnected) => "Disconnected".into(),
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
            let options = wifi_state
                .known_connections
                .iter()
                .map(|connection| RadioItem {
                    label: connection.into(),
                    ..Default::default()
                })
                .collect::<Vec<RadioItem>>();

            let available_connections_label = match wifi_state.available_connections.len() {
                0 => "No Networks Available",
                _ => "Available Networks",
            };

            let wifi_on_state = wifi_state.on;
            let mut submenu = vec![
                CheckmarkItem {
                    label: "On".into(),
                    checked: wifi_state.on,
                    activate: Box::new(move |this: &mut Self| {
                        this.app
                            .send_action_blocking(Action::ToggleWifi(!wifi_on_state));
                    }),
                    ..Default::default()
                }
                .into(),
                MenuItem::Separator,
                RadioGroup {
                    selected: 0,
                    options,
                    select: Box::new(|this: &mut Self, current| {
                        this.app
                            .send_action_blocking(Action::ChangeAccessPoint(current.to_string()));
                    }),
                }
                .into(),
                MenuItem::Separator,
                StandardItem {
                    label: available_connections_label.into(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            ];

            let label = match wifi_state.known_connections.first() {
                Some(ssid) => format!("WiFi: {}", ssid),
                None => "WiFi".into(),
            };

            let mut available_connections = wifi_state
                .available_connections
                .iter()
                .map(|connection| {
                    StandardItem {
                        label: connection.into(),
                        enabled: false,
                        ..Default::default()
                    }
                    .into()
                })
                .collect::<Vec<MenuItem<Self>>>();

            submenu.append(&mut available_connections);

            menu.push(
                SubMenu {
                    label,
                    submenu,
                    ..Default::default()
                }
                .into(),
            );
        }

        if let Some(wired_state) = &self.wired_state {
            let wired_on_state = wired_state.on;
            menu.push(
                CheckmarkItem {
                    label: "Wired".into(),
                    checked: wired_state.on,
                    activate: Box::new(move |this: &mut Self| {
                        this.app
                            .send_action_blocking(Action::ToggleWired(!wired_on_state));
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }

        if let Some(airplane_mode_state) = &self.airplane_mode_state {
            let airplane_mode_on_state = airplane_mode_state.on;
            menu.push(
                CheckmarkItem {
                    label: "Airplane Mode".into(),
                    checked: airplane_mode_state.on,
                    activate: Box::new(move |this: &mut Self| {
                        this.app.send_action_blocking(Action::ToggleAirplaneMode(
                            !airplane_mode_on_state,
                        ));
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }

        if let Some(vpn_state) = &self.vpn_state {
            let connections = vpn_state
                .connections
                .iter()
                .map(|connection| {
                    let vpn_name = connection.name.clone();
                    CheckmarkItem {
                        label: connection.name.clone(),
                        checked: connection.on,
                        activate: Box::new(move |this: &mut Self| {
                            this.app
                                .send_action_blocking(Action::ToggleVPN(vpn_name.clone()));
                        }),
                        ..Default::default()
                    }
                    .into()
                })
                .collect::<Vec<MenuItem<Self>>>();
            menu.push(
                SubMenu {
                    label: "VPN".into(),
                    submenu: connections,
                    ..Default::default()
                }
                .into(),
            );
        }

        menu
    }
}
