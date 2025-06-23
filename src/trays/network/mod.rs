use airplane_mode::AirplaneModeMenu;
use anyhow::Result;
use ksni::{MenuItem, Tray};
use vpn::VpnMenu;
use wired::WiredMenu;
use wireless::WirelessNetworkMenu;

use crate::{
    APP_ID,
    network::{devices::SpecificDevice, enums::DeviceType, network_manager::NetworkManager},
};

mod airplane_mode;
mod vpn;
mod wired;
mod wireless;

#[derive(Debug, Clone)]
pub struct NetworkTray {
    wireless_menu: Option<WirelessNetworkMenu>,
    wired_menu: Option<WiredMenu>,
    vpn_menu: Option<VpnMenu>,
    airplane_mode_menu: Option<AirplaneModeMenu>,
}

impl NetworkTray {
    pub async fn new(nm: NetworkManager) -> Result<Self> {
        let airplane_mode_menu = Some(AirplaneModeMenu::new(nm.clone()));
        let mut wireless_menu = None;
        let mut wired_menu = None;
        let mut vpn_menu = None;

        let devices = nm.all_devices().await.unwrap();

        for device in devices {
            if device.device_type().await.unwrap() == DeviceType::Wifi {
                wireless_menu = Some(WirelessNetworkMenu::new(nm.clone(), device.clone()));
            }

            if device.device_type().await.unwrap() == DeviceType::Ethernet {
                let wired_device = match device.to_specific_device().await {
                    Some(SpecificDevice::Wired(wired_device)) => wired_device,
                    _ => anyhow::bail!("Device is not a wired device"),
                };
                wired_menu = Some(WiredMenu::new(wired_device));
            }

            if device.device_type().await.unwrap() == DeviceType::WireGuard {
                let active_connection = device.active_connection().await.unwrap();
                vpn_menu = Some(VpnMenu::new(active_connection));
            }
        }

        Ok(Self {
            airplane_mode_menu,
            wireless_menu,
            wired_menu,
            vpn_menu,
        })
    }

    pub fn toggle_wireless(&mut self) {}
    pub fn toggle_wired(&mut self) {}
    pub fn toggle_vpn(&mut self) {}

    pub fn select_available_network(&mut self, _selected: usize) {}

    pub fn selecte_known_network(&mut self, _selected: usize) {}
}

impl Tray for NetworkTray {
    fn id(&self) -> String {
        APP_ID.into()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mut menu: Vec<MenuItem<Self>> = Vec::with_capacity(4);

        if let Some(wireless_network_menu) = self.wireless_menu.as_ref() {
            menu.push(wireless_network_menu.menu_item());
        }

        if let Some(wired_menu) = self.wired_menu.as_ref() {
            menu.push(wired_menu.menu_item());
        }

        if let Some(vpn_menu) = self.vpn_menu.as_ref() {
            menu.push(vpn_menu.menu_item());
        }

        if let Some(airplane_mode_menu) = self.airplane_mode_menu.as_ref() {
            menu.push(airplane_mode_menu.menu_item());
        }

        menu
    }
}
