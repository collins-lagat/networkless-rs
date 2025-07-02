use std::sync::LazyLock;

use ksni::{Icon, MenuItem, Tray, menu::CheckmarkItem};

use crate::{
    APP_ID,
    app::{Action, App},
    trays::get_icon_from_image_bytes,
};

use super::VPNState;

pub struct VpnTray {
    app: App,
    state: Option<VPNState>,
}

impl VpnTray {
    pub fn new(app: App) -> Self {
        Self { app, state: None }
    }

    pub fn set_state(&mut self, state: Option<VPNState>) {
        self.state = state;
    }
}

impl Tray for VpnTray {
    fn id(&self) -> String {
        format!("{}.Vpn", APP_ID)
    }

    fn title(&self) -> String {
        if let Some(state) = &self.state {
            let connection = state
                .connections
                .iter()
                .find(|c| c.on)
                .map(|c| c.name.clone());
            if let Some(connection) = connection {
                return format!("VPN: {}", connection);
            }
            return "VPN".into();
        }
        "VPN".into()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        let mut icon = Vec::with_capacity(1);

        static VPN_ICON: LazyLock<Icon> = LazyLock::new(|| {
            get_icon_from_image_bytes(include_bytes!("../../assets/virtual-vpn.png"))
        });

        if let Some(state) = &self.state {
            if state.connections.iter().any(|c| c.on) {
                icon.push(VPN_ICON.clone());
                return icon;
            }
        }

        icon
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mut menu = vec![];

        if let Some(state) = &self.state {
            let mut connections = state
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

            menu.push(MenuItem::Separator);
            menu.append(&mut connections);
        }

        menu
    }
}
