use zbus::proxy;

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
pub trait NetworkManager {
    // Methods

    fn activate_connection(
        &self,
        connection: &zbus::zvariant::ObjectPath<'_>,
        device: &zbus::zvariant::ObjectPath<'_>,
        specific_object: &zbus::zvariant::ObjectPath<'_>,
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    fn deactivate_connection(
        &self,
        active_connection: &zbus::zvariant::ObjectPath<'_>,
    ) -> zbus::Result<()>;

    // Signals

    /// CheckPermissions signal
    #[zbus(signal)]
    fn check_permissions(&self) -> zbus::Result<()>;

    /// DeviceAdded signal
    #[zbus(signal)]
    fn device_added(&self, device_path: zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;

    /// DeviceRemoved signal
    #[zbus(signal)]
    fn device_removed(&self, device_path: zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;

    // Properties

    /// ActiveConnections property
    #[zbus(property)]
    fn active_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

    /// PrimaryConnection property
    #[zbus(property)]
    fn primary_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// PrimaryConnectionType property
    #[zbus(property)]
    fn primary_connection_type(&self) -> zbus::Result<String>;

    /// Connectivity property
    #[zbus(property)]
    fn connectivity(&self) -> zbus::Result<u32>;

    /// Devices property
    #[zbus(property)]
    fn devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

    // WirelessEnabled property
    #[zbus(property)]
    fn wireless_enabled(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_wireless_enabled(&self, value: bool) -> zbus::Result<()>;

    // State property
    #[zbus(property)]
    fn state(&self) -> zbus::Result<u32>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait Device {
    /// Delete method
    fn delete(&self) -> zbus::Result<()>;

    /// Disconnect method
    fn disconnect(&self) -> zbus::Result<()>;

    /// GetAppliedConnection method
    fn get_applied_connection(
        &self,
        flags: u32,
    ) -> zbus::Result<(
        std::collections::HashMap<
            String,
            std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
        >,
        u64,
    )>;

    /// Reapply method
    fn reapply(
        &self,
        connection: std::collections::HashMap<
            &str,
            std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
        >,
        version_id: u64,
        flags: u32,
    ) -> zbus::Result<()>;

    /// ActiveConnection property
    #[zbus(property)]
    fn active_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Autoconnect property
    #[zbus(property)]
    fn autoconnect(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_autoconnect(&self, value: bool) -> zbus::Result<()>;

    /// AvailableConnections property
    #[zbus(property)]
    fn available_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

    /// Capabilities property
    #[zbus(property)]
    fn capabilities(&self) -> zbus::Result<u32>;

    /// DeviceType property
    #[zbus(property)]
    fn device_type(&self) -> zbus::Result<u32>;

    /// Dhcp4Config property
    #[zbus(property)]
    fn dhcp4_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Dhcp6Config property
    #[zbus(property)]
    fn dhcp6_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Driver property
    #[zbus(property)]
    fn driver(&self) -> zbus::Result<String>;

    /// DriverVersion property
    #[zbus(property)]
    fn driver_version(&self) -> zbus::Result<String>;

    /// FirmwareMissing property
    #[zbus(property)]
    fn firmware_missing(&self) -> zbus::Result<bool>;

    /// FirmwareVersion property
    #[zbus(property)]
    fn firmware_version(&self) -> zbus::Result<String>;

    /// HwAddress property
    #[zbus(property)]
    fn hw_address(&self) -> zbus::Result<String>;

    /// Interface property
    #[zbus(property)]
    fn interface(&self) -> zbus::Result<String>;

    /// InterfaceFlags property
    #[zbus(property)]
    fn interface_flags(&self) -> zbus::Result<u32>;

    /// Ip4Address property
    #[zbus(property)]
    fn ip4_address(&self) -> zbus::Result<u32>;

    /// Ip4Config property
    #[zbus(property)]
    fn ip4_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Ip4Connectivity property
    #[zbus(property)]
    fn ip4_connectivity(&self) -> zbus::Result<u32>;

    /// Ip6Config property
    #[zbus(property)]
    fn ip6_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Ip6Connectivity property
    #[zbus(property)]
    fn ip6_connectivity(&self) -> zbus::Result<u32>;

    /// IpInterface property
    #[zbus(property)]
    fn ip_interface(&self) -> zbus::Result<String>;

    /// LldpNeighbors property
    #[zbus(property)]
    fn lldp_neighbors(
        &self,
    ) -> zbus::Result<Vec<std::collections::HashMap<String, zbus::zvariant::OwnedValue>>>;

    /// Managed property
    #[zbus(property)]
    fn managed(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_managed(&self, value: bool) -> zbus::Result<()>;

    /// Metered property
    #[zbus(property)]
    fn metered(&self) -> zbus::Result<u32>;

    /// Mtu property
    #[zbus(property)]
    fn mtu(&self) -> zbus::Result<u32>;

    /// NmPluginMissing property
    #[zbus(property)]
    fn nm_plugin_missing(&self) -> zbus::Result<bool>;

    /// Path property
    #[zbus(property)]
    fn path_(&self) -> zbus::Result<String>;

    /// PhysicalPortId property
    #[zbus(property)]
    fn physical_port_id(&self) -> zbus::Result<String>;

    /// Real property
    #[zbus(property)]
    fn real(&self) -> zbus::Result<bool>;

    /// State property
    #[zbus(property)]
    fn state(&self) -> zbus::Result<u32>;

    /// StateReason property
    #[zbus(property)]
    fn state_reason(&self) -> zbus::Result<(u32, u32)>;

    /// Udi property
    #[zbus(property)]
    fn udi(&self) -> zbus::Result<String>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait ActiveConnection {
    /// StateChanged signal
    // #[zbus(signal)]
    // fn state_changed(&self, state: u32, reason: u32) -> zbus::Result<()>;

    /// Connection property
    #[zbus(property)]
    fn connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Controller property
    #[zbus(property)]
    fn controller(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Default property
    #[zbus(property)]
    fn default(&self) -> zbus::Result<bool>;

    /// Default6 property
    #[zbus(property)]
    fn default6(&self) -> zbus::Result<bool>;

    /// Devices property
    #[zbus(property)]
    fn devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

    /// Dhcp4Config property
    #[zbus(property)]
    fn dhcp4_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Dhcp6Config property
    #[zbus(property)]
    fn dhcp6_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Id property
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;

    /// Ip4Config property
    #[zbus(property)]
    fn ip4_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Ip6Config property
    #[zbus(property)]
    fn ip6_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Master property
    #[zbus(property)]
    fn master(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// SpecificObject property
    #[zbus(property)]
    fn specific_object(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// State property
    #[zbus(property)]
    fn state(&self) -> zbus::Result<u32>;

    /// StateFlags property
    #[zbus(property)]
    fn state_flags(&self) -> zbus::Result<u32>;

    /// Type property
    #[zbus(property)]
    fn type_(&self) -> zbus::Result<String>;

    /// Uuid property
    #[zbus(property)]
    fn uuid(&self) -> zbus::Result<String>;

    /// Vpn property
    #[zbus(property)]
    fn vpn(&self) -> zbus::Result<bool>;
}
