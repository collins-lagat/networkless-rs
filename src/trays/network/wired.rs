use ksni::{
    MenuItem,
    menu::{CheckmarkItem, StandardItem, SubMenu},
};

use crate::network::devices::Wired;

use super::NetworkTray;

#[derive(Debug, Clone)]
pub struct WiredMenu {
    on: bool,
    wired_device: Wired,
}

impl WiredMenu {
    pub fn new(wired_device: Wired) -> Self {
        Self {
            on: false,
            wired_device,
        }
    }

    pub fn menu_item(&self) -> MenuItem<NetworkTray> {
        let mut submenu: Vec<MenuItem<NetworkTray>> = Vec::with_capacity(2);

        let toggle = CheckmarkItem::<NetworkTray> {
            label: "On".to_string(),
            checked: self.on,
            activate: Box::new(|this| this.toggle_wired()),
            ..Default::default()
        };

        submenu.push(toggle.into());

        let speed = tokio::runtime::Handle::current()
            .block_on(async { self.wired_device.speed().await.unwrap() });

        let speed_item = StandardItem {
            label: speed,
            ..Default::default()
        };

        submenu.push(speed_item.into());

        SubMenu {
            submenu,
            ..Default::default()
        }
        .into()
    }

    pub fn toggle_wired(&mut self) {}
}
