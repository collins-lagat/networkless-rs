use std::sync::LazyLock;

use ksni::{Icon, Tray};

use crate::{APP_ID, app::App, trays::get_icon_from_image_bytes};

pub struct VpnTray {
    app: Option<App>,
}

impl VpnTray {
    pub fn new() -> Self {
        Self { app: None }
    }

    pub fn set_app(&mut self, app: App) {
        self.app = Some(app);
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

        icon.push(VPN_ICON.clone());

        icon
    }
}
