use zbus::Result;

use crate::interfaces::{
    active::ActiveProxy, device::DeviceProxy, devices::wireless::WirelessProxy,
};

use super::{
    active_connection::ActiveConnection,
    devices::{SpecificDevice, Wireless},
    enums::DeviceType,
};

#[derive(Debug, Clone)]
pub struct Device {
    device: DeviceProxy<'static>,
}

impl Device {
    pub fn new(device: DeviceProxy<'static>) -> Self {
        Self { device }
    }

    // pub async fn state(&self) -> Result<DeviceState> {
    //     self.device.state().await.map(DeviceState::from)
    // }

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

    // pub async fn available_connections(&self) -> Result<Vec<ConnectionSetting>> {
    //     let configured_connections = self.device.available_connections().await.unwrap();
    //
    //     let mut out = Vec::with_capacity(configured_connections.len());
    //
    //     for conn in configured_connections {
    //         let setting = ConnectionProxy::builder(self.device.inner().connection())
    //             .path(conn)?
    //             .build()
    //             .await?;
    //         out.push(ConnectionSetting::new(setting));
    //     }
    //
    //     Ok(out)
    // }
    //
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
        match self.device_type().await.unwrap() {
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
}
