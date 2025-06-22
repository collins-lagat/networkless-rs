use zbus::{Connection, Result};

use crate::interfaces::{active::ActiveProxy, device::DeviceProxy};

use super::{
    active_connection::ActiveConnection,
    enums::{DeviceState, DeviceType},
};

#[derive(Debug, Clone)]
pub struct Device {
    device: DeviceProxy<'static>,
}

impl Device {
    pub fn new(device: DeviceProxy<'static>) -> Self {
        Self { device }
    }

    pub async fn state(&self) -> Result<DeviceState> {
        self.device.state().await.map(DeviceState::from)
    }

    pub async fn device_type(&self) -> Result<DeviceType> {
        self.device.device_type().await.map(DeviceType::from)
    }

    pub async fn active_connection(&self) -> Result<ActiveConnection> {
        let active_connection = self.device.active_connection().await?;
        let active_connection = ActiveProxy::builder(self.device.inner().connection())
            .path(active_connection)?
            .build()
            .await?;
        Ok(ActiveConnection::new(active_connection))
    }

    pub async fn with_connection_and_path<'a, F, Fut, R>(&'a self, f: F) -> Option<R>
    where
        F: FnOnce(&'a Connection, ObjectPath<'a>) -> Fut,
        Fut: Future<Output = R> + 'a,
    {
        let connection = self.device.inner().connection();
        let path = self.device.inner().path().clone();
        let r = f(connection, path).await;
        Some(r)
    }
}
