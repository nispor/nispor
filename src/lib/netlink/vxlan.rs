use crate::ifaces::VxlanInfo;
use crate::netlink::nla::parse_as_be16;
use crate::netlink::nla::parse_as_be32;
use crate::netlink::nla::parse_as_ipv4;
use crate::netlink::nla::parse_as_ipv6;
use crate::netlink::nla::parse_as_u32;
use crate::netlink::nla::parse_as_u8;
use netlink_packet_route::rtnl::nlas::NlasIterator;

const IFLA_VXLAN_ID: u16 = 1;
const IFLA_VXLAN_GROUP: u16 = 2;
const IFLA_VXLAN_LINK: u16 = 3;
const IFLA_VXLAN_LOCAL: u16 = 4;
const IFLA_VXLAN_TTL: u16 = 5;
const IFLA_VXLAN_TOS: u16 = 6;
const IFLA_VXLAN_LEARNING: u16 = 7;
const IFLA_VXLAN_AGEING: u16 = 8;
const IFLA_VXLAN_LIMIT: u16 = 9;
const IFLA_VXLAN_PORT_RANGE: u16 = 10;
const IFLA_VXLAN_PROXY: u16 = 11;
const IFLA_VXLAN_RSC: u16 = 12;
const IFLA_VXLAN_L2MISS: u16 = 13;
const IFLA_VXLAN_L3MISS: u16 = 14;
const IFLA_VXLAN_PORT: u16 = 15;
const IFLA_VXLAN_GROUP6: u16 = 16;
const IFLA_VXLAN_LOCAL6: u16 = 17;
const IFLA_VXLAN_UDP_CSUM: u16 = 18;
const IFLA_VXLAN_UDP_ZERO_CSUM6_TX: u16 = 19;
const IFLA_VXLAN_UDP_ZERO_CSUM6_RX: u16 = 20;
const IFLA_VXLAN_REMCSUM_TX: u16 = 21;
const IFLA_VXLAN_REMCSUM_RX: u16 = 22;
const IFLA_VXLAN_GBP: u16 = 23;
const IFLA_VXLAN_REMCSUM_NOPARTIAL: u16 = 24;
const IFLA_VXLAN_COLLECT_METADATA: u16 = 25;
const IFLA_VXLAN_LABEL: u16 = 26;
const IFLA_VXLAN_GPE: u16 = 27;
const IFLA_VXLAN_TTL_INHERIT: u16 = 28;
const IFLA_VXLAN_DF: u16 = 29;

pub(crate) fn parse_vxlan_info(raw: &[u8]) -> VxlanInfo {
    let nlas = NlasIterator::new(raw);
    let mut info = VxlanInfo::default();
    for nla in nlas {
        match nla {
            Ok(nla) => match nla.kind() {
                IFLA_VXLAN_ID => {
                    info.vxlan_id = parse_as_u32(nla.value());
                }
                IFLA_VXLAN_GROUP => {
                    info.remote = parse_as_ipv4(nla.value());
                }
                IFLA_VXLAN_LINK => {
                    info.base_iface = format!("{}", parse_as_u32(nla.value()));
                }
                IFLA_VXLAN_LOCAL => {
                    info.local = parse_as_ipv4(nla.value());
                }
                IFLA_VXLAN_TTL => {
                    info.ttl = parse_as_u8(nla.value());
                }
                IFLA_VXLAN_TOS => {
                    info.tos = parse_as_u8(nla.value());
                }
                IFLA_VXLAN_LEARNING => {
                    info.learning = parse_as_u8(nla.value()) > 0;
                }
                IFLA_VXLAN_AGEING => {
                    info.ageing = parse_as_u32(nla.value());
                }
                IFLA_VXLAN_LIMIT => {
                    info.max_address = parse_as_u32(nla.value());
                }
                IFLA_VXLAN_PORT_RANGE => {
                    info.src_port_min = parse_as_be16(nla.value());
                    info.src_port_max =
                        parse_as_be16(&[nla.value()[2], nla.value()[3]]);
                }
                IFLA_VXLAN_PROXY => {
                    info.proxy = parse_as_u8(nla.value()) > 0;
                }
                IFLA_VXLAN_RSC => {
                    info.rsc = parse_as_u8(nla.value()) > 0;
                }
                IFLA_VXLAN_L2MISS => {
                    info.l2miss = parse_as_u8(nla.value()) > 0;
                }
                IFLA_VXLAN_L3MISS => {
                    info.l3miss = parse_as_u8(nla.value()) > 0;
                }
                IFLA_VXLAN_PORT => {
                    info.dst_port_min = parse_as_be16(nla.value());
                    info.dst_port_max =
                        parse_as_be16(&[nla.value()[2], nla.value()[3]]);
                }
                IFLA_VXLAN_GROUP6 => {
                    info.remote = parse_as_ipv6(nla.value());
                }
                IFLA_VXLAN_LOCAL6 => {
                    info.local = parse_as_ipv6(nla.value());
                }
                IFLA_VXLAN_UDP_CSUM => {
                    info.udp_check_sum = parse_as_u8(nla.value()) > 0;
                }
                IFLA_VXLAN_UDP_ZERO_CSUM6_TX => {
                    info.udp6_zero_check_sum_tx = parse_as_u8(nla.value()) > 0;
                }
                IFLA_VXLAN_UDP_ZERO_CSUM6_RX => {
                    info.udp6_zero_check_sum_rx = parse_as_u8(nla.value()) > 0;
                }
                IFLA_VXLAN_REMCSUM_TX => {
                    info.remote_check_sum_tx = parse_as_u8(nla.value()) > 0;
                }
                IFLA_VXLAN_REMCSUM_RX => {
                    info.remote_check_sum_rx = parse_as_u8(nla.value()) > 0;
                }
                IFLA_VXLAN_GBP => {
                    info.gbp = true;
                }
                IFLA_VXLAN_REMCSUM_NOPARTIAL => {
                    info.remote_check_sum_no_partial = true;
                }
                IFLA_VXLAN_COLLECT_METADATA => {
                    info.collect_metadata = parse_as_u8(nla.value()) > 0;
                }
                IFLA_VXLAN_LABEL => {
                    info.label = parse_as_be32(nla.value());
                }
                IFLA_VXLAN_GPE => {
                    info.gpe = true;
                }
                IFLA_VXLAN_TTL_INHERIT => {
                    info.ttl_inherit = parse_as_u8(nla.value()) > 0;
                }
                IFLA_VXLAN_DF => {
                    info.df = parse_as_u8(nla.value());
                }
                _ => eprintln!(
                    "Unhandled VxLAN IFLA_INFO_DATA: {}, {:?}",
                    nla.kind(),
                    nla.value()
                ),
            },
            Err(e) => eprintln!("{}", e),
        }
    }
    info
}
