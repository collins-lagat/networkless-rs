use ksni::{MenuItem, menu::CheckmarkItem};

use crate::network::network_manager::NetworkManager;

use super::NetworkTray;

#[derive(Debug, Clone)]
pub struct AirplaneModeMenu {
    on: bool,
    nm: NetworkManager,
}

impl AirplaneModeMenu {
    pub fn new(nm: NetworkManager) -> Self {
        Self { on: false, nm }
    }

    pub async fn sync(&mut self) {
        self.on = self.nm.airplane_mode_enabled().await.unwrap_or(false);
    }

    pub fn menu_item(&self) -> MenuItem<NetworkTray> {
        CheckmarkItem {
            label: "On".to_string(),
            checked: self.on,
            ..Default::default()
        }
        .into()
    }
}
