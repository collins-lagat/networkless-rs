use anyhow::Result;
use anyhow::bail;
use futures::StreamExt;
use tokio::process::Command;
use zbus::Connection;
use zbus::Result as ZbusResult;

use super::active_connection::ActiveConnection;
use super::device::Device;
use super::enums::DeviceType;
use super::enums::NmConnectivityState;
use super::enums::NmState;
use crate::interfaces::active::ActiveProxy;
use crate::interfaces::network_manager::DeviceAdded;
use crate::interfaces::network_manager::DeviceRemoved;
use crate::interfaces::network_manager::StateChanged;
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
        F: AsyncFnOnce(StateChanged) -> () + Send + Copy,
    {
        let mut stream = self.nm.receive_state_changed_signal().await?;
        while let Some(state) = stream.next().await {
            f(state).await;
        }
        Ok(())
    }

    pub async fn listening_to_device_added<F>(&self, f: F) -> Result<()>
    where
        F: AsyncFnOnce(DeviceAdded) -> () + Send + Copy,
    {
        let mut stream = self.nm.receive_device_added().await?;
        while let Some(state) = stream.next().await {
            f(state).await;
        }
        Ok(())
    }

    pub async fn listening_to_device_removed<F>(&self, f: F) -> Result<()>
    where
        F: AsyncFnOnce(DeviceRemoved) -> () + Send + Copy,
    {
        let mut stream = self.nm.receive_device_removed().await?;
        while let Some(state) = stream.next().await {
            f(state).await;
        }
        Ok(())
    }

    pub async fn devices(&self) -> Result<Vec<Device>> {
        let devices = self.nm.get_devices().await?;

        let mut out = Vec::with_capacity(devices.len());

        for device in devices {
            let device = DeviceProxy::builder(&self.connection)
                .path(device)?
                .build()
                .await?;
            out.push(Device::new(device));
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
        let primary_connection = self.nm.primary_connection().await?;
        let primary_connection = ActiveProxy::builder(&self.connection)
            .path(primary_connection)?
            .build()
            .await?;
        Ok(ActiveConnection::new(primary_connection))
    }

    pub async fn primary_connection_type(&self) -> ZbusResult<DeviceType> {
        self.nm
            .primary_connection_type()
            .await
            .map(DeviceType::from)
    }

    pub async fn check_connectivity(&self) -> Result<()> {
        let state = self.state().await?;
        if state == NmState::Disconnected {
            self.nm.check_connectivity().await?;
        }
        Ok(())
    }

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
        active_connection: &zbus::zvariant::OwnedObjectPath,
    ) -> Result<()> {
        self.nm.deactivate_connection(active_connection).await?;
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
}
