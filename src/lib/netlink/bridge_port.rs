use crate::netlink::nla::parse_as_u16;
use crate::netlink::nla::parse_as_u32;
use crate::netlink::nla::parse_as_u64;
use crate::netlink::nla::parse_as_u8;
use crate::netlink::nla::parse_nla_header;
use crate::netlink::nla::NL_ATTR_HDR_LEN;
use crate::BridgePortInfo;
use std::convert::TryInto;

fn parse_void_port_info(_data: &[u8], _port_info: &mut BridgePortInfo) {
    ()
}

fn parse_brport_state(data: &[u8], port_info: &mut BridgePortInfo) {
    port_info.stp_state = parse_as_u8(data).into();
}

fn parse_brport_priority(data: &[u8], port_info: &mut BridgePortInfo) {
    port_info.stp_priority = parse_as_u16(data);
}

fn parse_brport_cost(data: &[u8], port_info: &mut BridgePortInfo) {
    port_info.stp_path_cost = parse_as_u32(data);
}

fn parse_brport_mode(data: &[u8], port_info: &mut BridgePortInfo) {
    port_info.hairpin_mode = parse_as_u8(data) > 0;
}

fn parse_brport_guard(data: &[u8], port_info: &mut BridgePortInfo) {
    port_info.bpdu_guard = parse_as_u8(data) > 0;
}

fn parse_brport_protect(data: &[u8], port_info: &mut BridgePortInfo) {
    port_info.root_block = parse_as_u8(data) > 0;
}

fn parse_brport_fast_leave(data: &[u8], port_info: &mut BridgePortInfo) {
    port_info.multicast_fast_leave = parse_as_u8(data) > 0;
}

fn parse_brport_learning(data: &[u8], port_info: &mut BridgePortInfo) {
    port_info.learning = parse_as_u8(data) > 0;
}

fn parse_brport_unicast_flood(data: &[u8], port_info: &mut BridgePortInfo) {
    port_info.unicast_flood = parse_as_u8(data) > 0;
}

fn parse_brport_proxyarp(data: &[u8], port_info: &mut BridgePortInfo) {
    port_info.proxyarp = parse_as_u8(data) > 0;
}

fn parse_brport_learning_sync(_data: &[u8], _port_info: &mut BridgePortInfo) {
    () // Ther kernel 5.7-rc6 never update fill value in br_port_fill_attrs
}

fn parse_brport_proxyarp_wifi(data: &[u8], port_info: &mut BridgePortInfo) {
    port_info.proxyarp_wifi = parse_as_u8(data) > 0;
}

fn parse_brport_root_id(data: &[u8], port_info: &mut BridgePortInfo) {
    port_info.designated_root = parse_as_bridge_id(data);
}

fn parse_brport_bridge_id(data: &[u8], port_info: &mut BridgePortInfo) {
    port_info.designated_bridge = parse_as_bridge_id(data);
}

fn parse_brport_designated_port(data: &[u8], port_info: &mut BridgePortInfo) {
    port_info.designated_port = parse_as_u16(data);
}

fn parse_brport_designated_cost(data: &[u8], cost_info: &mut BridgePortInfo) {
    cost_info.designated_cost = parse_as_u16(data);
}

fn parse_brport_id(data: &[u8], cost_info: &mut BridgePortInfo) {
    cost_info.port_id = format!("0x{:04x}", parse_as_u16(data));
}

fn parse_brport_no(data: &[u8], cost_info: &mut BridgePortInfo) {
    cost_info.port_no = format!("0x{:x}", parse_as_u16(data));
}

fn parse_brport_topology_change_ack(
    data: &[u8],
    cost_info: &mut BridgePortInfo,
) {
    cost_info.change_ack = parse_as_u8(data) > 0;
}

fn parse_brport_config_pending(data: &[u8], cost_info: &mut BridgePortInfo) {
    cost_info.config_pending = parse_as_u8(data) > 0;
}

fn parse_brport_message_age_timer(data: &[u8], cost_info: &mut BridgePortInfo) {
    cost_info.message_age_timer = parse_as_u64(data);
}

fn parse_brport_forward_delay_timer(
    data: &[u8],
    cost_info: &mut BridgePortInfo,
) {
    cost_info.forward_delay_timer = parse_as_u64(data);
}

