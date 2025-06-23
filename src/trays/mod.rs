use airplane_mode::AirplaneModeTray;
use anyhow::Result;
use image::GenericImageView;
use ksni::{Handle, TrayMethods};
use network::NetworkTray;
use vpn::VpnTray;

use crate::network::network_manager::NetworkManager;

mod airplane_mode;
mod network;
mod vpn;

pub struct TrayManager {
    nm: NetworkManager,
}

impl TrayManager {
    pub fn new(nm: NetworkManager) -> Self {
        Self { nm }
    }

    pub async fn spawn(
        &mut self,
    ) -> Result<(
        Handle<NetworkTray>,
        Handle<AirplaneModeTray>,
        Handle<VpnTray>,
    )> {
        let network_tray = NetworkTray::new(self.nm.clone());
        let airplane_mode_tray = AirplaneModeTray::new(self.nm.clone());
        let vpn_tray = VpnTray::new();

        let network_handle = network_tray.spawn().await?;
        let airplane_mode_handle = airplane_mode_tray.spawn().await?;
        let vpn_handle = vpn_tray.spawn().await?;

        Ok((network_handle, airplane_mode_handle, vpn_handle))
    }
}

pub fn get_icon_from_image_bytes(image_bytes: &[u8]) -> ksni::Icon {
    let img = image::load_from_memory_with_format(image_bytes, image::ImageFormat::Png)
        .expect("valid image");
    let (width, height) = img.dimensions();
    let mut data = img.into_rgba8().into_vec();
    assert_eq!(data.len() % 4, 0);
    for pixel in data.chunks_exact_mut(4) {
        pixel.rotate_right(1) // rgba to argb
    }
    ksni::Icon {
        width: width as i32,
        height: height as i32,
        data,
    }
}
