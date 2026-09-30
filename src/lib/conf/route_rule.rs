// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use rtnetlink::packet_route::{
    AddressFamily as NlAddressFamily,
    route::RouteHeader,
    rule::{RuleAction as NlRuleAction, RuleAttribute, RuleMessage},
};
use serde::{Deserialize, Serialize};

use super::super::query::parse_ip_net_addr_str;
use crate::{AddressFamily, NisporError, RuleAction};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RouteRuleConf {
    /// Remove the matching route rule instead of adding this route rule.
    #[serde(default)]
    pub remove: bool,
    /// Address family of this route rule.
    pub address_family: AddressFamily,
    /// Action of this route rule. Must be explicitly specified since
    /// `unspec` is not a valid rule action.
    pub action: RuleAction,
    /// Routing table ID to lookup.
    pub table: Option<u32>,
    /// Source network to match in CIDR notation or single IP address.
    pub src: Option<String>,
    /// Destination network to match in CIDR notation or single IP address.
    pub dst: Option<String>,
    /// Incoming interface name.
    pub iif: Option<String>,
    /// Target route rule priority to jump to, only allowed with action
    /// `goto`.
    pub goto: Option<u32>,
    /// Priority of this route rule. Smaller number means higher priority.
    /// Mandatory so the adding of duplicate rules can be detected and
    /// ignored, keeping `NetConf::apply()` idempotent.
    pub priority: u32,
    /// Firewall mark to match.
    pub fw_mark: Option<u32>,
    /// Firewall mark mask to match.
    pub fw_mask: Option<u32>,
    /// Prefix length to suppress.
    pub suppress_prefix_len: Option<u32>,
}

pub(crate) async fn apply_rules_conf(
    rules: &[RouteRuleConf],
) -> Result<(), NisporError> {
    let (connection, handle, _) = rtnetlink::new_connection()?;
    tokio::spawn(connection);
    for rule in rules {
        apply_rule_conf(&handle, rule).await?;
    }
    Ok(())
}

async fn apply_rule_conf(
    handle: &rtnetlink::Handle,
    rule: &RouteRuleConf,
) -> Result<(), NisporError> {
    let msg = rule_conf_to_message(rule)?;
    if rule.remove {
        if let Err(e) = handle.rule().del(msg).execute().await {
            if let rtnetlink::Error::NetlinkError(ref e) = e
                && e.raw_code() == -libc::ENOENT
            {
                return Ok(());
            }
            return Err(e.into());
        }
    } else {
        let mut request = handle.rule().add();
        *request.message_mut() = msg;
        if let Err(e) = request.execute().await {
            if let rtnetlink::Error::NetlinkError(ref e) = e
                && e.raw_code() == -libc::EEXIST
            {
                return Ok(());
            }
            return Err(e.into());
        }
    }
    Ok(())
}

fn rule_conf_to_message(
    rule: &RouteRuleConf,
) -> Result<RuleMessage, NisporError> {
    let address_family = match rule.address_family {
        AddressFamily::Ipv4 => NlAddressFamily::Inet,
        AddressFamily::Ipv6 => NlAddressFamily::Inet6,
        _ => {
            return Err(invalid_argument(format!(
                "Unsupported route rule address family {:?}",
                rule.address_family
            )));
        }
    };

    let mut msg = RuleMessage::default();
    msg.header.family = address_family;
    msg.header.action = match rule.action {
        RuleAction::Table => NlRuleAction::ToTable,
        RuleAction::Goto => NlRuleAction::Goto,
        RuleAction::Nop => NlRuleAction::Nop,
        RuleAction::Blackhole => NlRuleAction::Blackhole,
        RuleAction::Unreachable => NlRuleAction::Unreachable,
        RuleAction::Prohibit => NlRuleAction::Prohibit,
        RuleAction::Other(v) => NlRuleAction::Other(v),
        RuleAction::Unspec => {
            return Err(invalid_argument(
                "Route rule action `unspec` is invalid, please specify the \
                 action explicitly"
                    .to_string(),
            ));
        }
    };

    if msg.header.action == NlRuleAction::Goto {
        let Some(goto) = rule.goto else {
            return Err(invalid_argument(
                "Route rule action goto requires a goto target".to_string(),
            ));
        };
        if goto <= rule.priority {
            return Err(invalid_argument(format!(
                "Route rule goto target {goto} should be greater than its \
                 priority {}",
                rule.priority
            )));
        }
        msg.attributes.push(RuleAttribute::Goto(goto));
    } else if rule.goto.is_some() {
        return Err(invalid_argument(
            "Route rule goto target is only allowed with action goto"
                .to_string(),
        ));
    }

    if let Some(table) = rule.table {
        if table == 0 {
            return Err(invalid_argument(
                "Route rule table ID 0 is invalid".to_string(),
            ));
        }
        if table <= u8::MAX.into() {
            msg.header.table = table as u8;
        } else {
            msg.attributes.push(RuleAttribute::Table(table));
        }
    } else if msg.header.action == NlRuleAction::ToTable {
        msg.header.table = RouteHeader::RT_TABLE_MAIN;
    }

    if let Some(src) = rule.src.as_deref() {
        let (ip, prefix_len) = parse_rule_network(src, address_family)?;
        msg.header.src_len = prefix_len;
        msg.attributes.push(RuleAttribute::Source(ip));
    }
    if let Some(dst) = rule.dst.as_deref() {
        let (ip, prefix_len) = parse_rule_network(dst, address_family)?;
        msg.header.dst_len = prefix_len;
        msg.attributes.push(RuleAttribute::Destination(ip));
    }
    msg.attributes.push(RuleAttribute::Priority(rule.priority));
    if let Some(fw_mark) = rule.fw_mark {
        msg.attributes.push(RuleAttribute::FwMark(fw_mark));
    }
    if let Some(fw_mask) = rule.fw_mask {
        msg.attributes.push(RuleAttribute::FwMask(fw_mask));
    }
    if let Some(iif) = rule.iif.as_deref() {
        msg.attributes.push(RuleAttribute::Iifname(iif.to_string()));
    }
    if let Some(suppress_prefix_len) = rule.suppress_prefix_len {
        msg.attributes
            .push(RuleAttribute::SuppressPrefixLen(suppress_prefix_len));
    }

    Ok(msg)
}

