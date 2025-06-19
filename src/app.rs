use std::{ops::ControlFlow, time::Duration};

use anyhow::Result;
use futures::future::join_all;
use ksni::Handle;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    interfaces::{access_point::AccessPointProxy, devices::wireless::WirelessProxy},
    network::{
        device::Device,
        enums::{DeviceType, NmConnectivityState, NmState},
        network_manager::NetworkManager,
    },
    tray::{Icon, Tray, VPNState},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Tick,
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    ChangeAccessPoint(String),
    ToggleWifi,
    ToggleWired,
    ToggleAirplaneMode,
    ToggleVPN,
}

#[derive(Debug, Clone)]
pub struct App {
    event_tx: Sender<Event>,
    action_tx: Sender<Action>,
    network_manager: NetworkManager,
}

impl App {
    pub fn new(
        event_tx: Sender<Event>,
        action_tx: Sender<Action>,
        network_manager: NetworkManager,
    ) -> Self {
        let actor = AppTicker::new(event_tx.clone());
        tokio::spawn(async move { actor.run().await });

        Self {
            event_tx,
            action_tx,
            network_manager,
        }
    }

    pub async fn send_event(&self, event: Event) {
        self.event_tx.send(event).await.unwrap();
    }

    pub async fn send_action(&self, action: Action) {
        self.action_tx.send(action).await.unwrap();
    }

    pub fn send_action_blocking(&self, action: Action) {
        self.action_tx.blocking_send(action).unwrap();
    }

    pub async fn run(
        &self,
        mut event_rx: Receiver<Event>,
        mut action_rx: Receiver<Action>,
        tray_handle: Handle<Tray>,
    ) {
        let action_tray_handle = tray_handle.clone();
        tokio::spawn(async move {
            while let Some(event) = action_rx.recv().await {
                println!("Action: {:?}", event);
            }
        });

        while let Some(event) = event_rx.recv().await {
            match event {
                Event::Tick => {
                    if let ControlFlow::Break(_) = self.update_tray_icon(&tray_handle).await {
                        break;
                    }

                    continue;
                }
                Event::Shutdown => break,
            }
        }
    }

