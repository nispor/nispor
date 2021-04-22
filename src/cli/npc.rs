use clap::{clap_app, crate_authors, crate_version, Arg};
use nispor::{
    Iface, NetConf, NetState, NisporError, Route, RouteRule, RouteScope,
};
use serde_derive::Serialize;
use serde_json;
use serde_yaml;
use std::fmt;
use std::io::{stderr, stdout, Write};
use std::process;

const ROUTE_SCOPE_STRINGS: [&str; 12] = [
    "all", "u", "universe", "global", "s", "site", "l", "link", "h", "host",
    "n", "nowhere",
];

#[derive(Serialize)]
pub struct CliError {
    pub msg: String,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

enum CliResult {
    Pass,
    Full(NetState),
    Ifaces(Vec<Iface>),
    Routes(Vec<Route>),
    RouteRules(Vec<RouteRule>),
    CliError(CliError),
    NisporError(NisporError),
}

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
    match output_type {
        CliOutputType::Json => npc_print!(serde_json::to_string_pretty, result),
        CliOutputType::Yaml => npc_print!(serde_yaml::to_string, result),
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

fn get_route_scope_from_str(
    route_scope_str: &str,
) -> Result<RouteScope, &'static str> {
    match route_scope_str {
        "universe" | "global" | "u" => Ok(RouteScope::Universe),
        "site" | "s" => Ok(RouteScope::Site),
        "link" | "l" => Ok(RouteScope::Link),
        "host" | "h" => Ok(RouteScope::Host),
        "nowhere" | "n" => Ok(RouteScope::NoWhere),
        _ => Err("Invalid route scope"),
    }
}

fn get_routes(
    state: &NetState,
    matches: &clap::ArgMatches,
) -> Result<Vec<Route>, String> {
    let mut routes = state.routes.clone();
    let mut error: Option<String> = None;

    if let Some(iface_name) = matches.value_of("dev") {
        routes = routes
            .into_iter()
            .filter(|route| _is_route_to_specified_dev(route, iface_name))
            .collect();
    }

    if let Some(scope) = matches.value_of("scope") {
        if scope != "all" {
            match get_route_scope_from_str(scope) {
                Ok(route_scope) => {
                    routes = routes
                        .into_iter()
                        .filter(|route| route.scope == route_scope)
                        .collect()
                }
                Err(msg) => error = Some(String::from(msg)),
            }
        }
    }

    match error {
        None => Ok(routes),
        Some(error) => Err(error),
    }
}

fn main() {
    let mut app = clap_app!(npc =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: "Nispor CLI")
        (@arg ifname: [INTERFACE_NAME] "interface name")
        (@arg json: -j --json "Show in json format")
        (@subcommand rule =>
            (@arg json: -j --json "Show in json format")
            (about: "Show routes rules")
        )
        (@subcommand set =>
            (@arg file_path: [FILE_PATH] +required "config file to apply")
            (about: "Apply network config")
        )
    );

    let route_subcmd = clap_app!(
        @subcommand route =>
            (@arg json: -j --json "Show in json format")
            (@arg dev: -d --dev [OIF] "Show only route entries with output to the specified interface")
            (about: "Show routes")
    ).arg(
        Arg::with_name("scope")
            .short("s")
            .long("scope")
            .value_name("SCOPE")
            .required(false)
            .possible_values(&ROUTE_SCOPE_STRINGS)
            .help("Show only route entries that match the specified scope")
    );

    app = app.subcommand(route_subcmd);

    let matches = app.get_matches();

    let mut output_format = parse_arg_output_format(&matches);

    if let Some(m) = matches.subcommand_matches("set") {
        if let Some(file_path) = m.value_of("file_path") {
            print_result(&apply_conf(&file_path), output_format);
            process::exit(0);
        } else {
            eprintln!("file path undefined");
            process::exit(1);
        }
    } else {
        let result = match NetState::retrieve() {
            Ok(mut state) => {
                if let Some(ifname) = matches.value_of("ifname") {
                    if let Some(iface) = state.ifaces.remove(ifname) {
                        CliResult::Ifaces(vec![iface])
                    } else {
                        CliResult::CliError(CliError {
                            msg: format!("Interface '{}' not found", ifname),
                        })
                    }
                } else if let Some(m) = matches.subcommand_matches("route") {
                    output_format = parse_arg_output_format(m);
                    match get_routes(&state, &m) {
                        Ok(routes) => CliResult::Routes(routes),
                        Err(msg) => CliResult::CliError(CliError { msg }),
                    }
                } else if let Some(m) = matches.subcommand_matches("rule") {
                    output_format = parse_arg_output_format(m);
                    CliResult::RouteRules(state.rules)
                } else {
                    /* Show everything if no cmdline arg has been supplied */
                    CliResult::Full(state)
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
