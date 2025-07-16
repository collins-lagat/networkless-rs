use airplane_mode_tray::AirplaneModeTray;
use image::GenericImageView;
use ksni::{Handle, TrayMethods};
use log::error;
use network_tray::NetworkTray;
use vpn_tray::VpnTray;

use crate::app::App;

mod airplane_mode_tray;
mod network_tray;
mod vpn_tray;
pub use network_tray::{
    AirplaneModeState, Icon, VPNConnection, VPNState, WifiConnection, WifiState, WiredState,
};

pub enum TrayUpdate {
    Icon(Icon),
    Wireless(Option<WifiState>),
    Wired(Option<WiredState>),
    Vpn(Option<VPNState>),
    AirplaneMode(Option<AirplaneModeState>),
}

pub struct TrayManager {
    app: App,
    network_tray_handle: Option<Handle<NetworkTray>>,
    vpn_tray_handle: Option<Handle<VpnTray>>,
    airplane_mode_tray_handle: Option<Handle<AirplaneModeTray>>,
}

impl TrayManager {
    pub fn new(app: App) -> Self {
        Self {
            app,
            network_tray_handle: None,
            vpn_tray_handle: None,
            airplane_mode_tray_handle: None,
        }
    }

    pub async fn update(&mut self, state: TrayUpdate) {
        match state {
            TrayUpdate::Icon(icon) => self.update_icon(icon).await,
            TrayUpdate::Wireless(state) => self.update_wireless(state).await,
            TrayUpdate::Wired(state) => self.update_wired(state).await,
            TrayUpdate::Vpn(state) => self.update_vpn(state).await,
            TrayUpdate::AirplaneMode(state) => self.update_airplane_mode(state).await,
        };
    }

    async fn create_network_tray(&mut self) {
        if self.network_tray_handle.is_some() {
            return;
        }
        let network_tray = NetworkTray::new(self.app.clone());
        match network_tray.spawn().await {
            Ok(handle) => self.network_tray_handle = Some(handle),
            Err(e) => error!("Error creating network tray: {:?}", e),
        };
    }

    async fn create_vpn_tray(&mut self) {
        if self.vpn_tray_handle.is_some() {
            return;
        }
        let vpn_tray = VpnTray::new(self.app.clone());
        let handle = vpn_tray.spawn().await.unwrap();
        self.vpn_tray_handle = Some(handle);
    }

    async fn create_airplane_mode_tray(&mut self) {
        if self.airplane_mode_tray_handle.is_some() {
            return;
        }
        let airplane_mode_tray = AirplaneModeTray::new(self.app.clone());
        let handle = airplane_mode_tray.spawn().await.unwrap();
        self.airplane_mode_tray_handle = Some(handle);
    }

    async fn update_icon(&mut self, icon: Icon) {
        if self.network_tray_handle.is_none() {
            self.create_network_tray().await;
        }

        if let Some(network_tray_handle) = &mut self.network_tray_handle {
            network_tray_handle
                .update(|tray| {
                    tray.set_icon(icon);
                })
                .await;
        }
    }

    async fn update_wireless(&mut self, state: Option<WifiState>) {
        if self.network_tray_handle.is_none() {
            self.create_network_tray().await;
        }

        if let Some(network_tray_handle) = &mut self.network_tray_handle {
            network_tray_handle
                .update(|tray| {
                    tray.set_wifi_state(state);
                })
                .await;
        }
    }

    async fn update_wired(&mut self, state: Option<WiredState>) {
        if self.network_tray_handle.is_none() {
            self.create_network_tray().await;
        }

        if let Some(network_tray_handle) = &mut self.network_tray_handle {
            network_tray_handle
                .update(|tray| {
                    tray.set_wired_state(state);
                })
                .await;
        }
    }

    async fn update_vpn(&mut self, state: Option<VPNState>) {
        if self.network_tray_handle.is_none() {
            self.create_network_tray().await;
        }

        if let Some(network_tray_handle) = &mut self.network_tray_handle {
            network_tray_handle
                .update(|tray| {
                    tray.set_vpn_state(state.clone());
                })
                .await;
        }

        if state.is_none() && self.vpn_tray_handle.is_none() {
            return;
        }

        if state.is_none() && self.vpn_tray_handle.is_some() {
            self.vpn_tray_handle.as_mut().unwrap().shutdown();
            self.vpn_tray_handle = None;
            return;
        }

        if self.vpn_tray_handle.is_none() {
            self.create_vpn_tray().await;
        }

        if let Some(vpn_tray_handle) = &mut self.vpn_tray_handle {
            vpn_tray_handle
                .update(|tray| {
                    tray.set_state(state);
                })
                .await;
        }
    }

    async fn update_airplane_mode(&mut self, state: Option<AirplaneModeState>) {
        if self.network_tray_handle.is_none() {
            self.create_network_tray().await;
        }

        if let Some(network_tray_handle) = &mut self.network_tray_handle {
            network_tray_handle
                .update(|tray| {
                    tray.set_airplane_mode_state(state.clone());
                })
                .await;
        }

        if let Some(state) = state {
            if state.on {
                self.create_airplane_mode_tray().await;
                return;
            }
        }

        if self.airplane_mode_tray_handle.is_some() {
            self.airplane_mode_tray_handle.as_mut().unwrap().shutdown();
            self.airplane_mode_tray_handle = None;
        }
    }
}

fn get_icon_from_image_bytes(image_bytes: &[u8]) -> ksni::Icon {
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
