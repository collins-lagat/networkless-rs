use anyhow::Result;
use ksni::{
    MenuItem,
    menu::{StandardItem, SubMenu},
};

use crate::network::{
    device::Device,
    devices::{SpecificDevice, Wireless},
    network_manager::NetworkManager,
};

use super::NetworkTray;

mod available_networks_section;
mod known_networks_section;
mod wireless_toggle;

use available_networks_section::AvailableNetworksSection;
use known_networks_section::KnownNetworksSection;
use wireless_toggle::WirelessToggle;

#[derive(Debug, Clone)]
pub struct WirelessNetworkMenu {
    nm: NetworkManager,
    device: Device,
    wireless_device: Option<Wireless>,
}

impl WirelessNetworkMenu {
    pub fn new(nm: NetworkManager, device: Device) -> Self {
        Self {
            nm,
            device,
            wireless_device: None,
        }
    }

    pub async fn sync(&mut self) -> Result<()> {
        let wireless_device = match self.device.to_specific_device().await {
            Some(SpecificDevice::Wireless(wireless_device)) => wireless_device,
            _ => anyhow::bail!("Device is not a wireless device"),
        };

        self.wireless_device = Some(wireless_device);

        Ok(())
    }

    pub fn menu_item(&self) -> MenuItem<NetworkTray> {
        let mut submenu: Vec<MenuItem<NetworkTray>> = vec![];

        let toggle = WirelessToggle::new(self.nm.clone());

        submenu.push(toggle.menu_item().into());

        submenu.push(MenuItem::Separator);

        let mut known_networks_section = KnownNetworksSection::new(self.device.clone());

        submenu.push(known_networks_section.menu_item().into());

        submenu.push(MenuItem::Separator);

        submenu.push(
            StandardItem {
                label: "Available Networks".to_string(),
                enabled: false,
                ..Default::default()
            }
            .into(),
        );

        if let Some(wireless_device) = self.wireless_device.as_ref() {
            let mut available_networks_section =
                AvailableNetworksSection::new(wireless_device.clone());

            submenu.push(available_networks_section.menu_item().into());
        }

        SubMenu {
            submenu,
            ..Default::default()
        }
        .into()
    }
}
