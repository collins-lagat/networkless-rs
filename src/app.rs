use std::ops::ControlFlow;

use log::{error, info, warn};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    network::{
        devices::SpecificDevice,
        enums::{ActiveConnectionState, DeviceType, NmConnectivityState, NmState},
        network_manager::NetworkManager,
    },
    trays::{AirplaneModeState, Icon, TrayManager, TrayUpdate, VPNState, WifiState, WiredState},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Init,
    Update,
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
        Self {
            event_tx,
            action_tx,
            network_manager,
        }
    }

    pub async fn send_event(&self, event: Event) {
        self.event_tx.send(event).await.unwrap();
    }

    pub fn send_action_blocking(&self, action: Action) {
        self.action_tx.blocking_send(action).unwrap();
    }

    pub async fn run(
        &self,
        mut event_rx: Receiver<Event>,
        mut action_rx: Receiver<Action>,
        mut tray_manager: TrayManager,
    ) {
        let app = self.clone();
        tokio::spawn(async move {
            app.network_manager
                .listening_to_state_changes(async |_| {
                    app.send_event(Event::Update).await;
                })
                .await
                .unwrap();
        });

        let app = self.clone();
        tokio::spawn(async move {
            app.network_manager
                .listening_to_device_added(async |_| {
                    app.send_event(Event::Update).await;
                })
                .await
                .unwrap();
        });

        let app = self.clone();
        tokio::spawn(async move {
            app.network_manager
                .listening_to_device_removed(async |_| {
                    app.send_event(Event::Update).await;
                })
                .await
                .unwrap();
        });

        let app = self.clone();
        tokio::spawn(async move {
            let primary_connection = app.network_manager.primary_connection().await.unwrap();
            primary_connection
                .listening_to_state_changes(async |_| {
                    app.send_event(Event::Update).await;
                })
                .await
                .unwrap();
        });

        tokio::spawn(async move {
            while let Some(event) = action_rx.recv().await {
                println!("Action: {:?}", event);
            }
        });

        while let Some(event) = event_rx.recv().await {
            match event {
                Event::Init => {
                    if let ControlFlow::Break(_) = self.update(&mut tray_manager).await {
                        break;
                    }

                    continue;
                }
                Event::Update => {
                    if let ControlFlow::Break(_) = self.update(&mut tray_manager).await {
                        break;
                    }

                    continue;
                }
                Event::Shutdown => break,
            }
        }
    }

    async fn update(&self, tray_manager: &mut TrayManager) -> ControlFlow<()> {
        tray_manager.update(TrayUpdate::Wireless(None)).await;
        tray_manager.update(TrayUpdate::Wired(None)).await;
        tray_manager.update(TrayUpdate::Vpn(None)).await;

        let state = match self.network_manager.state().await {
            Ok(state) => state,
            Err(e) => {
                error!("Failed to get state: {}", e);
                return ControlFlow::Break(());
            }
        };

        info!("State: {:?}", state);

        let is_airplane_mode = match self.network_manager.airplane_mode_enabled().await {
            Ok(is_airplane_mode) => is_airplane_mode,
            Err(e) => {
                error!("Failed to get airplane mode: {}", e);
                return ControlFlow::Break(());
            }
        };

        if is_airplane_mode {
            tray_manager
                .update(TrayUpdate::AirplaneMode(AirplaneModeState { on: true }))
                .await;
        } else {
            tray_manager
                .update(TrayUpdate::AirplaneMode(AirplaneModeState { on: false }))
                .await;
        }

        match state {
            NmState::Unknown => {
                tray_manager.update(TrayUpdate::Icon(Icon::Unknown)).await;
                return ControlFlow::Continue(());
            }
            NmState::Asleep => {
                tray_manager.update(TrayUpdate::Icon(Icon::Off)).await;
                return ControlFlow::Continue(());
            }
            NmState::Connecting | NmState::Disconnecting => {
                tray_manager.update(TrayUpdate::Icon(Icon::Busy)).await;
                return ControlFlow::Continue(());
            }
            NmState::Disconnected => {
                tray_manager
                    .update(TrayUpdate::Icon(Icon::Disconnected))
                    .await;
                return ControlFlow::Continue(());
            }
            _ => {}
        };

        let connectivity = match self.network_manager.connectivity().await {
            Ok(connectivity) => connectivity,
            Err(e) => {
                error!("Failed to get connectivity: {}", e);
                return ControlFlow::Break(());
            }
        };

        info!("Connectivity: {:?}", connectivity);

        match connectivity {
            NmConnectivityState::Unknown => {
                tray_manager.update(TrayUpdate::Icon(Icon::Unknown)).await;
                return ControlFlow::Continue(());
            }
            NmConnectivityState::None => {
                tray_manager
                    .update(TrayUpdate::Icon(Icon::Disconnected))
                    .await;
                return ControlFlow::Continue(());
            }
            NmConnectivityState::Portal => {
                todo!("handle portal");
            }
            NmConnectivityState::Loss => {
                tray_manager.update(TrayUpdate::Icon(Icon::Limited)).await;
                return ControlFlow::Continue(());
            }
            _ => {}
        }

        let primary_connection = match self.network_manager.primary_connection().await {
            Ok(primary_connection) => primary_connection,
            Err(e) => {
                error!("Failed to get primary connection: {}", e);
                return ControlFlow::Break(());
            }
        };

        info!("Primary connection: {:?}", primary_connection.id().await);

        let devices = match primary_connection.devices().await {
            Ok(devices) => devices,
            Err(e) => {
                error!("Failed to get devices: {}", e);
                return ControlFlow::Break(());
            }
        };

        for device in devices {
            let device_type = match device.device_type().await {
                Ok(device_type) => device_type,
                Err(e) => {
                    warn!("Failed to get device type: {}", e);
                    continue;
                }
            };

            match device_type {
                DeviceType::Wifi => {
                    let wireless_device = match device.to_specific_device().await {
                        Some(SpecificDevice::Wireless(device)) => device,
                        _ => return ControlFlow::Break(()),
                    };

                    let active_access_point = wireless_device.active_access_point().await.unwrap();

                    let strength = active_access_point.strength().await.unwrap();

                    tray_manager
                        .update(TrayUpdate::Icon(Icon::Wifi(strength)))
                        .await;
                }
                DeviceType::Ethernet => {
                    tray_manager.update(TrayUpdate::Icon(Icon::Ethernet)).await;
                }
                DeviceType::TunTap => {
                    tray_manager.update(TrayUpdate::Icon(Icon::Tun)).await;
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

        let devices = match self.network_manager.devices().await {
            Ok(devices) => devices,
            Err(e) => {
                error!("Failed to get devices: {}", e);
                return ControlFlow::Break(());
            }
        };

        for device in devices {
            let device_type = match device.device_type().await {
                Ok(device_type) => device_type,
                Err(e) => {
                    warn!("Failed to get device type: {}", e);
                    continue;
                }
            };

            match device_type {
                DeviceType::Wifi => {
                    let wireless_device = match device.to_specific_device().await {
                        Some(SpecificDevice::Wireless(device)) => device,
                        _ => return ControlFlow::Break(()),
                    };

                    let configured_connections = device.available_connections().await.unwrap();
                    let futures = configured_connections
                        .iter()
                        .map(|setting| async { setting.id().await.unwrap() });

                    let known_connections = futures::future::join_all(futures).await;

                    let mut access_points = wireless_device.access_points().await.unwrap();
                    let futures = access_points
                        .iter_mut()
                        .map(|ap| async { ap.id().await.unwrap().into() });
                    let available_connections = futures::future::join_all(futures).await;
                    // TODO: sort access points by:
                    // 1. known connections
                    // 2. strength
                    // 3. name alphabetically

                    let active_connection = device.active_connection().await.unwrap();
                    match active_connection.state().await {
                        Ok(state) => {
                            let on = matches!(state, ActiveConnectionState::Activated);

                            tray_manager
                                .update(TrayUpdate::Wireless(Some(WifiState {
                                    on,
                                    known_connections,
                                    available_connections,
                                })))
                                .await;
                        }
                        Err(e) => {
                            warn!("WiFi: Failed to get active connection state: {}", e);
                        }
                    }
                }
                DeviceType::Ethernet => {
                    let active_connection = match device.active_connection().await {
                        Ok(active_connection) => active_connection,
                        Err(e) => {
                            error!("Failed to get active connection: {}", e);
                            return ControlFlow::Break(());
                        }
                    };

                    match active_connection.state().await {
                        Ok(state) => {
                            let on = matches!(state, ActiveConnectionState::Activated);
                            tray_manager
                                .update(TrayUpdate::Wired(Some(WiredState { on })))
                                .await;
                        }
                        Err(e) => {
                            warn!("Ethernet: Failed to get active connection state: {}", e);
                        }
                    };
                }
                DeviceType::WireGuard => {
                    let wire_guard_connection = device.active_connection().await.unwrap();
                    let wire_guard_connection_id = wire_guard_connection.id().await.unwrap();

                    let active_connection = device.active_connection().await.unwrap();

                    match active_connection.state().await {
                        Ok(state) => {
                            let on = matches!(state, ActiveConnectionState::Activated);

                            tray_manager
                                .update(TrayUpdate::Vpn(Some(VPNState {
                                    on,
                                    active_connection: wire_guard_connection_id.clone(),
                                })))
                                .await;
                        }
                        Err(e) => {
                            warn!("WireGuard: Failed to get active connection state: {}", e);
                        }
                    }
                }
                _ => {}
            }
        }
        ControlFlow::Continue(())
    }
}
