use zbus::Result;

use crate::interfaces::device::DeviceProxy;

use super::enums::DeviceType;

#[derive(Debug, Clone)]
pub struct Device {
    device: DeviceProxy<'static>,
}

impl Device {
    pub fn new(device: DeviceProxy<'static>) -> Self {
        Self { device }
    }

    pub async fn device_type(&self) -> Result<DeviceType> {
        self.device.device_type().await.map(DeviceType::from)
    }
}
