// Copyright 2021 Red Hat, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use clap::{crate_authors, crate_version};
use nispor::{
    Iface, IfaceConf, IfaceState, NetConf, NetState, NisporError, Route,
    RouteRule,
};
use serde_derive::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::io::{stderr, stdout, Write};
use std::process;

const INDENT: &str = "    ";

#[derive(Serialize, Debug)]
pub struct CliError {
    pub msg: String,
}

#[derive(Serialize, Default)]
struct CliIfaceBrief {
    index: u32,
    name: String,
    state: IfaceState,
    flags: Vec<String>,
    mac: String,
    permanent_mac: String,
    mtu: i64,
    ipv4: Vec<String>,
    ipv6: Vec<String>,
    gw4: Vec<String>,
    gw6: Vec<String>,
}

impl CliIfaceBrief {
    fn list_show(briefs: &[CliIfaceBrief]) -> String {
        let mut ret = Vec::new();
        for brief in briefs {
            ret.push(format!(
                "{}: {}: <{}> state {} mtu {}",
                brief.index,
                brief.name,
                brief.flags.join(","),
                brief.state,
                brief.mtu,
            ));
            if !&brief.mac.is_empty() {
                ret.push(format!(
                    "{}mac {}{}",
                    INDENT,
                    brief.mac,
                    if !&brief.permanent_mac.is_empty()
                        && brief.permanent_mac != brief.mac
                    {
                        format!(" permanent_mac: {}", brief.permanent_mac)
                    } else {
                        "".into()
                    }
                ));
            }
            if !brief.ipv4.is_empty() {
                ret.push(format!(
                    "{}ipv4 {}{}",
                    INDENT,
                    brief.ipv4.join(" "),
                    if !brief.gw4.is_empty() {
                        format!(" gw4 {}", brief.gw4.join(" "))
                    } else {
                        "".into()
                    },
                ));
            }
            if !brief.ipv6.is_empty() {
                ret.push(format!(
                    "{}ipv6 {}{}",
                    INDENT,
                    brief.ipv6.join(" "),
                    if !brief.gw6.is_empty() {
                        format!(" gw6 {}", brief.gw6.join(" "))
                    } else {
                        "".into()
                    }
                ));
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
                name: iface.name.clone(),
                flags: (&iface.flags)
                    .iter()
                    .map(|flag| format!("{:?}", flag).to_uppercase())
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
                                "{}/{} {} {}",
                                addr.address,
                                addr.prefix_len,
                                addr.preferred_lft,
                                addr.valid_lft
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
                                "{}/{} {} {}",
                                addr.address,
                                addr.prefix_len,
                                addr.preferred_lft,
                                addr.valid_lft
                            ));
                        }
                        addr_strs
                    }
                    None => Vec::new(),
                },
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
        write!(f, "{}", self.msg)
    }
}

enum CliResult {
    Pass,
    Brief(Vec<CliIfaceBrief>),
    Full(NetState),
    Ifaces(Vec<Iface>),
    Routes(Vec<Route>),
    RouteRules(Vec<RouteRule>),
    CliError(CliError),
    NisporError(NisporError),
}

#[derive(PartialEq)]
enum CliOutputType {
    Json,
    Yaml,
}

macro_rules! npc_print {
    ($display_func:expr, $data: expr) => {
        match $data {
            CliResult::Pass => {
                process::exit(0);
            }
            CliResult::Brief(_) => unreachable!(),
            CliResult::Full(netstate) => {
                writeln!(stdout(), "{}", $display_func(&netstate).unwrap())
                    .ok();
                process::exit(0);
            }
            CliResult::Ifaces(ifaces) => {
                writeln!(stdout(), "{}", $display_func(&ifaces).unwrap()).ok();
                process::exit(0);
            }
            CliResult::Routes(routes) => {
                writeln!(stdout(), "{}", $display_func(&routes).unwrap()).ok();
                process::exit(0);
            }
            CliResult::RouteRules(rules) => {
                writeln!(stdout(), "{}", $display_func(&rules).unwrap()).ok();
                process::exit(0);
            }
            CliResult::NisporError(e) => {
                writeln!(stderr(), "{}", $display_func(&e).unwrap()).ok();
                process::exit(1);
            }
            CliResult::CliError(e) => {
                writeln!(stderr(), "{}", $display_func(&e).unwrap()).ok();
                process::exit(1);
            }
        }
    };
}

fn print_result(result: &CliResult, output_type: CliOutputType) {
    if let CliResult::Brief(briefs) = result {
        if output_type == CliOutputType::Json {
            writeln!(
                stdout(),
                "{}",
                serde_json::to_string_pretty(&briefs).unwrap()
            )
            .ok();
        } else {
            writeln!(stdout(), "{}", CliIfaceBrief::list_show(briefs)).ok();
        }
    } else {
        match output_type {
            CliOutputType::Json => {
                npc_print!(serde_json::to_string_pretty, result)
            }
            CliOutputType::Yaml => npc_print!(serde_yaml::to_string, result),
        }
    }
}

