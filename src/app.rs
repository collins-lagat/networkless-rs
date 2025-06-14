use std::time::Duration;

use anyhow::Result;
use futures::future::join_all;
use ksni::Handle;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    network::{
        device::Device,
        enums::{DeviceType, NmConnectivityState, NmState},
        network_manager::NetworkManager,
    },
    tray::{Icon, Tray},
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
                    let state = match self.network_manager.state().await {
                        Ok(state) => state,
                        Err(e) => {
                            println!("Failed to get state: {}", e);
                            continue;
                        }
                    };

                    match state {
                        NmState::Asleep => {
                            // check airplane mode, otherwise show sleep icon
                            continue;
                        }
                        NmState::Disconnected => {
                            // check airplane mode, otherwise show disconnected icon
                            continue;
                        }
                        NmState::Disconnecting | NmState::Connecting => {
                            // show network busy icon then continue
                            continue;
                        }
                        _ => {}
                    };

                    let connectivity = match self.network_manager.connectivity().await {
                        Ok(connectivity) => connectivity,
                        Err(e) => {
                            println!("Failed to get connectivity: {}", e);
                            continue;
                        }
                    };

                    match connectivity {
                        NmConnectivityState::Unknown => {
                            // show unknown connectivity icon then continue
                            continue;
                        }
                        NmConnectivityState::None => {
                            // show no connectivity icon then continue
                            continue;
                        }
                        NmConnectivityState::Portal => {
                            todo!("handle portal");
                        }
                        NmConnectivityState::Loss => {
                            // show limited connectivity icon then continue
                            continue;
                        }
                        _ => {}
                    }

                    let primary_connection = match self.network_manager.primary_connection().await {
                        Ok(primary_connection) => primary_connection,
                        Err(e) => {
                            println!("Failed to get primary connection: {}", e);
                            continue;
                        }
                    };

                    let devices = match primary_connection.devices().await {
                        Ok(devices) => devices,
                        Err(e) => {
                            println!("Failed to get devices: {}", e);
                            continue;
                        }
                    };

                    let device_icons: Vec<Icon> = Vec::with_capacity(2);

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
                                // TODO: get device path
                                // TODO: get access point from wireless device
                                // TODO: get strength from wireless device
                                // TODO: show wifi icon
                            }
                            DeviceType::Ethernet => {
                                // TODO: show ethernet icon
                            }
                            DeviceType::TunTap => {
                                // TODO: show tuntap icon
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
                            continue;
                        }
                    };

                    let futures = devices.iter().map(async |device: &Device| {
                        let device_type = match device.device_type().await {
                            Ok(device_type) => device_type,
                            Err(e) => {
                                println!("Failed to get device type: {}", e);
                                return false;
                            }
                        };

                        matches!(device_type, DeviceType::WireGuard)
                    });

                    let results = join_all(futures).await;

                    let has_wireguard_device = results.iter().any(|result| *result);

                    if has_wireguard_device {
                        // TODO: show wireguard icon
                    }
                }
                Event::Shutdown => break,
            }
        }
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
