use anyhow::{Result, bail};
use tokio::process::Command;
use zbus::{Connection, Result as ZbusResult};

use crate::dbus::NetworkManagerProxy;

#[derive(Debug, Clone)]
pub struct NetworkManager {
    nm: NetworkManagerProxy<'static>,
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

        let bluetooth_enabled = output.contains("Soft blocked: yes");
        Ok(bluetooth_enabled)
    }

    pub async fn airplane_mode_enabled(&self) -> Result<bool> {
        let wifi_enabled = self.wifi_enabled().await?;
        let bluetooth_enabled = self.bluetooth_enabled().await?;

        Ok(wifi_enabled && bluetooth_enabled)
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
