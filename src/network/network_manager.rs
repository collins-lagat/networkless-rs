use anyhow::Result;
use anyhow::bail;
use futures::StreamExt;
use log::info;
use tokio::process::Command;
use zbus::Connection;
use zbus::Result as ZbusResult;
use zbus::zvariant::ObjectPath;

use super::active_connection::ActiveConnection;
use super::device::Device;
use super::devices::SpecificDevice;
use super::enums::NmConnectivityState;
use super::enums::NmState;
use crate::interfaces::active::ActiveProxy;
use crate::interfaces::{device::DeviceProxy, network_manager::NetworkManagerProxy};

#[derive(Debug, Clone)]
pub struct NetworkManager {
    connection: Connection,
    nm: NetworkManagerProxy<'static>,
}

impl NetworkManager {
    pub async fn new(connection: Connection) -> Result<Self> {
        let nm = NetworkManagerProxy::new(&connection).await?;
        Ok(Self { connection, nm })
    }

    pub async fn listening_to_state_changes<F>(&self, f: F) -> Result<()>
    where
        F: AsyncFnOnce(NmState) -> () + Send + Copy,
    {
        let mut stream = self.nm.receive_state_changed_signal().await?;
        while let Some(state_changed) = stream.next().await {
            info!("State changed");
            let state = match state_changed.args() {
                Ok(args) => args.state().to_owned(),
                Err(e) => {
                    anyhow::bail!("Failed to get StateChanged arguments: {e}");
                }
            };

            let state = NmState::from(state);

            f(state).await;
        }
        Ok(())
    }

