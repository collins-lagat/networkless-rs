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
}

impl Tray {
    pub fn new() -> Self {
        Self {
            icon: None,
        }
    }
}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        APP_ID.into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
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

        match self.icon {
            Some(Icon::Unknown) => vec![UNKNOWN_ICON.clone()],
            Some(Icon::Off) => vec![OFF_ICON.clone()],
            Some(Icon::Busy) => vec![BUSY_ICON.clone()],
            Some(Icon::Disconnected) => vec![DISCONNECTED_ICON.clone()],
            Some(Icon::AirplaneMode) => vec![AIRPLANE_MODE_ICON.clone()],
            Some(Icon::Limited) => vec![LIMITED_ICON.clone()],
            Some(Icon::Vpn) => vec![VPN_ICON.clone()],
            Some(Icon::Ethernet) => vec![ETHERNET_ICON.clone()],
            Some(Icon::Wifi(0..=19)) => vec![WIFI_0_ICON.clone()],
            Some(Icon::Wifi(20..=39)) => vec![WIFI_25_ICON.clone()],
            Some(Icon::Wifi(40..=49)) => vec![WIFI_50_ICON.clone()],
            Some(Icon::Wifi(50..=79)) => vec![WIFI_75_ICON.clone()],
            Some(Icon::Wifi(80..=100)) => vec![WIFI_100_ICON.clone()],
            Some(Icon::Wifi(_)) => unreachable!(),
            None => vec![],
        }
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
        use ksni::menu::SubMenu;
        vec![
            SubMenu {
                label: "wireless".into(),
                submenu: vec![],
                ..Default::default()
            }
            .into(),
        ]
    }
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
