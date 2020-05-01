mod ifaces;
mod bond;
mod common;
mod netlink;

pub(crate) use crate::ifaces::ifaces::get_ifaces;
pub use crate::ifaces::common::Iface;
pub use crate::ifaces::common::IfaceType;
pub use crate::ifaces::common::IfaceState;
