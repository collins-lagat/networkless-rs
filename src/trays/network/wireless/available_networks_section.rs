use ksni::menu::{RadioGroup, RadioItem};

use crate::network::{access_point::AccessPoint, devices::Wireless};

use super::NetworkTray;

pub struct AvailableNetworksSection {
    wireless_device: Wireless,
    aps: Vec<AccessPoint>,
}

impl AvailableNetworksSection {
    pub fn new(wireless_device: Wireless) -> Self {
        Self {
            wireless_device,
            aps: Vec::new(),
        }
    }

    async fn sync(&mut self) {
        self.aps = self.wireless_device.access_points().await.unwrap();
    }

    pub fn menu_item(&mut self) -> RadioGroup<NetworkTray> {
        let options = self
            .aps
            .iter_mut()
            .map(|ap| {
                let label = tokio::runtime::Handle::current().block_on(async {
                    let label = ap.id().await.unwrap_or("Unknown");
                    label.to_string()
                });

                RadioItem {
                    label,
                    ..Default::default()
                }
            })
            .collect::<Vec<RadioItem>>();

        RadioGroup {
            select: Box::new(|this, selected| this.select_available_network(selected)),
            options,
            ..Default::default()
        }
    }
}
