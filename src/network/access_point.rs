use zbus::{Result, zvariant::ObjectPath};

use crate::interfaces::access_point::AccessPointProxy;

#[derive(Clone, Debug)]
pub struct AccessPoint {
    id: Option<String>,
    access_point: AccessPointProxy<'static>,
}

impl AccessPoint {
    pub fn new(access_point: AccessPointProxy<'static>) -> Self {
        Self {
            id: None,
            access_point,
        }
    }

    pub async fn id(&mut self) -> Result<&str> {
        if self.id.is_some() {
            return Ok(self.id.as_ref().unwrap());
        }
        let ssid = self.access_point.ssid().await?;
        let id = String::from_utf8_lossy(&ssid);
        self.id = Some(id.to_string());
        Ok(self.id.as_ref().unwrap())
    }

    pub async fn ssid(&self) -> Result<Vec<u8>> {
        self.access_point.ssid().await
    }

    pub async fn hw_address(&self) -> Result<String> {
        self.access_point.hw_address().await
    }

    pub async fn strength(&self) -> Result<u8> {
        self.access_point.strength().await
    }

    pub fn path(&self) -> ObjectPath<'static> {
        self.access_point.inner().path().clone()
    }
}
