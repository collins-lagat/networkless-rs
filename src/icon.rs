use std::thread;

use anyhow::{Context, Result, bail};
use futures::StreamExt;
use futures::channel::mpsc::{UnboundedSender, unbounded};
use log::error;
use tray_icon::menu::{MenuEvent, MenuItem, Submenu};
use tray_icon::{Icon, TrayIconBuilder, menu::Menu};

use crate::event::{Action, Event};

const AIRPLANE_MODE_BYTES: &[u8] = include_bytes!("../assets/airplane_mode.png");
const BUSY_BYTES: &[u8] = include_bytes!("../assets/busy.png");
const DISCONNECTED_BYTES: &[u8] = include_bytes!("../assets/disconnected.png");
const ETHERNET_BYTES: &[u8] = include_bytes!("../assets/ethernet.png");
const LIMITED_BYTES: &[u8] = include_bytes!("../assets/limited.png");
const UNKNOWN_BYTES: &[u8] = include_bytes!("../assets/unknown.png");
const VPN_BYTES: &[u8] = include_bytes!("../assets/vpn.png");
const WIFI_OFF_BYTES: &[u8] = include_bytes!("../assets/wifi-off.png");
const WIFI_100_BYTES: &[u8] = include_bytes!("../assets/wifi-100.png");
const WIFI_75_BYTES: &[u8] = include_bytes!("../assets/wifi-75.png");
const WIFI_50_BYTES: &[u8] = include_bytes!("../assets/wifi-50.png");
const WIFI_25_BYTES: &[u8] = include_bytes!("../assets/wifi-25.png");
const WIFI_0_BYTES: &[u8] = include_bytes!("../assets/wifi-0.png");

pub struct TrayIcon {
    tx: UnboundedSender<Event>,
}

impl TrayIcon {
    pub fn new() -> Self {
        let (tx, mut rx) = unbounded::<Event>();

        let (local_action_sender, mut local_action_receiver) = unbounded::<Action>();
        thread::spawn(move || {
            gtk::init().unwrap();
            let menu = Menu::new();

            let wired_toggle_menu_item =
                MenuItem::with_id("wired_toggle_menu_item", "OFF", true, None);

            let wired_submenu = Submenu::with_id_and_items(
                "wired_submenu",
                "Wired: OFF",
                true,
                &[&wired_toggle_menu_item],
            )
            .unwrap();

            let wifi_toggle_menu_item =
                MenuItem::with_id("wifi_toggle_menu_item", "OFF", true, None);

            let wifi_submenu = Submenu::with_id_and_items(
                "wifi_submenu",
                "Wifi: OFF",
                true,
                &[&wifi_toggle_menu_item],
            )
            .unwrap();

            let airplane_mode_toggle_menu_item =
                MenuItem::with_id("airplane_mode_toggle_menu_item", "OFF", true, None);

            let airplane_mode_submenu = Submenu::with_id_and_items(
                "airplane_mode_submenu",
                "Airplane Mode: OFF",
                true,
                &[&airplane_mode_toggle_menu_item],
            )
            .unwrap();

            if let Err(e) = menu.append_items(&[
                &wired_submenu,
                &tray_icon::menu::PredefinedMenuItem::separator(),
                &wifi_submenu,
                &tray_icon::menu::PredefinedMenuItem::separator(),
                &airplane_mode_submenu,
            ]) {
                error!("Failed to append menu item: {}", e);
                return;
            };

            let tray_icon = match TrayIconBuilder::new()
                .with_id("networkless")
                .with_menu(Box::new(menu))
                .build()
            {
                Ok(tray_icon) => tray_icon,
                Err(e) => {
                    error!("Failed to create tray icon: {}", e);
                    return;
                }
            };

            MenuEvent::set_event_handler(Some(move |event: MenuEvent| match event.id().as_ref() {
                "wired_toggle_menu_item" => {
                    local_action_sender
                        .unbounded_send(Action::ToggleWired)
                        .unwrap();
                }
                "wifi_toggle_menu_item" => {
                    local_action_sender
                        .unbounded_send(Action::ToggleWifi)
                        .unwrap();
                }
                "airplane_mode_toggle_menu_item" => {
                    local_action_sender
                        .unbounded_send(Action::ToggleAirplaneMode)
                        .unwrap();
                }
                _ => {}
            }));

            let ctx = gtk::glib::MainContext::default();

            ctx.spawn_local(async move {
                while let Some(action) = local_action_receiver.next().await {
                    match action {
                        Action::ToggleWired => {
                            if wired_toggle_menu_item.text() == "OFF" {
                                wired_submenu.set_text("Wired: ON");
                                wired_toggle_menu_item.set_text("ON");
                                println!("Turning wired on");
                            } else {
                                wired_submenu.set_text("Wired: OFF");
                                wired_toggle_menu_item.set_text("OFF");
                                println!("Turning wired off");
                            }
                        }
                        Action::ToggleWifi => {
                            if wifi_toggle_menu_item.text() == "OFF" {
                                wifi_submenu.set_text("Wifi: ON");
                                wifi_toggle_menu_item.set_text("ON");
                                println!("Turning wifi on");
                            } else {
                                wifi_submenu.set_text("Wifi: OFF");
                                wifi_toggle_menu_item.set_text("OFF");
                                println!("Turning wifi off");
                            }
                        }
                        Action::ToggleAirplaneMode => {
                            if airplane_mode_toggle_menu_item.text() == "OFF" {
                                airplane_mode_submenu.set_text("Airplane Mode: ON");
                                airplane_mode_toggle_menu_item.set_text("ON");
                                println!("Turning airplane mode on");
                            } else {
                                airplane_mode_submenu.set_text("Airplane Mode: OFF");
                                airplane_mode_toggle_menu_item.set_text("OFF");
                                println!("Turning airplane mode off");
                            }
                        }
                        _ => {}
                    }
                }
            });

            ctx.spawn_local(async move {
                let mut current_event: Option<Event> = None;
                let mut counter = 0;

                while let Some(event) = rx.next().await {
                    if current_event == Some(event.clone()) && counter == 2 {
                        continue;
                    }

                    if current_event == Some(event.clone()) {
                        counter += 1;
                    } else {
                        counter = 0;
                    }

                    current_event = Some(event.clone());

                    match event {
                        Event::Unknown => {
                            if set_icon(&tray_icon, UNKNOWN_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::Off => {
                            if set_icon(&tray_icon, WIFI_OFF_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::Busy => {
                            if set_icon(&tray_icon, BUSY_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::Disconnected => {
                            if set_icon(&tray_icon, DISCONNECTED_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::AirplaneMode => {
                            if set_icon(&tray_icon, AIRPLANE_MODE_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::Limited => {
                            if set_icon(&tray_icon, LIMITED_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::Vpn => {
                            if set_icon(&tray_icon, VPN_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::Ethernet => {
                            if set_icon(&tray_icon, ETHERNET_BYTES).is_err() {
                                error!("Failed to set icon");
                            };
                        }
                        Event::Wifi(strength) => {
                            // https://gitlab.gnome.org/GNOME/gnome-shell/-/blob/main/js/ui/status/network.js?ref_type=heads#L46-L57

                            let icon = match strength {
                                0..=19 => WIFI_0_BYTES,
                                20..=39 => WIFI_25_BYTES,
                                40..=49 => WIFI_50_BYTES,
                                50..=79 => WIFI_75_BYTES,
                                80..=100 => WIFI_100_BYTES,
                                _ => WIFI_100_BYTES,
                            };

                            if set_icon(&tray_icon, icon).is_err() {
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
