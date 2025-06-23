use ksni::{
    MenuItem,
    menu::{CheckmarkItem, StandardItem, SubMenu},
};

use crate::network::active_connection::ActiveConnection;

use super::NetworkTray;

#[derive(Debug, Clone)]
pub struct VpnMenu {
    on: bool,
    connection: ActiveConnection,
}

impl VpnMenu {
    pub fn new(connection: ActiveConnection) -> Self {
        Self {
            on: false,
            connection,
        }
    }

    pub fn menu_item(&self) -> MenuItem<NetworkTray> {
        let mut submenu = Vec::with_capacity(2);

        let toggle = CheckmarkItem::<NetworkTray> {
            label: "On".to_string(),
            checked: self.on,
            activate: Box::new(|this| this.toggle_vpn()),
            ..Default::default()
        };

        submenu.push(toggle.into());

        let vpn_name = tokio::runtime::Handle::current()
            .block_on(async { self.connection.id().await.unwrap() });

        let vpn_name_item = StandardItem {
            label: vpn_name,
            ..Default::default()
        };

        submenu.push(vpn_name_item.into());

        SubMenu {
            submenu,
            ..Default::default()
        }
        .into()
    }

    pub fn toggle_vpn(&mut self) {}
}
