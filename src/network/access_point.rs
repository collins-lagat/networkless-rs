use zbus::{Result, zvariant::ObjectPath};

use crate::interfaces::access_point::AccessPointProxy;

use super::enums::{ApFlags, ApSecurityFlags};

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

    pub async fn hw_address(&self) -> Result<String> {
        self.access_point.hw_address().await
    }

    pub async fn strength(&self) -> Result<u8> {
        self.access_point.strength().await
    }

    pub async fn flags(&self) -> Result<ApFlags> {
        self.access_point
            .flags()
            .await
            .map(ApFlags::from_bits_truncate)
    }

    pub async fn rsn_flags(&self) -> Result<ApSecurityFlags> {
        self.access_point
            .rsn_flags()
            .await
            .map(ApSecurityFlags::from_bits_truncate)
    }

    pub async fn wpa_flags(&self) -> Result<ApSecurityFlags> {
        self.access_point
            .wpa_flags()
            .await
            .map(ApSecurityFlags::from_bits_truncate)
    }

    pub async fn secure(&self) -> Result<bool> {
        let has_privacy_flag = self.flags().await?.contains(ApFlags::PRIVACY);
        let has_some_level_of_security = !self.rsn_flags().await?.contains(ApSecurityFlags::NONE)
            && !self.wpa_flags().await?.contains(ApSecurityFlags::NONE);
        Ok(has_privacy_flag && has_some_level_of_security)
    }

    pub fn path(&self) -> ObjectPath<'static> {
        self.access_point.inner().path().clone()
    }
}
