use std::collections::HashMap;

use anyhow::Result as AnyResult;
use futures::StreamExt;
use log::info;
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

    pub async fn listening_to_access_point_added<F>(&self, f: F) -> AnyResult<()>
    where
        F: AsyncFnOnce(AccessPoint) -> () + Send + Copy,
    {
        let mut stream = self.wireless_device.receive_access_point_added().await?;

        while let Some(access_point_added) = stream.next().await {
            info!("Access Point added");

            let access_point_path = match access_point_added.args() {
                Ok(state) => state.access_point.to_owned(),
                Err(e) => {
                    anyhow::bail!("Failed to get AccessPointAdded arguments: {e}");
                }
            };

            let access_point_proxy =
                AccessPointProxy::builder(self.wireless_device.inner().connection())
                    .path(&access_point_path)?
                    .build()
                    .await?;

            let access_point = AccessPoint::new(access_point_proxy);

            f(access_point).await;
        }
        Ok(())
    }

    pub async fn listening_to_access_point_removed<F>(&self, f: F) -> AnyResult<()>
    where
        F: AsyncFnOnce(AccessPoint) -> () + Send + Copy,
    {
        let mut stream = self.wireless_device.receive_access_point_removed().await?;

        while let Some(access_point_removed) = stream.next().await {
            info!("Access Point removed");

            let access_point_path = match access_point_removed.args() {
                Ok(state) => state.access_point.to_owned(),
                Err(e) => {
                    anyhow::bail!("Failed to get AccessPointRemoved arguments: {e}");
                }
            };

            let access_point_proxy =
                AccessPointProxy::builder(self.wireless_device.inner().connection())
                    .path(&access_point_path)?
                    .build()
                    .await?;

            let access_point = AccessPoint::new(access_point_proxy);

            f(access_point).await;
        }
        Ok(())
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
