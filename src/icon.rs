use std::thread;

use anyhow::{Context, Result, bail};
use futures::StreamExt;
use futures::channel::mpsc::{UnboundedSender, unbounded};
use log::error;
use tray_icon::{Icon, TrayIconBuilder, menu::Menu};

use crate::event::Event;

const ICON_BYTES: &[u8] = include_bytes!("../assets/checking.png");

pub struct TrayIcon {
    tx: UnboundedSender<Event>,
}

impl TrayIcon {
    pub fn new() -> Self {
        let (tx, mut rx) = unbounded();

        let _tx = tx.clone();
        thread::spawn(move || {
            gtk::init().unwrap();
            let menu = Menu::new();

            let tray_icon = match TrayIconBuilder::new().with_menu(Box::new(menu)).build() {
                Ok(tray_icon) => tray_icon,
                Err(e) => {
                    error!("Failed to create tray icon: {}", e);
                    return;
                }
            };

            let ctx = gtk::glib::MainContext::default();
            ctx.spawn_local(async move {
                while let Some(event) = rx.next().await {
                    match event {
                        Event::Unknown => {
                            if set_icon(&tray_icon, ICON_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::Off => {
                            if set_icon(&tray_icon, ICON_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::Busy => {
                            if set_icon(&tray_icon, ICON_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::Disconnected => {
                            if set_icon(&tray_icon, ICON_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::AirplaneMode => {
                            if set_icon(&tray_icon, ICON_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::Limited => {
                            if set_icon(&tray_icon, ICON_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::Vpn => {
                            if set_icon(&tray_icon, ICON_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::Ethernet => {
                            if set_icon(&tray_icon, ICON_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::Wifi(strength) => {
                            if strength < 30 && set_icon(&tray_icon, ICON_BYTES).is_err() {
                                error!("Failed to set icon");
                            } else if strength < 50 && set_icon(&tray_icon, ICON_BYTES).is_err() {
                                error!("Failed to set icon");
                            } else if set_icon(&tray_icon, ICON_BYTES).is_err() {
                                error!("Failed to set icon");
                            }
                        }
                        Event::Shutdown => {
                            break;
                        }
                    };

                    gtk::glib::timeout_future_seconds(1).await;
                }
            });

            gtk::main();
        });

        Self { tx }
    }

    pub async fn send(&mut self, event: Event) {
        let _ = self.tx.unbounded_send(event);
    }
}

fn set_icon(tray_icon: &tray_icon::TrayIcon, icon_bytes: &[u8]) -> Result<()> {
    let icon = match convert_bytes_to_icon(icon_bytes) {
        Ok(icon) => icon,
        Err(e) => {
            bail!("Failed to create icon: {}", e);
        }
    };
    if let Err(e) = tray_icon.set_icon(Some(icon)) {
        bail!("Failed to set icon: {}", e);
    }

    Ok(())
}

fn convert_bytes_to_icon(bytes: &[u8]) -> Result<Icon> {
    let image_buff = match image::load_from_memory(bytes) {
        Ok(image_dyn) => image_dyn.into_rgba8(),
        Err(e) => return Err(e).context("Failed to load icon"),
    };

    let (width, height) = image_buff.dimensions();
    let icon_rgba = image_buff.into_raw();

    let icon = match Icon::from_rgba(icon_rgba, width, height) {
        Ok(icon) => icon,
        Err(e) => return Err(e).context("Failed to create icon"),
    };

    Ok(icon)
}
