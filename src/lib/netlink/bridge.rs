use crate::parse_as_mac;
use crate::BridgeInfo;
use netlink_packet_route::rtnl::link::nlas::InfoBridge;

const ETH_ALEN: usize = 6;

pub(crate) fn parse_bridge_info(infos: &[InfoBridge]) -> BridgeInfo {
    let mut bridge_info = BridgeInfo::default();

    for info in infos {
        if let InfoBridge::ForwardDelay(d) = info {
            bridge_info.stp.forward_delay = *d;
        } else if let InfoBridge::HelloTime(d) = info {
            bridge_info.stp.hello_time = *d;
        } else if let InfoBridge::MaxAge(d) = info {
            bridge_info.stp.max_age = *d;
        } else if let InfoBridge::AgeingTime(d) = info {
            bridge_info.ageing_time = *d;
        } else if let InfoBridge::StpState(d) = info {
            bridge_info.stp.state = (*d).into();
        } else if let InfoBridge::Priority(d) = info {
            bridge_info.stp.priority = *d;
        } else if let InfoBridge::VlanFiltering(d) = info {
            bridge_info.vlan_filtering.enabled = *d > 0;
        } else if let InfoBridge::VlanProtocol(d) = info {
            bridge_info.vlan_filtering.vlan_protocol = (*d).into();
        } else if let InfoBridge::GroupFwdMask(d) = info {
            bridge_info.group_fwd_mask = *d;
        } else if let InfoBridge::RootId((priority, mac)) = info {
            bridge_info.root_id = parse_bridge_id(*priority, mac);
        } else if let InfoBridge::BridgeId((priority, mac)) = info {
            bridge_info.bridge_id = parse_bridge_id(*priority, mac);
        } else if let InfoBridge::RootPort(d) = info {
            bridge_info.root_port = *d;
        } else if let InfoBridge::RootPathCost(d) = info {
            bridge_info.root_path_cost = *d;
        } else if let InfoBridge::TopologyChange(d) = info {
            bridge_info.topology_change = *d > 0;
        } else if let InfoBridge::TopologyChangeDetected(d) = info {
            bridge_info.topology_change_detected = *d > 0;
        } else if let InfoBridge::HelloTimer(d) = info {
            bridge_info.hello_timer = *d;
        } else if let InfoBridge::TcnTimer(d) = info {
            bridge_info.tcn_timer = *d;
        } else if let InfoBridge::TopologyChangeTimer(d) = info {
            bridge_info.topology_change_timer = *d;
        } else if let InfoBridge::GcTimer(d) = info {
            bridge_info.gc_timer = *d;
        } else if let InfoBridge::GroupAddr(d) = info {
            bridge_info.group_addr = parse_as_mac(ETH_ALEN, d);
        // InfoBridge::FdbFlush is only used for changing bridge
        } else if let InfoBridge::MulticastRouter(d) = info {
            bridge_info.multicast_igmp.router = (*d).into();
        } else if let InfoBridge::MulticastSnooping(d) = info {
            bridge_info.multicast_igmp.snooping = (*d) > 0;
        } else if let InfoBridge::MulticastQueryUseIfaddr(d) = info {
            bridge_info.multicast_igmp.query_use_ifaddr = (*d) > 0;
        } else if let InfoBridge::MulticastQuerier(d) = info {
            bridge_info.multicast_igmp.querier = (*d) > 0;
        } else if let InfoBridge::MulticastHashElasticity(d) = info {
            bridge_info.multicast_igmp.hash_elasticity = *d;
        } else if let InfoBridge::MulticastHashMax(d) = info {
            bridge_info.multicast_igmp.hash_max = *d;
        } else if let InfoBridge::MulticastLastMemberCount(d) = info {
            bridge_info.multicast_igmp.last_member_count = *d;
        } else if let InfoBridge::MulticastStartupQueryCount(d) = info {
            bridge_info.multicast_igmp.startup_query_count = *d;
        } else if let InfoBridge::MulticastLastMemberInterval(d) = info {
            bridge_info.multicast_igmp.last_member_interval = *d;
        } else if let InfoBridge::MulticastMembershipInterval(d) = info {
            bridge_info.multicast_igmp.membership_interval = *d;
        } else if let InfoBridge::MulticastQuerierInterval(d) = info {
            bridge_info.multicast_igmp.querier_interval = *d;
        } else if let InfoBridge::MulticastQueryInterval(d) = info {
            bridge_info.multicast_igmp.query_interval = *d;
        } else if let InfoBridge::MulticastQueryResponseInterval(d) = info {
            bridge_info.multicast_igmp.query_response_interval = *d;
        } else if let InfoBridge::MulticastStartupQueryInterval(d) = info {
            bridge_info.multicast_igmp.startup_query_interval = *d;
        } else if let InfoBridge::NfCallIpTables(d) = info {
            bridge_info.nf_call_iptables = *d > 0;
        } else if let InfoBridge::NfCallIp6Tables(d) = info {
            bridge_info.nf_call_ip6tables = *d > 0;
        } else if let InfoBridge::NfCallArpTables(d) = info {
            bridge_info.nf_call_arptables = *d > 0;
        } else if let InfoBridge::VlanDefaultPvid(d) = info {
            bridge_info.vlan_filtering.default_pvid = Some((*d).into());
        } else if let InfoBridge::VlanStatsEnabled(d) = info {
            bridge_info.vlan_filtering.vlan_stats_enabled = *d > 0;
        } else if let InfoBridge::MulticastStatsEnabled(d) = info {
            bridge_info.multicast_igmp.stats_enabled = *d > 0;
        } else if let InfoBridge::MulticastIgmpVersion(d) = info {
            bridge_info.multicast_igmp.igmp_version = *d;
        } else if let InfoBridge::MulticastMldVersion(d) = info {
            bridge_info.multicast_igmp.mld_version = *d;
        // TODO: wait https://github.com/little-dude/netlink/pull/80
        //  for IFLA_BR_VLAN_STATS_PER_PORT and IFLA_BR_MULTI_BOOLOPT
        } else {
            eprintln!("Unknown NLA {:?}", &info);
        }
    }
    bridge_info
}

fn parse_bridge_id(priority: u16, mac: &[u8; 6]) -> String {
    //Following the format of sysfs
    let priority_bytes = priority.to_ne_bytes();
    format!(
        "{:02x}{:02x}.{}",
        priority_bytes[0],
        priority_bytes[1],
        parse_as_mac(ETH_ALEN, mac).to_lowercase().replace(":", "")
    )
}
