use zbus::Result;

use crate::interfaces::{active::ActiveProxy, device::DeviceProxy};

use super::device::Device;

#[derive(Debug, Clone)]
pub struct ActiveConnection {
    active_connection: ActiveProxy<'static>,
}

impl ActiveConnection {
    pub fn new(active_connection: ActiveProxy<'static>) -> Self {
        Self { active_connection }
    }

    pub async fn id(&self) -> Result<String> {
        self.active_connection.id().await
    }

    pub async fn devices(&self) -> Result<Vec<Device>> {
        let devices = self.active_connection.devices().await?;

        let mut out = Vec::with_capacity(devices.len());

        for device in devices {
            let device = DeviceProxy::builder(self.active_connection.inner().connection())
                .path(device)?
                .build()
                .await?;
            out.push(Device::new(device));
        }

        Ok(out)
    }
}
