use anyhow::Result as AnyResult;
use futures::StreamExt;
use log::info;
use zbus::Result;

use crate::{
    interfaces::{active::ActiveProxy, device::DeviceProxy},
    network::enums::ActiveConnectionState,
};

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

    pub async fn state(&self) -> Result<ActiveConnectionState> {
        self.active_connection
            .state()
            .await
            .map(ActiveConnectionState::from)
    }

    // pub async fn device_type(&self) -> Result<DeviceType> {
    //     self.active_connection.type_().await.map(DeviceType::from)
    // }

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

    pub async fn listening_to_state_changes<F>(&self, f: F) -> AnyResult<()>
    where
        F: AsyncFnOnce(ActiveConnectionState) -> () + Send + Copy,
    {
        let mut stream = self
            .active_connection
            .receive_state_changed_signal()
            .await?;

        while let Some(state_changed) = stream.next().await {
            info!("Active Connection State changed");

            let state = match state_changed.args() {
                Ok(state) => state.state().to_owned(),
                Err(e) => {
                    anyhow::bail!("Failed to get StateChanged arguments: {e}");
                }
            };

            let state = ActiveConnectionState::from(state);

            f(state).await;
        }
        Ok(())
    }
}
