// SPDX-License-Identifier: Apache-2.0

use clap::{crate_authors, crate_version};
use nispor::{
    Iface, IfaceConf, IfaceState, IfaceType, Mptcp, NetConf, NetState,
    NetStateFilter, NetStateIfaceFilter, NetStateRouteFilter,
    NetStateRouteRuleFilter, NisporError, Route, RouteProtocol, RouteRule,
    RouteScope,
};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write as _FmtWrite;
use std::io::{stderr, stdout, Write};
use std::process;

const INDENT: &str = "    ";
const LIST_SPLITER: &str = ",";
const RT_TABLE_MAIN: u8 = 254;
const RT_TABLE_LOCAL: u8 = 255;

#[derive(Serialize, Debug)]
pub struct CliError {
    pub error: String,
}

impl From<String> for CliError {
    fn from(error: String) -> Self {
        Self { error }
    }
}

impl From<NisporError> for CliError {
    fn from(e: NisporError) -> Self {
        Self {
            error: format!("{e}"),
        }
    }
}

#[derive(Serialize, Default)]
struct CliIfaceBrief {
    index: u32,
    name: String,
    iface_type: IfaceType,
    controller: Option<String>,
    link_info: String,
    state: IfaceState,
    flags: Vec<String>,
    mac: String,
    permanent_mac: String,
    mtu: i64,
    ipv4: Vec<String>,
    ipv6: Vec<String>,
    ipv6_token: Option<String>,
    gw4: Vec<String>,
    gw6: Vec<String>,
}

impl CliIfaceBrief {
    fn list_show(briefs: &[CliIfaceBrief]) -> String {
        let mut ret = Vec::new();
        for brief in briefs {
            ret.push(format!(
                "{: >2}: {}: <{}> state {} mtu {}",
                brief.index,
                brief.name,
                brief.flags.join(","),
                brief.state,
                brief.mtu,
            ));
            let mut link_string =
                format!("{}link {}", INDENT, brief.iface_type);

            if !brief.link_info.is_empty() {
                write!(link_string, " {}", brief.link_info.as_str()).ok();
            }
            if let Some(ctrl) = brief.controller.as_ref() {
                write!(link_string, " controller {ctrl}").ok();
            }

            ret.push(link_string);

            let mut mac_string = String::new();
            if !&brief.mac.is_empty() {
                write!(mac_string, "{}mac {}", INDENT, brief.mac).ok();
                if !&brief.permanent_mac.is_empty() {
                    write!(
                        mac_string,
                        " permanent_mac {}",
                        brief.permanent_mac
                    )
                    .ok();
                }
            }

            if !mac_string.is_empty() {
                ret.push(mac_string);
            }

            for ip in &brief.ipv4 {
                ret.push(format!("{INDENT}ipv4 {ip}"));
            }
            for gw in &brief.gw4 {
                ret.push(format!("{INDENT}gw4 {gw}"));
            }
            for ip in &brief.ipv6 {
                ret.push(format!("{INDENT}ipv6 {ip}"));
            }
            if let Some(token) = brief.ipv6_token.as_ref() {
                ret.push(format!("{INDENT}ipv6_token {token}"));
            }
            for gw in &brief.gw6 {
                ret.push(format!("{INDENT}gw6 {gw}"));
            }
        }
        ret.join("\n")
    }

