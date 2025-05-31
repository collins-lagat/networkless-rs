use anyhow::{Result, bail};
use tokio::process::Command;
use zbus::{Connection, Result as ZbusResult};

use crate::interfaces::{
    active::ActiveProxy, device::DeviceProxy, network_manager::NetworkManagerProxy,
};

#[derive(Debug, Clone)]
pub struct NetworkManager {
    pub nm: NetworkManagerProxy<'static>,
}

impl NetworkManager {
    pub async fn new() -> Result<Self> {
        let connection = Connection::system().await?;
        let nm = NetworkManagerProxy::new(&connection).await?;

        Ok(Self { nm })
    }

    pub async fn state(&self) -> ZbusResult<State> {
        self.nm.state().await.map(State::from)
    }

    pub async fn connectivity(&self) -> ZbusResult<Connectivity> {
        self.nm.connectivity().await.map(Connectivity::from)
    }

    pub async fn wifi_enabled(&self) -> Result<bool> {
        Ok(self.nm.wireless_enabled().await?)
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

    pub async fn active_connection(&self) -> Result<ActiveConnection> {
        let active_connections = self.nm.primary_connection().await?;
        Ok(ActiveConnection::new(active_connections).await.unwrap())
    }

    pub async fn activate_connection(
        &self,
        connection: zbus::zvariant::OwnedObjectPath,
        device: zbus::zvariant::OwnedObjectPath,
        specific_object: zbus::zvariant::OwnedObjectPath,
    ) -> Result<()> {
        self.nm
            .activate_connection(&connection, &device, &specific_object)
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
}

#[derive(Debug, Clone)]
pub struct ActiveConnection {
    pub connection: ActiveProxy<'static>,
}

impl ActiveConnection {
    pub async fn new(path: zbus::zvariant::OwnedObjectPath) -> Result<Self> {
        let connection = Connection::system().await?;
        let active_connection = ActiveProxy::new(&connection, path).await?;

        Ok(Self {
            connection: active_connection,
        })
    }

    pub async fn devices(&self) -> ZbusResult<Vec<Device>> {
        let mut devices = vec![];

        for device_path in self.connection.devices().await? {
            let device = Device::new(device_path).await.unwrap();
            devices.push(device);
        }

        Ok(devices)
    }
}

#[derive(Debug, Clone)]
pub struct Device {
    pub path: zbus::zvariant::OwnedObjectPath,
    pub device: DeviceProxy<'static>,
}

impl Device {
    pub async fn new(path: zbus::zvariant::OwnedObjectPath) -> Result<Self> {
        let connection = Connection::system().await?;
        let device = DeviceProxy::new(&connection, path.clone()).await?;

        Ok(Self { device, path })
    }

    pub async fn device_type(&self) -> ZbusResult<DeviceType> {
        self.device.device_type().await.map(DeviceType::from)
    }
}

// https://github.com/pop-os/dbus-settings-bindings/blob/3b86984332be2c930a3536ab714b843c851fa8ca/networkmanager/src/interface/enums.rs#L1

#[derive(Debug, Clone, Default)]
pub enum State {
    Asleep,
    Disconnected,
    Disconnecting,
    Connecting,
    ConnectedLocal,
    ConnectedSite,
    ConnectedGlobal,
    #[default]
    Unknown,
}

impl From<u32> for State {
    fn from(state: u32) -> State {
        match state {
            10 => State::Asleep,
            20 => State::Disconnected,
            30 => State::Disconnecting,
            40 => State::Connecting,
            50 => State::ConnectedLocal,
            60 => State::ConnectedSite,
            70 => State::ConnectedGlobal,
            _ => State::Unknown,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum Connectivity {
    None,
    Portal,
    Loss,
    Full,
    #[default]
    Unknown,
}

impl From<u32> for Connectivity {
    fn from(state: u32) -> Connectivity {
        match state {
            1 => Connectivity::None,
            2 => Connectivity::Portal,
            3 => Connectivity::Loss,
            4 => Connectivity::Full,
            _ => Connectivity::Unknown,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum DeviceType {
    Ethernet,
    Wifi,
    Bluetooth,
    TunTap,
    WireGuard,
    Generic,
    Other,
    #[default]
    Unknown,
}

impl From<u32> for DeviceType {
    fn from(device_type: u32) -> DeviceType {
        match device_type {
            1 => DeviceType::Ethernet,
            2 => DeviceType::Wifi,
            5 => DeviceType::Bluetooth,
            14 => DeviceType::Generic,
            16 => DeviceType::TunTap,
            29 => DeviceType::WireGuard,
            3..=32 => DeviceType::Other,
            _ => DeviceType::Unknown,
        }
    }
}