    pub async fn listening_to_device_added<F>(&self, f: F) -> Result<()>
    where
        F: AsyncFnOnce(Option<SpecificDevice>) -> () + Send + Copy,
    {
        let mut stream = self.nm.receive_device_added().await?;

        // Idea from https://github.com/pop-os/cosmic-settings/blob/bd46f922155f6df172096f0f86f144968903c380/cosmic-settings/src/pages/power/backend/mod.rs#L491
        while let Some(device_added) = stream.next().await {
            let device_path: ObjectPath<'static> = match device_added.args() {
                Ok(args) => args.device_path().to_owned(),
                Err(e) => {
                    anyhow::bail!("Failed to get DeviceAdded arguments: {e}");
                }
            };

            let device = match DeviceProxy::builder(&self.connection)
                .path(&device_path)
                .unwrap()
                .build()
                .await
            {
                Ok(device_proxy) => Device::new(device_proxy).to_specific_device().await,
                Err(e) => {
                    anyhow::bail!("Failed to build DeviceProxy: {e}");
                }
            };

            f(device).await;

            info!("Device added");
        }
        Ok(())
    }

    pub async fn listening_to_device_removed<F>(&self, f: F) -> Result<()>
    where
        F: AsyncFnOnce(Option<SpecificDevice>) -> () + Send + Copy,
    {
        let mut stream = self.nm.receive_device_removed().await?;
        while let Some(device_removed) = stream.next().await {
            info!("Device removed");
            let device_path: ObjectPath<'static> = match device_removed.args() {
                Ok(args) => args.device_path().to_owned(),
                Err(e) => {
                    anyhow::bail!("Failed to get DeviceRemoved arguments: {e}");
                }
            };

            let device = match DeviceProxy::builder(&self.connection)
                .path(&device_path)
                .unwrap()
                .build()
                .await
            {
                Ok(device_proxy) => Device::new(device_proxy).to_specific_device().await,
                Err(e) => {
                    anyhow::bail!("Failed to build DeviceProxy: {e}");
                }
            };

            f(device).await;
        }
        Ok(())
    }

    pub async fn devices(&self) -> Result<Vec<Device>> {
        let devices = self.nm.get_devices().await?;

        let mut out = Vec::with_capacity(devices.len());

        for device in devices {
            // BUG: It's possible for device to be an ObjectPath("/") which means that the device
            // is not set. This will cause panics when you try to acess properties
            // and call methods on it. Until this is fixed, we need to check if the device
            // doesn't throw an error on any of the methods.
            // In the future, invalid devices will probably be filtered out from the list
            let device = DeviceProxy::builder(&self.connection)
                .path(device)?
                .build()
                .await?;
            out.push(Device::new(device));
        }

        Ok(out)
    }

    pub async fn all_devices(&self) -> Result<Vec<Device>> {
        let devices = self.nm.get_devices().await?;

        let mut out = Vec::with_capacity(devices.len());

        for device in devices {
            // BUG: It's possible for device to be an ObjectPath("/") which means that the device
            // is not set. This will cause panics when you try to acess properties
            // and call methods on it. Until this is fixed, we need to check if the device
            // doesn't throw an error on any of the methods.
            // In the future, invalid devices will probably be filtered out from the list
            let deice = Device::from_connection_and_path(&self.connection, device).await;
            out.push(deice);
        }

        Ok(out)
    }

    pub async fn state(&self) -> ZbusResult<NmState> {
        self.nm.state().await.map(NmState::from)
    }

    pub async fn connectivity(&self) -> ZbusResult<NmConnectivityState> {
        self.nm.connectivity().await.map(NmConnectivityState::from)
    }

    pub async fn primary_connection(&self) -> ZbusResult<ActiveConnection> {
        // BUG: It's possible for self.nm.primary_connection() to return an ObjectPath("/") which means that an
        // active connection is not set. This will cause panics when you try to acess properties
        // and call methods on it. Until this is fixed, we need to check if the active connection
        // doesn't throw an error on any of the methods.
        // In the future, this will probably return an Option<ActiveConnection> instead of
        // a Result<ActiveConnection> and handle the error in the caller.

        let primary_connection = self.nm.primary_connection().await?;
        let primary_connection = ActiveProxy::builder(&self.connection)
            .path(primary_connection)?
            .build()
            .await?;
        Ok(ActiveConnection::new(primary_connection))
    }

    pub async fn active_connections(&self) -> Result<Vec<ActiveConnection>> {
        let active_connections = self.nm.active_connections().await?;
        let mut out = Vec::with_capacity(active_connections.len());
        for active_connection in active_connections {
            let active_connection = ActiveProxy::builder(&self.connection)
                .path(active_connection)?
                .build()
                .await?;
            out.push(ActiveConnection::new(active_connection));
        }
        Ok(out)
    }

    // pub async fn primary_connection_type(&self) -> ZbusResult<DeviceType> {
    //     self.nm
    //         .primary_connection_type()
    //         .await
    //         .map(DeviceType::from)
    // }
    //
    // pub async fn check_connectivity(&self) -> Result<NmConnectivityState> {
    //     let connectivity = self.nm.check_connectivity().await?;
    //     let connectivity = NmConnectivityState::from(connectivity);
    //     Ok(connectivity)
    // }

    pub async fn activate_connection(
        &self,
        active_connection: zbus::zvariant::OwnedObjectPath,
        device: zbus::zvariant::OwnedObjectPath,
        specific_object: zbus::zvariant::OwnedObjectPath,
    ) -> Result<()> {
        self.nm
            .activate_connection(&active_connection, &device, &specific_object)
            .await?;
        Ok(())
    }

    pub async fn deactivate_connection(
        &self,
        active_connection: zbus::zvariant::OwnedObjectPath,
    ) -> Result<()> {
        self.nm.deactivate_connection(&active_connection).await?;
        Ok(())
    }

    pub async fn wifi_enabled(&self) -> Result<bool> {
        Ok(self.nm.wireless_enabled().await?)
    }

    pub async fn set_wifi_enabled(&self, enabled: bool) -> Result<()> {
        self.nm.set_wireless_enabled(enabled).await?;
        Ok(())
    }

    pub async fn bluetooth_enabled(&self) -> Result<bool> {
        let cmd = Command::new("rfkill")
            .arg("list")
            .arg("bluetooth")
            .output()
            .await?;
        if !cmd.status.success() {
            bail!("Failed to list bluetooth devices");
        }

        let output = String::from_utf8(cmd.stdout)?;

        let bluetooth_enabled = output.contains("Soft blocked: no");
        Ok(bluetooth_enabled)
    }

    pub async fn airplane_mode_enabled(&self) -> Result<bool> {
        let wifi_enabled = self.wifi_enabled().await?;
        let bluetooth_enabled = self.bluetooth_enabled().await?;

        if !wifi_enabled && !bluetooth_enabled {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn with_connection<'a, F, Fut, R>(&'a self, f: F) -> Option<R>
    where
        F: FnOnce(&'a Connection) -> Fut,
        Fut: Future<Output = R> + 'a,
    {
        let connection = &self.connection;

        let r = f(connection).await;

        Some(r)
    }
}
