use anyhow::Result as AnyResult;
use futures::StreamExt;
use log::{error, info};
use zbus::{
    Result,
    zvariant::{ObjectPath, OwnedObjectPath},
};

use crate::{
    interfaces::{active::ActiveProxy, device::DeviceProxy},
    network::enums::ActiveConnectionState,
};

use super::{device::Device, enums::DeviceType};

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

    pub async fn device_type(&self) -> Result<DeviceType> {
        self.active_connection.type_().await.map(DeviceType::from)
    }

    pub async fn connection(&self) -> Result<OwnedObjectPath> {
        self.active_connection.connection().await
    }

    pub fn path(&self) -> ObjectPath<'static> {
        self.active_connection.inner().path().clone()
    }

    pub async fn specific_object(&self) -> Result<OwnedObjectPath> {
        let specific_object = match self.active_connection.specific_object().await {
            Ok(specific_object) => specific_object,
            Err(e) => {
                error!("Failed to get SpecificObject: {}", e);
                return Err(e);
            }
        };

        Ok(specific_object)
    }

    pub async fn with<'a, F, Fut, R>(&'a self, f: F) -> Option<R>
    where
        F: FnOnce(OwnedObjectPath, OwnedObjectPath, Vec<OwnedObjectPath>) -> Fut,
        Fut: Future<Output = R> + 'a,
    {
        let connection = self.active_connection.connection().await.unwrap();
        let specific_object = self.active_connection.specific_object().await.unwrap();

        let devices = self.active_connection.devices().await.unwrap();

        let r = f(connection, specific_object, devices).await;
        Some(r)
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