fn parse_rule_network(
    ip_net: &str,
    address_family: NlAddressFamily,
) -> Result<(IpAddr, u8), NisporError> {
    let (ip, prefix_len) = parse_ip_net_addr_str(ip_net)?;
    if ip.is_ipv6() != (address_family == NlAddressFamily::Inet6) {
        return Err(invalid_argument(format!(
            "Route rule network {ip_net} does not match the address family"
        )));
    }
    let max_prefix_len = if ip.is_ipv6() { 128 } else { 32 };
    if prefix_len > max_prefix_len {
        return Err(invalid_argument(format!(
            "Invalid route rule network {ip_net}: prefix length should be no \
             greater than {max_prefix_len}"
        )));
    }
    Ok((ip, prefix_len))
}

fn invalid_argument(message: String) -> NisporError {
    let e = NisporError::invalid_argument(message);
    log::error!("{e}");
    e
}

#[cfg(test)]
mod tests {
    use rtnetlink::packet_route::rule::RuleAttribute;

    use super::*;

    fn rule_conf_from_yaml(yaml: &str) -> RouteRuleConf {
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn test_rule_conf_to_message_ipv4_table_lookup() {
        let conf = rule_conf_from_yaml(
            r#"
            address-family: ipv4
            action: table
            src: 198.51.100.0/24
            dst: 192.0.2.0/24
            iif: eth1
            table: 200
            priority: 1000
            fw-mark: 0x10
            fw-mask: 0xff00
            suppress-prefix-len: 0
            "#,
        );

        let msg = rule_conf_to_message(&conf).unwrap();

        assert_eq!(msg.header.family, NlAddressFamily::Inet);
        assert_eq!(msg.header.action, NlRuleAction::ToTable);
        assert_eq!(msg.header.table, 200);
        assert_eq!(msg.header.src_len, 24);
        assert_eq!(msg.header.dst_len, 24);
        assert!(msg.attributes.contains(&RuleAttribute::Source(
            "198.51.100.0".parse::<IpAddr>().unwrap()
        )));
        assert!(msg.attributes.contains(&RuleAttribute::Destination(
            "192.0.2.0".parse::<IpAddr>().unwrap()
        )));
        assert!(
            msg.attributes
                .contains(&RuleAttribute::Iifname("eth1".to_string()))
        );
        assert!(msg.attributes.contains(&RuleAttribute::Priority(1000)));
        assert!(msg.attributes.contains(&RuleAttribute::FwMark(0x10)));
        assert!(msg.attributes.contains(&RuleAttribute::FwMask(0xff00)));
        assert!(
            msg.attributes
                .contains(&RuleAttribute::SuppressPrefixLen(0))
        );
    }

    #[test]
    fn test_rule_conf_to_message_ipv6_blackhole() {
        let conf = rule_conf_from_yaml(
            r#"
            address-family: ipv6
            action: blackhole
            src: 2001:db8:f::/64
            priority: 1001
            "#,
        );

        let msg = rule_conf_to_message(&conf).unwrap();

        assert_eq!(msg.header.family, NlAddressFamily::Inet6);
        assert_eq!(msg.header.action, NlRuleAction::Blackhole);
        assert_eq!(msg.header.table, 0);
        assert_eq!(msg.header.src_len, 64);
        assert!(msg.attributes.contains(&RuleAttribute::Source(
            "2001:db8:f::".parse::<IpAddr>().unwrap()
        )));
        assert!(msg.attributes.contains(&RuleAttribute::Priority(1001)));
    }

    #[test]
    fn test_rule_conf_to_message_default_table_is_main() {
        let conf = rule_conf_from_yaml(
            r#"
            address-family: ipv4
            action: table
            src: 198.51.100.1
            priority: 1002
            "#,
        );

        let msg = rule_conf_to_message(&conf).unwrap();

        assert_eq!(msg.header.action, NlRuleAction::ToTable);
        assert_eq!(msg.header.table, RouteHeader::RT_TABLE_MAIN);
        assert_eq!(msg.header.src_len, 32);
    }

    #[test]
    fn test_rule_conf_to_message_big_table_uses_attribute() {
        let conf = rule_conf_from_yaml(
            r#"
            address-family: ipv4
            action: table
            table: 500
            priority: 1003
            "#,
        );

        let msg = rule_conf_to_message(&conf).unwrap();

        assert_eq!(msg.header.table, 0);
        assert!(msg.attributes.contains(&RuleAttribute::Table(500)));
    }

    #[test]
    fn test_rule_conf_to_message_goto() {
        let conf = rule_conf_from_yaml(
            r#"
            address-family: ipv4
            action: goto
            goto: 1010
            priority: 1002
            "#,
        );

        let msg = rule_conf_to_message(&conf).unwrap();

        assert_eq!(msg.header.action, NlRuleAction::Goto);
        assert_eq!(msg.header.table, 0);
        assert!(msg.attributes.contains(&RuleAttribute::Goto(1010)));
        assert!(msg.attributes.contains(&RuleAttribute::Priority(1002)));
    }

    #[test]
    fn test_rule_conf_to_message_goto_without_target() {
        let conf = rule_conf_from_yaml(
            r#"
            address-family: ipv4
            action: goto
            priority: 1002
            "#,
        );

        assert!(rule_conf_to_message(&conf).is_err());
    }

    #[test]
    fn test_rule_conf_to_message_goto_with_other_action() {
        let conf = rule_conf_from_yaml(
            r#"
            address-family: ipv4
            action: table
            goto: 1010
            priority: 1002
            "#,
        );

        assert!(rule_conf_to_message(&conf).is_err());
    }

    #[test]
    fn test_rule_conf_to_message_backward_goto() {
        // The goto target should be greater than the rule priority.
        for target in [1001, 1002] {
            let conf = rule_conf_from_yaml(&format!(
                r#"
                address-family: ipv4
                action: goto
                goto: {target}
                priority: 1002
                "#
            ));

            assert!(rule_conf_to_message(&conf).is_err());
        }
    }

    #[test]
    fn test_rule_conf_to_message_invalid_table_id() {
        let conf = rule_conf_from_yaml(
            r#"
            address-family: ipv4
            action: table
            table: 0
            priority: 1005
            "#,
        );

        assert!(rule_conf_to_message(&conf).is_err());
    }

    #[test]
    fn test_rule_conf_to_message_invalid_prefix_len() {
        for (family, src) in
            [("ipv4", "192.0.2.0/33"), ("ipv6", "2001:db8::/129")]
        {
            let conf = rule_conf_from_yaml(&format!(
                r#"
                address-family: {family}
                action: table
                src: {src}
                priority: 1006
                "#
            ));

            assert!(rule_conf_to_message(&conf).is_err());
        }
    }

    #[test]
    fn test_rule_conf_to_message_family_mismatch() {
        let conf = rule_conf_from_yaml(
            r#"
            address-family: ipv4
            action: table
            src: 2001:db8:f::/64
            priority: 1004
            "#,
        );

        assert!(rule_conf_to_message(&conf).is_err());
    }

    #[test]
    fn test_rule_conf_to_message_invalid_network() {
        let conf = rule_conf_from_yaml(
            r#"
            address-family: ipv4
            action: table
            src: not-a-network
            priority: 1005
            "#,
        );

        assert!(rule_conf_to_message(&conf).is_err());
    }

    #[test]
    fn test_rule_conf_to_message_unspec_action() {
        let conf = rule_conf_from_yaml(
            r#"
            address-family: ipv4
            action: unspec
            priority: 1007
            "#,
        );

        assert!(rule_conf_to_message(&conf).is_err());
    }

    #[test]
    fn test_rule_conf_action_is_required() {
        assert!(
            serde_yaml::from_str::<RouteRuleConf>(
                r#"
                address-family: ipv4
                src: 198.51.100.0/24
                priority: 1007
                "#
            )
            .is_err()
        );
    }

    #[test]
    fn test_rule_conf_priority_is_required() {
        assert!(
            serde_yaml::from_str::<RouteRuleConf>(
                r#"
                address-family: ipv4
                action: table
                src: 198.51.100.0/24
                "#
            )
            .is_err()
        );
    }
}
