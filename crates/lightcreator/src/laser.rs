//! The serial link to the laser is in baseCreator (`base_ui::link`); what is here is only the part that knows a laser
//! profile: where to connect.
pub use base_ui::link::*;

/// The connection settings of a device profile, if a target is configured.
pub fn target_of(dev: &lc_core::Device) -> Option<Target> {
    if !dev.has_target() {
        return None;
    }
    Some(match dev.connection.link {
        lc_core::LinkKind::Serial => Target::Serial { port: resolve_port(&dev.connection.port, &dev.connection.usb_id, &list_port_info()), baud: dev.connection.baud },
        lc_core::LinkKind::Tcp => Target::Tcp { host: dev.connection.host.trim().to_string(), port: dev.connection.tcp_port },
    })
}
