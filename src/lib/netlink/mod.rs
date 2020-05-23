mod bond;
mod ip;

pub(crate) use crate::netlink::bond::parse_bond_info;
pub(crate) use crate::netlink::bond::parse_bond_slave_info;
pub(crate) use crate::netlink::ip::fill_ip_addr;
