use anyhow::Result;
use ksni::menu::{RadioGroup, RadioItem};

use crate::network::{access_point::AccessPoint, device::Device, devices::SpecificDevice};

use super::NetworkTray;

pub struct KnownNetworksSection {
    device: Device,
    selected: Option<AccessPoint>,
    aps: Vec<AccessPoint>,
}

impl KnownNetworksSection {
    pub fn new(device: Device) -> Self {
        Self {
            device,
            selected: None,
            aps: Vec::new(),
        }
    }
    async fn sync(&mut self) -> Result<()> {
        let configured_connections = self.device.available_connections().await.unwrap();
        let mut aps = Vec::with_capacity(configured_connections.len());

        let wireless_device = match self.device.to_specific_device().await {
            Some(SpecificDevice::Wireless(wireless_device)) => wireless_device,
            _ => anyhow::bail!("Device is not a wireless device"),
        };

        let active_access_point = wireless_device.active_access_point().await.unwrap();

        self.selected = Some(active_access_point.clone());
        aps.push(active_access_point);

        // let available_access_points = wireless_device
        //     .access_points()
        //     .await
        //     .unwrap()
        //     .iter()
        //     .filter(|ap| async {
        //         ap.id().await
        //             != active_access_point
        //                 .id()
        //                 .await
        //                 .unwrap_or("Unknown".to_string())
        //     })
        //     .collect::<Vec<&AccessPoint>>();

        self.aps = aps;

        Ok(())
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
            selected: 0,
            select: Box::new(|this: , selected | this.selecte_known_network(selected)),
            options,
            ..Default::default()
        }
    }

    fn selected(&mut self, selected: usize) {
        self.selected = self.aps.get(selected).cloned();
    }
}