    fn from_net_state(netstate: &NetState) -> Vec<Self> {
        let mut ret = Vec::new();
        let mut iface_to_gw4: HashMap<String, Vec<String>> = HashMap::new();
        let mut iface_to_gw6: HashMap<String, Vec<String>> = HashMap::new();

        for route in &netstate.routes {
            if let Route {
                dst: None,
                gateway: Some(gw),
                oif: Some(iface_name),
                ..
            } = route
            {
                if gw.contains(':') {
                    match iface_to_gw6.get_mut(iface_name) {
                        Some(gateways) => {
                            gateways.push(gw.to_string());
                        }
                        None => {
                            iface_to_gw6.insert(
                                iface_name.to_string(),
                                vec![gw.to_string()],
                            );
                        }
                    }
                } else {
                    match iface_to_gw4.get_mut(iface_name) {
                        Some(gateways) => {
                            gateways.push(gw.to_string());
                        }
                        None => {
                            iface_to_gw4.insert(
                                iface_name.to_string(),
                                vec![gw.to_string()],
                            );
                        }
                    }
                }
            }
        }

        for iface in netstate.ifaces.values() {
            ret.push(CliIfaceBrief {
                index: iface.index,
                iface_type: iface.iface_type.clone(),
                controller: iface.controller.clone(),
                link_info: get_link_info(iface),
                name: iface.name.clone(),
                flags: iface
                    .flags
                    .iter()
                    .map(|flag| format!("{flag:?}").to_uppercase())
                    .collect(),
                state: iface.state.clone(),
                mac: iface.mac_address.clone(),
                permanent_mac: iface.permanent_mac_address.clone(),
                mtu: iface.mtu,
                ipv4: match &iface.ipv4 {
                    Some(ip_info) => {
                        let mut addr_strs = Vec::new();
                        for addr in &ip_info.addresses {
                            addr_strs.push(format!(
                                "{}/{} valid_lft {} preferred_lft {}",
                                addr.address,
                                addr.prefix_len,
                                addr.valid_lft,
                                addr.preferred_lft,
                            ));
                        }
                        addr_strs
                    }
                    None => Vec::new(),
                },
                ipv6: match &iface.ipv6 {
                    Some(ip_info) => {
                        let mut addr_strs = Vec::new();
                        for addr in &ip_info.addresses {
                            addr_strs.push(format!(
                                "{}/{} valid_lft {} preferred_lft {}",
                                addr.address,
                                addr.prefix_len,
                                addr.valid_lft,
                                addr.preferred_lft,
                            ));
                        }
                        addr_strs
                    }
                    None => Vec::new(),
                },
                ipv6_token: iface
                    .ipv6
                    .as_ref()
                    .and_then(|i| i.token.as_ref())
                    .map(|t| t.to_string()),
                gw4: match &iface_to_gw4.get(&iface.name) {
                    Some(gws) => gws.to_vec(),
                    None => Vec::new(),
                },
                gw6: match &iface_to_gw6.get(&iface.name) {
                    Some(gws) => gws.to_vec(),
                    None => Vec::new(),
                },
            })
        }
        ret.sort_by(|a, b| a.index.cmp(&b.index));
        ret
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

enum CliReply {
    Pass,
    Brief(Vec<CliIfaceBrief>),
    Full(NetState),
    Ifaces(Vec<Iface>),
    Routes(Vec<Route>),
    RouteRules(Vec<RouteRule>),
    Mptcp(Mptcp),
}

#[derive(PartialEq, Eq)]
enum CliOutputType {
    Json,
    Yaml,
}

macro_rules! npc_print {
    ($display_func:expr, $data: expr) => {
        match $data {
            CliReply::Pass => {
                process::exit(0);
            }
            CliReply::Brief(_) => unreachable!(),
            CliReply::Full(netstate) => {
                writeln!(stdout(), "{}", $display_func(&netstate).unwrap())
                    .ok();
                process::exit(0);
            }
            CliReply::Ifaces(ifaces) => {
                writeln!(stdout(), "{}", $display_func(&ifaces).unwrap()).ok();
                process::exit(0);
            }
            CliReply::Routes(routes) => {
                writeln!(stdout(), "{}", $display_func(&routes).unwrap()).ok();
                process::exit(0);
            }
            CliReply::RouteRules(rules) => {
                writeln!(stdout(), "{}", $display_func(&rules).unwrap()).ok();
                process::exit(0);
            }
            CliReply::Mptcp(mptcp) => {
                writeln!(stdout(), "{}", $display_func(&mptcp).unwrap()).ok();
                process::exit(0);
            }
        }
    };
}

fn print_result(
    result: Result<CliReply, CliError>,
    output_type: CliOutputType,
) {
    match result {
        Ok(result) => {
            if let CliReply::Brief(briefs) = result {
                if output_type == CliOutputType::Json {
                    writeln!(
                        stdout(),
                        "{}",
                        serde_json::to_string_pretty(&briefs).unwrap()
                    )
                    .ok();
                } else {
                    writeln!(stdout(), "{}", CliIfaceBrief::list_show(&briefs))
                        .ok();
                }
            } else {
                match output_type {
                    CliOutputType::Json => {
                        npc_print!(serde_json::to_string_pretty, result)
                    }
                    CliOutputType::Yaml => {
                        npc_print!(serde_yaml::to_string, result)
                    }
                }
            }
        }
        Err(e) => {
            writeln!(
                stderr(),
                "{}",
                match output_type {
                    CliOutputType::Json =>
                        serde_json::to_string_pretty(&e).unwrap(),
                    CliOutputType::Yaml => serde_yaml::to_string(&e).unwrap(),
                }
            )
            .ok();
        }
    }
}

fn parse_arg_output_format(matches: &clap::ArgMatches) -> CliOutputType {
    match matches.contains_id("json") {
        true => CliOutputType::Json,
        false => CliOutputType::Yaml,
    }
}

fn _is_route_to_specified_dev(route: &Route, iface_name: &str) -> bool {
    if let Some(oif) = &route.oif {
        if oif == iface_name {
            return true;
        }
    }
    if let Some(mp_routes) = &route.multipath {
        for mp_route in mp_routes {
            if mp_route.iface == iface_name {
                return true;
            }
        }
    }

    false
}

fn main() {
    let matches = clap::Command::new("npc")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Nispor CLI")
        .arg(
            clap::Arg::new("verbose")
                .short('v')
                .action(clap::ArgAction::Count)
                .help("Set verbose level"),
        )
        .arg(
            clap::Arg::new("json")
                .short('j')
                .action(clap::ArgAction::Append)
                .global(true)
                .help("Show in json format"),
        )
        .arg(
            clap::Arg::new("iface_name")
                .index(1)
                .help("Show specific interface only"),
        )
        .subcommand(clap::Command::new("full").about("Full network state"))
        .subcommand(
            clap::Command::new("iface")
                .about("Show interface")
                .arg(
                    clap::Arg::new("iface_name")
                        .index(1)
                        .help("Show specific interface only"),
                )
                .arg(
                    clap::Arg::new("delete")
                        .long("delete")
                        .help("Delete the specified interface"),
                ),
        )
        .subcommand(
            clap::Command::new("route")
                .about("Show route")
                .arg(
                    clap::Arg::new("dev")
                        .short('d')
                        .long("dev")
                        .action(clap::ArgAction::Append)
                        .help(
                            "Show only route entries output to \
                            the specified interface",
                        ),
                )
                .arg(
                    clap::Arg::new("table")
                        .short('t')
                        .long("table")
                        .action(clap::ArgAction::Append)
                        .help(
                            "Show only route entries output in \
                            the specified route table",
                        ),
                )
                .arg(
                    clap::Arg::new("scope")
                        .short('s')
                        .long("scope")
                        .action(clap::ArgAction::Append)
                        .help("Show only route entries with specified scope")
                        .value_parser([
                            "a", "all", "u", "universe", "g", "global", "s",
                            "site", "l", "link", "h", "host", "n", "nowhere",
                            "no_where",
                        ]),
                )
                .arg(
                    clap::Arg::new("protocol")
                        .short('p')
                        .long("protocol")
                        .action(clap::ArgAction::Append)
                        .help("Show only route with specified protocol")
                        .value_parser([
                            "icmp_redirect",
                            "kernel",
                            "boot",
                            "static",
                            "gated",
                            "ra",
                            "merit_mrt",
                            "zebra",
                            "bird",
                            "decnet_routing_daemon",
                            "xorp",
                            "netsukuku",
                            "Dhcp",
                            "multicast_daemon",
                            "keepalived_daemon",
                            "babel",
                            "bgp",
                            "isis",
                            "ospf",
                            "rip",
                            "eigrp",
                        ]),
                ),
        )
        .subcommand(clap::Command::new("rule").about("Show route route"))
        .subcommand(clap::Command::new("mptcp").about("Show mptcp state"))
        .subcommand(
            clap::Command::new("set")
                .about("Set network state from file")
                .arg(
                    clap::Arg::new("file_path")
                        .required(true)
                        .index(1)
                        .help("Network state file to apply"),
                ),
        )
        .get_matches();

    let (log_module_filter, log_level) = match matches.get_count("verbose") {
        0 => (Some("nispor"), log::LevelFilter::Warn),
        1 => (Some("nispor"), log::LevelFilter::Info),
        2 => (Some("nispor"), log::LevelFilter::Debug),
        _ => (None, log::LevelFilter::Debug),
    };

    let mut log_builder = env_logger::Builder::new();
    log_builder.filter(log_module_filter, log_level);
    log_builder.init();

    let mut output_format = parse_arg_output_format(&matches);

    if let Some(m) = matches.subcommand_matches("set") {
        if let Some(file_path) = m.get_one::<String>("file_path") {
            print_result(apply_conf(file_path), output_format);
            process::exit(0);
        } else {
            log::warn!("file path undefined");
            process::exit(1);
        }
    } else if let Some(m) = matches.subcommand_matches("full") {
        output_format = parse_arg_output_format(m);
        print_result(get_full(), output_format);
    } else if let Some(m) = matches.subcommand_matches("iface") {
        output_format = parse_arg_output_format(m);
        print_result(get_ifaces(m), output_format);
    } else if let Some(m) = matches.subcommand_matches("route") {
        output_format = parse_arg_output_format(m);
        print_result(get_routes(m), output_format);
    } else if let Some(m) = matches.subcommand_matches("rule") {
        output_format = parse_arg_output_format(m);
        print_result(get_rules(), output_format);
    } else if let Some(m) = matches.subcommand_matches("mptcp") {
        output_format = parse_arg_output_format(m);
        print_result(get_mptcp(), output_format);
    } else {
        print_result(get_brief(&matches), output_format);
    }
}

fn apply_conf(file_path: &str) -> Result<CliReply, CliError> {
    let fd = match std::fs::File::open(file_path) {
        Ok(fd) => fd,
        Err(e) => {
            return Err(format!("Filed to open file {file_path}: {e}").into());
        }
    };
    let net_conf: NetConf = match serde_yaml::from_reader(fd) {
        Ok(c) => c,
        Err(e) => {
            return Err(format!("Invalid YAML file {file_path}: {e}",).into());
        }
    };
    net_conf.apply()?;
    if let Some(desire_ifaces) = net_conf.ifaces {
        let mut desired_iface_names = Vec::new();
        for iface_conf in &desire_ifaces {
            desired_iface_names.push(iface_conf.name.clone());
        }
        Ok(CliReply::Ifaces(filter_iface_state(
            NetState::retrieve()?,
            desired_iface_names,
        )))
    } else {
        Ok(CliReply::Pass)
    }
}

fn filter_iface_state(
    cur_state: NetState,
    des_iface_names: Vec<String>,
) -> Vec<Iface> {
    let mut new_ifaces = Vec::new();
    for (iface_name, iface_state) in cur_state.ifaces.iter() {
        if des_iface_names.contains(iface_name) {
            new_ifaces.push(iface_state.clone());
        }
    }
    new_ifaces
}

fn delete_iface(iface_name: &str) -> Result<CliReply, CliError> {
    let mut conf = NetConf::default();
    let mut iface_conf = IfaceConf::default();
    iface_conf.name = iface_name.to_string();
    iface_conf.state = IfaceState::Absent;
    conf.ifaces = Some(vec![iface_conf]);
    conf.apply()?;
    Ok(CliReply::Pass)
}

fn get_link_info(iface: &Iface) -> String {
    if let Some(bond) = iface.bond.as_ref() {
        let mut bond_line = format!(
            "mode {} ports {}",
            bond.mode,
            bond.subordinates.join(LIST_SPLITER)
        );
        if let Some(p) = bond.primary.as_deref() {
            write!(bond_line, " primary {p}").ok();
        }
        bond_line
    } else if let Some(bridge) = iface.bridge.as_ref() {
        format!("ports {}", bridge.ports.join(LIST_SPLITER))
    } else if let Some(vrf) = iface.vrf.as_ref() {
        format!(
            "table {} ports {}",
            vrf.table_id,
            vrf.subordinates.join(LIST_SPLITER)
        )
    } else if let Some(veth) = iface.veth.as_ref() {
        format!("peer {}", veth.peer)
    } else if let Some(vlan) = iface.vlan.as_ref() {
        format!("parent {} id {}", vlan.base_iface, vlan.vlan_id)
    } else if let Some(vxlan) = iface.vxlan.as_ref() {
        format!(
            "parent {} id {} remote {} dst_port {} local {}",
            vxlan.base_iface,
            vxlan.vxlan_id,
            vxlan.remote,
            vxlan.dst_port,
            vxlan.local
        )
    } else {
        "".into()
    }
}

fn get_ifaces(matches: &clap::ArgMatches) -> Result<CliReply, CliError> {
    if let Some(iface_name) = matches.get_one::<String>("iface_name") {
        let mut filter = NetStateFilter::minimum();
        let mut iface_filter = NetStateIfaceFilter::default();
        // In order to get controller/port relation ship,
        // We do not request filter on interface name.
        iface_filter.include_ip_address = true;
        iface_filter.include_sriov_vf_info = true;
        iface_filter.include_bridge_vlan = true;
        iface_filter.include_ethtool = true;
        iface_filter.include_mptcp = true;
        filter.iface = Some(iface_filter);

        let state = NetState::retrieve_with_filter(&filter)?;

        if let Some(iface) = state.ifaces.get(iface_name) {
            if matches.contains_id("delete") {
                delete_iface(&iface.name)
            } else {
                Ok(CliReply::Ifaces(vec![iface.clone()]))
            }
        } else {
            Err(format!("Interface '{iface_name}' not found").into())
        }
    } else if matches.contains_id("delete") {
        Err("Need to specific a interface to delete".to_string().into())
    } else {
        let state = NetState::retrieve()?;
        Ok(CliReply::Ifaces(state.ifaces.values().cloned().collect()))
    }
}

fn get_routes(matches: &clap::ArgMatches) -> Result<CliReply, CliError> {
    let mut route_filter = NetStateRouteFilter::default();

    if let Some(scope) = matches.get_one::<String>("scope") {
        if scope != "a" && scope != "all" {
            let rt_scope = RouteScope::from(scope.as_str());
            if rt_scope == RouteScope::Unknown {
                return Err(format!("Invalid scope {scope}").into());
            }
            route_filter.scope = Some(rt_scope);
        }
    }

    if let Some(protocol) = matches.get_one::<String>("protocol") {
        let rt_protocol = RouteProtocol::from(protocol.as_str());
        if rt_protocol == RouteProtocol::Unknown {
            return Err(format!("Invalid protocol {protocol}").into());
        }
        route_filter.protocol = Some(rt_protocol);
    }

    if let Some(table) = matches.get_one::<String>("table") {
        route_filter.table = Some(match table.as_str() {
            "main" => RT_TABLE_MAIN,
            "local" => RT_TABLE_LOCAL,
            _ => table.parse::<u8>().map_err(|e| CliError {
                error: format!("{e}"),
            })?,
        });
    }

    if let Some(iface_name) = matches.get_one::<String>("dev") {
        route_filter.oif = Some(iface_name.to_string());
    }

    let mut filter = NetStateFilter::minimum();
    filter.route = Some(route_filter);

    let state = NetState::retrieve_with_filter(&filter)?;

    Ok(CliReply::Routes(state.routes))
}

fn get_rules() -> Result<CliReply, CliError> {
    let mut filter = NetStateFilter::minimum();
    filter.route_rule = Some(NetStateRouteRuleFilter::default());
    let state = NetState::retrieve_with_filter(&filter)?;
    Ok(CliReply::RouteRules(state.rules))
}

fn get_mptcp() -> Result<CliReply, CliError> {
    let mut iface_filter = NetStateIfaceFilter::minimum();
    iface_filter.include_mptcp = true;
    let mut filter = NetStateFilter::minimum();
    filter.iface = Some(iface_filter);
    let state = NetState::retrieve_with_filter(&filter)?;
    Ok(CliReply::Mptcp(state.mptcp.unwrap_or_default()))
}

fn get_brief(matches: &clap::ArgMatches) -> Result<CliReply, CliError> {
    let mut filter = NetStateFilter::minimum();
    let mut iface_filter = NetStateIfaceFilter::minimum();
    // In order to get controller/port relation ship,
    // We do not request filter on interface name.
    iface_filter.include_ip_address = true;
    filter.iface = Some(iface_filter);
    let mut route_filter = NetStateRouteFilter::default();
    route_filter.table = Some(RT_TABLE_MAIN);
    if let Some(iface_name) = matches.get_one::<String>("iface_name") {
        route_filter.oif = Some(iface_name.to_string());
    }
    filter.route = Some(route_filter);
    filter.route_rule = None;
    let state = NetState::retrieve_with_filter(&filter)?;

    if let Some(iface_name) = matches.get_one::<String>("iface_name") {
        if state.ifaces.contains_key(iface_name) {
            for iface_brief in CliIfaceBrief::from_net_state(&state) {
                if &iface_brief.name == iface_name {
                    return Ok(CliReply::Brief(vec![iface_brief]));
                }
            }
            Err(format!(
                "BUG: Interface '{iface_name}' not found in CliIfaceBrief"
            )
            .into())
        } else {
            Err(format!("Interface '{iface_name}' not found").into())
        }
    } else {
        /* Show everything if no cmdline arg has been supplied */
        Ok(CliReply::Brief(CliIfaceBrief::from_net_state(&state)))
    }
}

fn get_full() -> Result<CliReply, CliError> {
    Ok(CliReply::Full(NetState::retrieve()?))
}
