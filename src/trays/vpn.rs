use std::sync::LazyLock;

use ksni::{Icon, Tray};

use crate::{APP_ID, trays::get_icon_from_image_bytes};

pub struct VpnTray {
    on: bool,
}

impl VpnTray {
    pub fn new() -> Self {
        Self { on: false }
    }

    pub async fn sync(&mut self) {
        self.on = false;
    }
}

impl Tray for VpnTray {
    fn id(&self) -> String {
        format!("{}.Vpn", APP_ID)
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        let mut icon = Vec::with_capacity(1);

        static VPN_ICON: LazyLock<Icon> =
            LazyLock::new(|| get_icon_from_image_bytes(include_bytes!("../../assets/vpn.png")));

        if self.on {
            icon.push(VPN_ICON.clone());
        }

        icon
    }
}
