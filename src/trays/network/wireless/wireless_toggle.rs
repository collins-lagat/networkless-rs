use ksni::menu::CheckmarkItem;

use crate::network::network_manager::NetworkManager;

use super::NetworkTray;

pub struct WirelessToggle {
    on: bool,
    nm: NetworkManager,
}

impl WirelessToggle {
    pub fn new(nm: NetworkManager) -> Self {
        Self { on: false, nm }
    }

    pub async fn sync(&mut self) {
        self.on = self.nm.wifi_enabled().await.unwrap_or(false);
    }

    pub fn menu_item(&self) -> CheckmarkItem<NetworkTray> {
        CheckmarkItem {
            label: "On".to_string(),
            checked: self.on,
            activate: Box::new(|this| this.toggle_wireless()),
            ..Default::default()
        }
    }
}
