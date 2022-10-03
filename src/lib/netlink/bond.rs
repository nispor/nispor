// SPDX-License-Identifier: Apache-2.0

use crate::mac::parse_as_mac;
use crate::netlink::nla::parse_as_u16;
use crate::netlink::nla::parse_as_u32;
use crate::netlink::nla::parse_as_u8;
use crate::BondMiiStatus;
use crate::BondSubordinateInfo;
use crate::BondSubordinateState;
use crate::NisporError;
use netlink_packet_route::rtnl::nlas::NlasIterator;

const IFLA_BOND_SLAVE_STATE: u16 = 1;
const IFLA_BOND_SLAVE_MII_STATUS: u16 = 2;
const IFLA_BOND_SLAVE_LINK_FAILURE_COUNT: u16 = 3;
const IFLA_BOND_SLAVE_PERM_HWADDR: u16 = 4;
const IFLA_BOND_SLAVE_QUEUE_ID: u16 = 5;
const IFLA_BOND_SLAVE_AD_AGGREGATOR_ID: u16 = 6;
const IFLA_BOND_SLAVE_AD_ACTOR_OPER_PORT_STATE: u16 = 7;
const IFLA_BOND_SLAVE_AD_PARTNER_OPER_PORT_STATE: u16 = 8;

pub(crate) fn parse_bond_subordinate_info(
    raw: &[u8],
) -> Result<BondSubordinateInfo, NisporError> {
    let nlas = NlasIterator::new(raw);
    let mut subordinate_state = BondSubordinateState::Unknown;
    let mut mii_status = BondMiiStatus::Unknown;
    let mut link_failure_count = std::u32::MAX;
    let mut perm_hwaddr = String::new();
    let mut queue_id = std::u16::MAX;
    let mut ad_aggregator_id = None;
    let mut ad_actor_oper_port_state = None;
    let mut ad_partner_oper_port_state = None;
    for nla in nlas {
        match nla {
            Ok(nla) => match nla.kind() {
                IFLA_BOND_SLAVE_STATE => {
                    subordinate_state = parse_as_u8(nla.value())?.into()
                }
                IFLA_BOND_SLAVE_MII_STATUS => {
                    mii_status = parse_as_u8(nla.value())?.into()
                }
                IFLA_BOND_SLAVE_LINK_FAILURE_COUNT => {
                    link_failure_count = parse_as_u32(nla.value())?
                }
                IFLA_BOND_SLAVE_PERM_HWADDR => {
                    perm_hwaddr =
                        parse_as_mac(nla.value_length(), nla.value())?;
                }
                IFLA_BOND_SLAVE_QUEUE_ID => {
                    queue_id = parse_as_u16(nla.value())?
                }
                IFLA_BOND_SLAVE_AD_AGGREGATOR_ID => {
                    ad_aggregator_id = Some(parse_as_u16(nla.value())?);
                }
                IFLA_BOND_SLAVE_AD_ACTOR_OPER_PORT_STATE => {
                    ad_actor_oper_port_state = Some(parse_as_u8(nla.value())?);
                }
                IFLA_BOND_SLAVE_AD_PARTNER_OPER_PORT_STATE => {
                    ad_partner_oper_port_state =
                        Some(parse_as_u16(nla.value())?);
                }
                _ => {
                    log::warn!(
                        "unknown nla kind {} value: {:?}",
                        nla.kind(),
                        nla.value()
                    );
                }
            },
            Err(e) => {
                log::warn!("{}", e);
            }
        }
    }
    Ok(BondSubordinateInfo {
        subordinate_state,
        mii_status,
        link_failure_count,
        perm_hwaddr,
        queue_id,
        ad_aggregator_id,
        ad_actor_oper_port_state,
        ad_partner_oper_port_state,
    })
}