fn parse_brport_hold_timer(data: &[u8], cost_info: &mut BridgePortInfo) {
    cost_info.hold_timer = parse_as_u64(data);
}

fn parse_brport_multicast_router(data: &[u8], cost_info: &mut BridgePortInfo) {
    cost_info.multicast_router = parse_as_u8(data).into();
}

fn parse_brport_mcast_flood(data: &[u8], cost_info: &mut BridgePortInfo) {
    cost_info.multicast_flood = parse_as_u8(data) > 0;
}

fn parse_brport_mcast_to_ucast(data: &[u8], cost_info: &mut BridgePortInfo) {
    cost_info.multicast_to_unicast = parse_as_u8(data) > 0;
}

fn parse_brport_vlan_tunnel(data: &[u8], cost_info: &mut BridgePortInfo) {
    cost_info.vlan_tunnel = parse_as_u8(data) > 0;
}

fn parse_brport_bast_flood(data: &[u8], cost_info: &mut BridgePortInfo) {
    cost_info.broadcast_flood = parse_as_u8(data) > 0;
}

fn parse_brport_group_fwd_mask(data: &[u8], cost_info: &mut BridgePortInfo) {
    cost_info.group_fwd_mask = parse_as_u16(data);
}

fn parse_brport_neigh_suppress(data: &[u8], cost_info: &mut BridgePortInfo) {
    cost_info.neigh_suppress = parse_as_u8(data) > 0;
}

fn parse_brport_isolated(data: &[u8], cost_info: &mut BridgePortInfo) {
    cost_info.isolated = parse_as_u8(data) > 0;
}

fn parse_brport_backup_port(data: &[u8], cost_info: &mut BridgePortInfo) {
    cost_info.backup_port = format!("{}", parse_as_u32(data));
}

const NLA_PORT_PARSE_FUNS: &[fn(&[u8], &mut BridgePortInfo)] = &[
    parse_void_port_info, // IFLA_BRPORT_UNSPEC
    parse_brport_state,
    parse_brport_priority,
    parse_brport_cost,
    parse_brport_mode,
    parse_brport_guard,
    parse_brport_protect,
    parse_brport_fast_leave,
    parse_brport_learning,
    parse_brport_unicast_flood,
    parse_brport_proxyarp,
    parse_brport_learning_sync,
    parse_brport_proxyarp_wifi,
    parse_brport_root_id,
    parse_brport_bridge_id,
    parse_brport_designated_port,
    parse_brport_designated_cost,
    parse_brport_id,
    parse_brport_no,
    parse_brport_topology_change_ack,
    parse_brport_config_pending,
    parse_brport_message_age_timer,
    parse_brport_forward_delay_timer,
    parse_brport_hold_timer,
    parse_void_port_info, // IFLA_BRPORT_FLUSH
    parse_brport_multicast_router,
    parse_void_port_info, // IFLA_BRPORT_PAD
    parse_brport_mcast_flood,
    parse_brport_mcast_to_ucast,
    parse_brport_vlan_tunnel,
    parse_brport_bast_flood,
    parse_brport_group_fwd_mask,
    parse_brport_neigh_suppress,
    parse_brport_isolated,
    parse_brport_backup_port,
];

pub(crate) fn parse_bridge_port_info(raw: &[u8]) -> BridgePortInfo {
    let mut port_info = BridgePortInfo::default();
    let mut i: usize = 0;

    // TODO: Dup with parse_bond_info
    while i < raw.len() {
        let hdr_ptr = raw.as_ptr().wrapping_offset(i.try_into().unwrap());
        let hdr = parse_nla_header(hdr_ptr);
        let data_ptr = raw
            .as_ptr()
            .wrapping_offset((i + NL_ATTR_HDR_LEN).try_into().unwrap());
        let data = unsafe {
            std::slice::from_raw_parts(data_ptr, hdr.nla_len - NL_ATTR_HDR_LEN)
        };
        if let Some(func) =
            NLA_PORT_PARSE_FUNS.get::<usize>(hdr.nla_type.into())
        {
            func(data, &mut port_info);
        } else {
            eprintln!("unknown nla_type: {} {:?}", hdr.nla_type, data);
        }
        i = i + hdr.nla_len;
    }
    port_info
}

fn parse_as_bridge_id(data: &[u8]) -> String {
    format!(
        "{:02x}{:02x}.{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    )
}
