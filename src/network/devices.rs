use std::collections::HashMap;

use zbus::{Result, zvariant::Value};

use crate::interfaces::{access_point::AccessPointProxy, devices::wireless::WirelessProxy};

use super::access_point::AccessPoint;

#[derive(Debug, Clone)]
pub struct Wireless {
    wireless_device: WirelessProxy<'static>,
}

impl Wireless {
    pub async fn new(wireless_device: WirelessProxy<'static>) -> Self {
        Self { wireless_device }
    }

    pub async fn active_access_point(&self) -> Result<AccessPoint> {
        let ap = self.wireless_device.active_access_point().await?;
        let ap = AccessPointProxy::builder(self.wireless_device.inner().connection())
            .path(ap)?
            .build()
            .await?;
        Ok(AccessPoint::new(ap))
    }

    pub async fn access_points(&self) -> Result<Vec<AccessPoint>> {
        let aps = self.wireless_device.access_points().await?;
        let mut out = Vec::with_capacity(aps.len());
        for ap in aps {
            let ap = AccessPointProxy::builder(self.wireless_device.inner().connection())
                .path(ap)?
                .build()
                .await?;
            out.push(AccessPoint::new(ap));
        }
        Ok(out)
    }

    pub async fn request_scan(&self, opts: HashMap<&str, &Value<'static>>) -> Result<()> {
        self.wireless_device.request_scan(opts).await
    }
}
//
// #[derive(Debug, Clone)]
// pub struct Wired {
//     wired_device: WiredProxy<'static>,
// }
//
// impl Wired {
//     pub async fn new(wired_device: WiredProxy<'static>) -> Self {
//         Self { wired_device }
//     }
// }
//
// #[derive(Debug, Clone)]
// pub struct WireGuard {
//     wire_guard_device: WireGuardProxy<'static>,
// }
//
// impl WireGuard {
//     pub async fn new(wire_guard_device: WireGuardProxy<'static>) -> Self {
//         Self { wire_guard_device }
//     }
// }

pub enum SpecificDevice {
    Wireless(Wireless),
    // Wired(Wired),
    // WireGuard(WireGuard),
    Wired(()),
    WireGuard(()),
}
