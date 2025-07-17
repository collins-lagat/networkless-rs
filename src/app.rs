use std::{ops::ControlFlow, time::Duration};

use anyhow::Result;
use futures::StreamExt;
use log::{error, info, warn};
use tokio::sync::mpsc::{Receiver, Sender};
use zbus::zvariant::{ObjectPath, OwnedObjectPath};

use crate::{
    network::{
        device::Device,
        devices::SpecificDevice,
        enums::{ActiveConnectionState, DeviceState, DeviceType, NmConnectivityState, NmState},
        network_manager::NetworkManager,
    },
    trays::{
        AirplaneModeState, Icon, TrayManager, TrayUpdate, VPNConnection, VPNState, WifiConnection,
        WifiState, WiredState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Init,
    Update,
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    ChangeAccessPoint(WifiConnection),
    ToggleWifi,
    ToggleWired,
    ToggleAirplaneMode,
    ToggleVPN(String),
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
        let handle = tokio::runtime::Handle::current();

        let app = self.clone();
        handle.spawn(async move {
            if let Err(e) = app.action_tx.send(action).await {
                error!("Failed to send action: {}", e);
            }
        });
    }

    pub async fn toggle_wifi(&self) {
        let on = self.network_manager.wifi_enabled().await.unwrap();

        match self.network_manager.set_wifi_enabled(!on).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to set wifi enabled: {}", e);
            }
        }

        self.send_event(Event::Update).await;
    }

    pub async fn toggle_wired(&self) {
        let devices = self.network_manager.devices().await.unwrap();

        for device in devices {
            if device.device_type().await.unwrap() != DeviceType::Ethernet {
                continue;
            }

            let on = matches!(device.state().await.unwrap(), DeviceState::Activated);

            if on {
                self.deactivate_wired_connection(&device).await;
            } else {
                self.activate_wired_connection(device).await;
            }
        }
    }

    async fn deactivate_wired_connection(&self, device: &Device) {
        let active_connection = device.active_connection().await.unwrap();
        let connection_path = OwnedObjectPath::from(active_connection.path());

        match self
            .network_manager
            .deactivate_connection(connection_path)
            .await
        {
            Ok(_) => {
                info!("Deactivated Ethernet device");
            }
            Err(e) => {
                error!("Failed to deactivate Ethernet device: {}", e);
            }
        };
    }

    async fn activate_wired_connection(&self, device: Device) {
        let available_connections = device.available_connections().await.unwrap();
        if available_connections.is_empty() {
            return;
        }

        for connection in available_connections {
            let device_path = device.path();
            let connection_path = OwnedObjectPath::from(connection.path());
            let activation_result = self
                .network_manager
                .activate_connection(
                    connection_path,
                    OwnedObjectPath::from(device_path.clone()),
                    OwnedObjectPath::from(ObjectPath::from_string_unchecked("/".into())),
                )
                .await;

            match activation_result {
                Ok(_) => {
                    info!("Activated Ethernet device: {}", device_path);
                }
                Err(e) => {
                    error!("Failed to activate Ethernet device: {}", e);
                }
            };
        }
    }

    pub async fn toggle_airplane_mode(&self) {
        let on = self.network_manager.airplane_mode_enabled().await.unwrap();

        match self.network_manager.set_airplane_mode_enabled(!on).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to set airplane mode: {}", e);
            }
        };

        self.send_event(Event::Update).await;
    }

    pub async fn toggle_vpn(&self, vpn: String) {
        let active_connections = self.network_manager.active_connections().await.unwrap();
        for active_connection in active_connections {
            if active_connection.device_type().await.unwrap() != DeviceType::WireGuard {
                continue;
            }

            let vpn_name = active_connection.id().await.unwrap();

            if vpn_name != vpn {
                continue;
            }

            let on = matches!(
                active_connection.state().await.unwrap(),
                ActiveConnectionState::Activated
            );

            if on {
                let active_connection_path = OwnedObjectPath::from(active_connection.path());
                self.network_manager
                    .deactivate_connection(active_connection_path)
                    .await
                    .unwrap();
            } else {
                let specific_object_path = active_connection.specific_object().await.unwrap();
                self.network_manager
                    .activate_connection(
                        OwnedObjectPath::from(ObjectPath::from_string_unchecked("/".into())),
                        OwnedObjectPath::from(ObjectPath::from_string_unchecked("/".into())),
                        specific_object_path,
                    )
                    .await
                    .unwrap();
            }
        }
    }

    pub async fn change_access_point(&self, access_point: WifiConnection) -> Result<()> {
        let mut device = None;

        for d in self.network_manager.all_devices().await.unwrap() {
            let device_type = d.device_type().await.unwrap();
            let state = d.state().await.unwrap();
            if device_type == DeviceType::Wifi && state == DeviceState::Activated {
                device = Some(d);
                break;
            }
        }

        let device = match device {
            Some(device) => device,
            None => {
                anyhow::bail!("No active Wifi device found");
            }
        };

        let wireless_device = match device.to_specific_device().await {
            Some(SpecificDevice::Wireless(device)) => device,
            _ => anyhow::bail!("Device is not a Wifi device"),
        };

        let mut access_points = wireless_device.access_points().await.unwrap_or_default();

        let mut is_associated_active_connection_already_active = false;

        for c in self
            .network_manager
            .active_connections()
            .await
            .unwrap_or_default()
        {
            for ap in &mut access_points {
                if ap.id().await.unwrap() == c.id().await.unwrap()
                    && ap.hw_address().await.unwrap_or_default() == access_point.hw_address
                    && c.state().await.unwrap() == ActiveConnectionState::Activated
                {
                    is_associated_active_connection_already_active = true;
                    break;
                }
            }

            if is_associated_active_connection_already_active {
                break;
            }
        }

        if is_associated_active_connection_already_active {
            warn!("Associated active connection is already active");
            return Ok(());
        }

        let mut found_access_point = None;
        for ap in &mut access_points {
            if ap.id().await.unwrap() == access_point.ssid
                && ap.hw_address().await.unwrap() == access_point.hw_address
            {
                found_access_point = Some(ap);
                break;
            }
        }

        let found_access_point = match found_access_point {
            Some(found_access_point) => found_access_point,
            None => {
                anyhow::bail!("Access point not found");
            }
        };

        let configured_connections = device.available_connections().await.unwrap_or_default();

        let mut configured_connection = None;

        for conf in &configured_connections {
            if conf.id().await.unwrap() == found_access_point.id().await.unwrap() {
                configured_connection = Some(conf);
                break;
            }
        }

        let configured_connection = match configured_connection {
            Some(configured_connection) => configured_connection,
            None => {
                anyhow::bail!("Configured connection not found");
            }
        };

        let configured_connection_path = OwnedObjectPath::from(configured_connection.path());
        let device_path = OwnedObjectPath::from(device.path());
        let access_point_path = OwnedObjectPath::from(found_access_point.path());

        info!(
            "configured_connection_path {:?}",
            configured_connection_path
        );
        info!("device_path {:?}", device_path);
        info!("access_point_path {:?}", access_point_path);

        self.network_manager
            .activate_connection(configured_connection_path, device_path, access_point_path)
            .await?;

        Ok(())
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
        let mut primary_connection_handle = tokio::spawn(async move {
            let primary_connection = app.network_manager.primary_connection().await.unwrap();
            primary_connection
                .listening_to_state_changes(async |_| {
                    app.send_event(Event::Update).await;
                })
                .await
                .unwrap();
        });

        let app = self.clone();
        tokio::spawn(async move {
            while let Some(action) = action_rx.recv().await {
                match action {
                    Action::ChangeAccessPoint(access_point) => {
                        if let Err(e) = app.change_access_point(access_point).await {
                            error!("Failed to change access point: {}", e);
                        };
                    }
                    Action::ToggleWifi => {
                        app.toggle_wifi().await;
                    }
                    Action::ToggleWired => {
                        app.toggle_wired().await;
                    }
                    Action::ToggleAirplaneMode => {
                        app.toggle_airplane_mode().await;
                    }
                    Action::ToggleVPN(vpn) => {
                        app.toggle_vpn(vpn).await;
                    }
                }
            }

            warn!("Action channel closed");
        });

        while let Some(event) = event_rx.recv().await {
            let app = self.clone();
            match event {
                Event::Init => {
                    if let ControlFlow::Break(_) = self.update(&mut tray_manager).await {
                        break;
                    }

                    continue;
                }
                Event::Update => {
                    primary_connection_handle.abort();
                    primary_connection_handle = tokio::spawn(async move {
                        let primary_connection =
                            app.network_manager.primary_connection().await.unwrap();
                        primary_connection
                            .listening_to_state_changes(async |_| {
                                app.send_event(Event::Update).await;
                            })
                            .await
                            .unwrap();
                    });

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
        tray_manager.update(TrayUpdate::AirplaneMode(None)).await;

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
                .update(TrayUpdate::AirplaneMode(Some(AirplaneModeState {
                    on: true,
                })))
                .await;
        } else {
            tray_manager
                .update(TrayUpdate::AirplaneMode(Some(AirplaneModeState {
                    on: false,
                })))
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
            }
            NmConnectivityState::None => {
                tray_manager
                    .update(TrayUpdate::Icon(Icon::Disconnected))
                    .await;
                return ControlFlow::Continue(());
            }
            NmConnectivityState::Loss => {
                tray_manager.update(TrayUpdate::Icon(Icon::Limited)).await;
                let app = self.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    app.send_event(Event::Update).await;
                });
            }
            NmConnectivityState::Full => {}
            _ => {
                tray_manager.update(TrayUpdate::Icon(Icon::Unknown)).await;
                return ControlFlow::Continue(());
            }
        }

        if matches!(connectivity, NmConnectivityState::Full) {
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

                        let active_access_point =
                            wireless_device.active_access_point().await.unwrap();

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
                    _ => {
                        tray_manager.update(TrayUpdate::Icon(Icon::Unknown)).await;
                    }
                }
            }
        }

        let devices = match self.network_manager.all_devices().await {
            Ok(devices) => devices,
            Err(e) => {
                error!("Failed to get devices: {}", e);
                return ControlFlow::Break(());
            }
        };

        let mut vpn_connections = Vec::<VPNConnection>::new();

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
                    let state = device.state().await.unwrap();

                    match state {
                        DeviceState::Activated => {
                            let wireless_device = match device.to_specific_device().await {
                                Some(SpecificDevice::Wireless(device)) => device,
                                _ => return ControlFlow::Break(()),
                            };

                            let configured_connections =
                                device.available_connections().await.unwrap();
                            let futures = configured_connections
                                .iter()
                                .map(|setting| async { setting.id().await.unwrap() });

                            let ssids_of_known_connections =
                                futures::future::join_all(futures).await;

                            let mut access_points = wireless_device.access_points().await.unwrap();
                            let futures = access_points.iter_mut().map(|ap| async {
                                WifiConnection {
                                    ssid: ap.id().await.unwrap().into(),
                                    hw_address: ap.hw_address().await.unwrap(),
                                }
                            });
                            let available_connections = futures::future::join_all(futures).await;
                            // TODO: sort access points by:
                            // 1. known connections
                            // 2. strength
                            // 3. name alphabetically

                            let known_connections = ssids_of_known_connections
                                .into_iter()
                                .map(|ssid| {
                                    match available_connections
                                        .iter()
                                        .find(|connection| connection.ssid == ssid)
                                    {
                                        Some(connection) => connection.clone(),
                                        None => WifiConnection {
                                            ssid,
                                            hw_address: "".into(),
                                        },
                                    }
                                })
                                .collect::<Vec<WifiConnection>>();

                            let known_connections = known_connections
                                .into_iter()
                                .filter(|connection| !connection.hw_address.is_empty())
                                .collect::<Vec<WifiConnection>>();

                            tray_manager
                                .update(TrayUpdate::Wireless(Some(WifiState {
                                    on: true,
                                    known_connections,
                                    available_connections,
                                })))
                                .await;
                        }
                        DeviceState::Disconnected => {
                            let on = self.network_manager.wifi_enabled().await.unwrap();
                            tray_manager
                                .update(TrayUpdate::Wireless(Some(WifiState {
                                    on,
                                    known_connections: vec![],
                                    available_connections: vec![],
                                })))
                                .await;

                            if on {
                                info!("Waiting for wireless device to activate...");
                                let mut state_change_signal =
                                    device.receive_state_changed_signal().await.unwrap();

                                let mut attempts = 0;
                                let max_attempts = 20;

                                loop {
                                    attempts += 1;

                                    if attempts > max_attempts {
                                        break;
                                    }

                                    info!("Attempt {}/{}", attempts, max_attempts);

                                    if let Ok(Some(state_changed)) = tokio::time::timeout(
                                        Duration::from_secs(20),
                                        state_change_signal.next(),
                                    )
                                    .await
                                    {
                                        let state = match state_changed.args() {
                                            Ok(state) => DeviceState::from(state.new_state),
                                            Err(_) => {
                                                error!("Failed to get wireless device state");
                                                break;
                                            }
                                        };

                                        if matches!(state, DeviceState::Activated) {
                                            info!("Wireless device activated!");
                                            self.send_event(Event::Update).await;
                                            break;
                                        }

                                        warn!("Wireless device not activated yet. Trying again...");
                                    }
                                }
                            }
                        }
                        DeviceState::Unavailable => {
                            // Often happens when the wifi is disabled or airplane mode is on
                            info!("Wireless Device state: Unavailable");
                            tray_manager
                                .update(TrayUpdate::Wireless(Some(WifiState {
                                    on: false,
                                    known_connections: vec![],
                                    available_connections: vec![],
                                })))
                                .await;
                        }
                        _ => {}
                    }
                }
                DeviceType::Ethernet => match device.state().await.unwrap() {
                    DeviceState::Activated => {
                        tray_manager
                            .update(TrayUpdate::Wired(Some(WiredState { on: true })))
                            .await;
                    }
                    DeviceState::Disconnected => {
                        tray_manager
                            .update(TrayUpdate::Wired(Some(WiredState { on: false })))
                            .await;
                    }

                    _ => {}
                },
                DeviceType::WireGuard => {
                    let wire_guard_connection = device.active_connection().await.unwrap();
                    let wire_guard_connection_id = wire_guard_connection.id().await.unwrap();
                    let state = match device.state().await {
                        Ok(state) => state,
                        Err(e) => {
                            error!("Failed to get device state: {}", e);
                            return ControlFlow::Break(());
                        }
                    };

                    match state {
                        DeviceState::Activated => {
                            let vpn_connection = VPNConnection {
                                name: wire_guard_connection_id.clone(),
                                on: true,
                            };
                            vpn_connections.push(vpn_connection);
                        }
                        DeviceState::Disconnected | DeviceState::Unavailable => {
                            let vpn_connection = VPNConnection {
                                name: wire_guard_connection_id.clone(),
                                on: false,
                            };

                            vpn_connections.push(vpn_connection);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if !vpn_connections.is_empty() {
            tray_manager
                .update(TrayUpdate::Vpn(Some(VPNState {
                    connections: vpn_connections,
                })))
                .await;
        }

        ControlFlow::Continue(())
    }
}
