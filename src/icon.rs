use std::thread;

use anyhow::{Context, Result};
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

            let icon = match convert_bytes_to_icon(ICON_BYTES) {
                Ok(icon) => icon,
                Err(e) => {
                    error!("Failed to create icon: {}", e);
                    return;
                }
            };

            let menu = Menu::new();

            let tray_icon = match TrayIconBuilder::new()
                .with_icon(icon)
                .with_menu(Box::new(menu))
                .build()
            {
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
                        Event::WifiEnabled(enabled) => {
                            tray_icon.set_title(Some("Wifi Enabled"));
                        }
                        Event::AirplaneMode(enabled) => {
                            tray_icon.set_title(Some("Airplane Mode"));
                        }
                        _ => {}
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
