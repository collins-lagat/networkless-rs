use zbus::Result;

use crate::interfaces::access_point::AccessPointProxy;

pub struct AccessPoint {
    access_point: AccessPointProxy<'static>,
}

impl AccessPoint {
    pub fn new(access_point: AccessPointProxy<'static>) -> Self {
        Self { access_point }
    }

    pub async fn strength(&self) -> Result<u8> {
        self.access_point.strength().await
    }
}