    async fn update_tray_icon(&self, tray_handle: &Handle<Tray>) -> ControlFlow<()> {
        let update_tray_icon_helper = async |icon: Icon| {
            tray_handle
                .update(move |tray| {
                    tray.set_icon(icon);
                })
                .await;
        };
        let state = match self.network_manager.state().await {
            Ok(state) => state,
            Err(e) => {
                println!("Failed to get state: {}", e);
                return ControlFlow::Break(());
            }
        };
        let is_airplane_mode = match self.network_manager.airplane_mode_enabled().await {
            Ok(is_airplane_mode) => is_airplane_mode,
            Err(e) => {
                println!("Failed to get airplane mode: {}", e);
                return ControlFlow::Break(());
            }
        };
        if is_airplane_mode {
            update_tray_icon_helper(Icon::AirplaneMode).await;
            return ControlFlow::Break(());
        }
        match state {
            NmState::Asleep => {
                update_tray_icon_helper(Icon::Off).await;
                return ControlFlow::Break(());
            }
            NmState::Disconnected => {
                update_tray_icon_helper(Icon::Disconnected).await;
                return ControlFlow::Break(());
            }
            _ => {}
        };
        let connectivity = match self.network_manager.connectivity().await {
            Ok(connectivity) => connectivity,
            Err(e) => {
                println!("Failed to get connectivity: {}", e);
                return ControlFlow::Break(());
            }
        };
        match connectivity {
            NmConnectivityState::Unknown => {
                update_tray_icon_helper(Icon::Unknown).await;
                return ControlFlow::Break(());
            }
            NmConnectivityState::None => {
                update_tray_icon_helper(Icon::Disconnected).await;
                return ControlFlow::Break(());
            }
            NmConnectivityState::Portal => {
                todo!("handle portal");
            }
            NmConnectivityState::Loss => {
                update_tray_icon_helper(Icon::Limited).await;
                return ControlFlow::Break(());
            }
            _ => {}
        }
        let primary_connection = match self.network_manager.primary_connection().await {
            Ok(primary_connection) => primary_connection,
            Err(e) => {
                println!("Failed to get primary connection: {}", e);
                return ControlFlow::Break(());
            }
        };
        let primary_connection_id = primary_connection.id().await.unwrap();
        let devices = match primary_connection.devices().await {
            Ok(devices) => devices,
            Err(e) => {
                println!("Failed to get devices: {}", e);
                return ControlFlow::Break(());
            }
        };
        for device in devices {
            let device_type = match device.device_type().await {
                Ok(device_type) => device_type,
                Err(e) => {
                    println!("Failed to get device type: {}", e);
                    continue;
                }
            };

            match device_type {
                DeviceType::Wifi => {
                    device
                        .with_connection_and_path(async |connection, path| {
                            let wireless_device = WirelessProxy::builder(connection)
                                .path(path)
                                .unwrap()
                                .build()
                                .await
                                .unwrap();

                            let active_access_point_path =
                                wireless_device.active_access_point().await.unwrap();

                            let active_access_point = AccessPointProxy::builder(connection)
                                .path(active_access_point_path)
                                .unwrap()
                                .build()
                                .await
                                .unwrap();

                            let strength = active_access_point.strength().await.unwrap();

                            update_tray_icon_helper(Icon::Wifi(strength)).await;

                            Ok(())
                        })
                        .await
                        .unwrap();
                }
                DeviceType::Ethernet => {
                    update_tray_icon_helper(Icon::Ethernet).await;
                }
                DeviceType::TunTap => {
                    update_tray_icon_helper(Icon::Tun).await;
                }
                DeviceType::Bluetooth => {
                    todo!("support bluetooth in future");
                }
                DeviceType::Modem => {
                    todo!("support modem in future");
                }
                _ => {}
            }
        }
        let devices = match self.network_manager.all_devices().await {
            Ok(devices) => devices,
            Err(e) => {
                println!("Failed to get devices: {}", e);
                return ControlFlow::Break(());
            }
        };
        let futures = devices.iter().map(async |device: &Device| {
            let device_type = match device.device_type().await {
                Ok(device_type) => device_type,
                Err(e) => {
                    println!("Failed to get device type: {}", e);
                    DeviceType::Unknown
                }
            };
            (matches!(device_type, DeviceType::WireGuard), device.clone())
        });
        let results = join_all(futures).await;
        let (has_wireguard_device, wireguard_device) = results
            .iter()
            .reduce(|acc, result| if result.0 { result } else { acc })
            .unwrap();

        if *has_wireguard_device {
            let wire_guard_connection = match wireguard_device.active_connection().await {
                Ok(connection) => connection,
                Err(e) => {
                    println!("Failed to get wireguard connection: {}", e);
                    return ControlFlow::Break(());
                }
            };

            let wire_guard_connection_id = wire_guard_connection.id().await.unwrap();

            tray_handle
                .update(|tray| {
                    tray.set_vpn_state(Some(VPNState {
                        on: true,
                        active_connection: wire_guard_connection_id.clone(),
                    }));
                })
                .await;
        } else {
            tray_handle.update(|tray| tray.set_vpn_state(None)).await;
        }
        ControlFlow::Continue(())
    }
}

struct AppTicker {
    tx: Sender<Event>,
}

impl AppTicker {
    fn new(tx: Sender<Event>) -> Self {
        Self { tx }
    }

    async fn run(&self) -> Result<()> {
        let tick_rate = Duration::from_secs(1);
        let mut interval = tokio::time::interval(tick_rate);
        loop {
            interval.tick().await;
            self.tx.send(Event::Tick).await?;
        }
    }
}
