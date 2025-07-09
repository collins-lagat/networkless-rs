use zbus::{
    Connection, Result,
    zvariant::{ObjectPath, OwnedObjectPath},
};

use crate::interfaces::{
    active::ActiveProxy, device::DeviceProxy, devices::wireless::WirelessProxy,
    settings::connection::ConnectionProxy,
};

use super::{
    active_connection::ActiveConnection,
    devices::{SpecificDevice, Wireless},
    enums::{DeviceState, DeviceType},
    settings::ConnectionSetting,
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

    pub async fn disconnect(&self) -> Result<()> {
        self.device.disconnect().await
    }

    pub fn path(&self) -> ObjectPath<'static> {
        self.device.inner().path().clone()
    }

    pub async fn active_connection(&self) -> Result<ActiveConnection> {
        // BUG: It's possible for self.device.connection() to return an ObjectPath("/") which means that the
        // active connection is not set. This will cause panics when you try to acess properties
        // and call methods on it. Until this is fixed, we need to check if the active connection
        // doesn't throw an error on any of the methods.
        // In the future, this will probably return an Option<ActiveConnection> instead of
        // a Result<ActiveConnection> and handle the error in the caller.
        let active_connection = self.device.active_connection().await?;
        let active_connection = ActiveProxy::builder(self.device.inner().connection())
            .path(active_connection)?
            .build()
            .await?;
        Ok(ActiveConnection::new(active_connection))
    }

    pub async fn available_connections(&self) -> Result<Vec<ConnectionSetting>> {
        let configured_connections = self.device.available_connections().await.unwrap();

        let mut out = Vec::with_capacity(configured_connections.len());

        for conn in configured_connections {
            let setting = ConnectionProxy::builder(self.device.inner().connection())
                .path(conn)?
                .build()
                .await?;
            out.push(ConnectionSetting::new(setting));
        }

        Ok(out)
    }

    // pub async fn with_connection_and_path<'a, F, Fut, R>(&'a self, f: F) -> Option<R>
    // where
    //     F: FnOnce(&'a Connection, ObjectPath<'a>) -> Fut,
    //     Fut: Future<Output = R> + 'a,
    // {
    //     let connection = self.device.inner().connection();
    //     let path = self.device.inner().path().clone();
    //     let r = f(connection, path).await;
    //     Some(r)
    // }

    pub async fn to_specific_device(&self) -> Option<SpecificDevice> {
        let device_type = match self.device_type().await {
            Ok(device_type) => device_type,
            Err(_) => return None,
        };

        match device_type {
            DeviceType::Wifi => {
                let connection = self.device.inner().connection();
                let path = self.device.inner().path().clone();
                let wireless_device = WirelessProxy::builder(connection)
                    .path(path)
                    .unwrap()
                    .build()
                    .await
                    .unwrap();
                let device = Wireless::new(wireless_device).await;
                Some(SpecificDevice::Wireless(device))
            }
            DeviceType::Ethernet => {
                // let connection = self.device.inner().connection();
                // let path = self.device.inner().path().clone();
                // let wired_device = WiredProxy::builder(connection)
                //     .path(path)
                //     .unwrap()
                //     .build()
                //     .await
                //     .unwrap();
                // Some(SpecificDevice::Wired(wired_device))
                Some(SpecificDevice::Wired(()))
            }
            DeviceType::WireGuard => {
                // let connection = self.device.inner().connection();
                // let path = self.device.inner().path().clone();
                // let wire_guard_device = WireGuardProxy::builder(connection)
                //     .path(path)
                //     .unwrap()
                //     .build()
                //     .await
                //     .unwrap();
                // Some(SpecificDevice::WireGuard(wire_guard_device))
                Some(SpecificDevice::WireGuard(()))
            }
            _ => None,
        }
    }

    pub async fn from_connection_and_path(connection: &Connection, path: OwnedObjectPath) -> Self {
        let device = DeviceProxy::builder(connection)
            .path(path)
            .unwrap()
            .build()
            .await
            .unwrap();
        Self::new(device)
    }
}
