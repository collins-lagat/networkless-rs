use std::sync::LazyLock;

use ksni::{Icon, Tray};

use crate::{APP_ID, network::network_manager::NetworkManager, trays::get_icon_from_image_bytes};

pub struct AirplaneModeTray {
    nm: NetworkManager,
    on: bool,
}

impl AirplaneModeTray {
    pub fn new(nm: NetworkManager) -> Self {
        Self { nm, on: false }
    }

    pub async fn sync(&mut self) {
        self.on = self.nm.airplane_mode_enabled().await.unwrap_or(false);
    }
}

impl Tray for AirplaneModeTray {
    fn id(&self) -> String {
        format!("{}.AirplaneMode", APP_ID)
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let mut icon = Vec::with_capacity(1);

        static AIRPLANE_MODE_ICON: LazyLock<Icon> = LazyLock::new(|| {
            get_icon_from_image_bytes(include_bytes!("../../assets/airplane_mode.png"))
        });

        if self.on {
            icon.push(AIRPLANE_MODE_ICON.clone());
        }

        icon
    }
}