fn parse_arg_output_format(matches: &clap::ArgMatches) -> CliOutputType {
    match matches.is_present("json") {
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

fn get_routes(state: &NetState, matches: &clap::ArgMatches) -> CliResult {
    let mut routes = state.routes.clone();

    if let Some(iface_name) = matches.value_of("dev") {
        routes = routes
            .into_iter()
            .filter(|route| _is_route_to_specified_dev(route, iface_name))
            .collect();
    }

    CliResult::Routes(routes)
}

fn main() {
    let matches = clap::App::new("npc")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Nispor CLI")
        .arg(
            clap::Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("Set verbose level"),
        )
        .arg(
            clap::Arg::with_name("json")
                .short("j")
                .takes_value(false)
                .global(true)
                .help("Show in json format"),
        )
        .arg(
            clap::Arg::with_name("iface_name")
                .index(1)
                .help("Show speific interface only"),
        )
        .subcommand(
            clap::SubCommand::with_name("iface")
                .about("Show interface")
                .arg(
                    clap::Arg::with_name("iface_name")
                        .index(1)
                        .help("Show specific interface only"),
                )
                .arg(
                    clap::Arg::with_name("delete")
                        .takes_value(false)
                        .help("Delete the specified interface"),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("route")
                .about("Show route")
                .arg(
                    clap::Arg::with_name("dev")
                        .short("d")
                        .takes_value(true)
                        .help(
                            "Show only route entries output to \
                            the specified interface",
                        ),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("rule").about("Show route route"),
        )
        .subcommand(
            clap::SubCommand::with_name("set")
                .about("Set network state from file")
                .arg(
                    clap::Arg::with_name("file_path")
                        .required(true)
                        .index(1)
                        .help("Network state file to apply"),
                ),
        )
        .get_matches();

    let (log_module_filter, log_level) = match matches.occurrences_of("verbose")
    {
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
        if let Some(file_path) = m.value_of("file_path") {
            print_result(&apply_conf(file_path), output_format);
            process::exit(0);
        } else {
            log::warn!("file path undefined");
            process::exit(1);
        }
    } else {
        let result = match NetState::retrieve() {
            Ok(mut state) => {
                if let Some(m) = matches.subcommand_matches("iface") {
                    output_format = parse_arg_output_format(m);
                    if let Some(iface_name) = m.value_of("iface_name") {
                        if let Some(iface) = state.ifaces.remove(iface_name) {
                            if m.is_present("delete") {
                                delete_iface(&iface.name)
                            } else {
                                CliResult::Ifaces(vec![iface])
                            }
                        } else {
                            CliResult::CliError(CliError {
                                msg: format!(
                                    "Interface '{}' not found",
                                    iface_name
                                ),
                            })
                        }
                    } else if matches.is_present("delete") {
                        CliResult::CliError(CliError {
                            msg: "Need to specific a interface to delete"
                                .to_string(),
                        })
                    } else {
                        CliResult::Full(state)
                    }
                } else if let Some(m) = matches.subcommand_matches("route") {
                    output_format = parse_arg_output_format(m);
                    get_routes(&state, m)
                } else if let Some(m) = matches.subcommand_matches("rule") {
                    output_format = parse_arg_output_format(m);
                    CliResult::RouteRules(state.rules)
                } else if let Some(iface_name) = matches.value_of("iface_name")
                {
                    if state.ifaces.get(iface_name).is_some() {
                        let mut iface_briefs = Vec::new();
                        for iface_brief in CliIfaceBrief::from_net_state(&state)
                        {
                            if iface_brief.name == iface_name {
                                iface_briefs.push(iface_brief);
                                break;
                            }
                        }
                        if iface_briefs.is_empty() {
                            CliResult::CliError(CliError {
                            msg: format!(
                                "BUG: Interface '{}' not found in CliIfaceBrief",
                                iface_name
                            ),
                        })
                        } else {
                            CliResult::Brief(iface_briefs)
                        }
                    } else {
                        CliResult::CliError(CliError {
                            msg: format!(
                                "Interface '{}' not found",
                                iface_name
                            ),
                        })
                    }
                } else {
                    /* Show everything if no cmdline arg has been supplied */
                    CliResult::Brief(CliIfaceBrief::from_net_state(&state))
                }
            }
            Err(e) => CliResult::NisporError(e),
        };
        print_result(&result, output_format);
    }
}

fn apply_conf(file_path: &str) -> CliResult {
    let fd = match std::fs::File::open(file_path) {
        Ok(fd) => fd,
        Err(e) => {
            return CliResult::CliError(CliError {
                msg: format!("Filed to open file {}: {}", file_path, e),
            })
        }
    };
    let net_conf: NetConf = match serde_yaml::from_reader(fd) {
        Ok(c) => c,
        Err(e) => {
            return CliResult::CliError(CliError {
                msg: format!("Invalid YAML file {}: {}", file_path, e,),
            })
        }
    };
    if let Err(e) = net_conf.apply() {
        return CliResult::NisporError(e);
    }
    if let Some(desire_ifaces) = net_conf.ifaces {
        match NetState::retrieve() {
            Ok(cur_state) => {
                let mut desired_iface_names = Vec::new();
                for iface_conf in &desire_ifaces {
                    desired_iface_names.push(iface_conf.name.clone());
                }
                CliResult::Ifaces(filter_iface_state(
                    cur_state,
                    desired_iface_names,
                ))
            }
            Err(e) => CliResult::NisporError(e),
        }
    } else {
        CliResult::Pass
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

fn delete_iface(iface_name: &str) -> CliResult {
    if let Err(e) = (NetConf {
        ifaces: Some(vec![IfaceConf {
            name: iface_name.to_string(),
            state: IfaceState::Absent,
            ..Default::default()
        }]),
    }
    .apply())
    {
        CliResult::NisporError(e)
    } else {
        CliResult::Pass
    }
}
