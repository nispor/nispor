use clap::{clap_app, crate_authors, crate_version};
use nispor::{Iface, NetState, NisporError, Route, RouteRule};
use serde_derive::Serialize;
use serde_json;
use serde_yaml;
use std::fmt::{Display, Formatter, Result};
use std::process;

#[derive(Serialize)]
pub struct CliError {
    pub msg: String,
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.msg)
    }
}

enum CliResult {
    Full(NetState),
    Ifaces(Vec<Iface>),
    Routes(Vec<Route>),
    RouteRules(Vec<RouteRule>),
    Error(CliError),
    NisporError(NisporError),
}

enum CliOutputType {
    Json,
    Yaml,
}

macro_rules! npc_print {
    ($display_func:expr, $data: expr) => {
        match $data {
            CliResult::Full(netstate) => {
                println!("{}", $display_func(&netstate).unwrap());
                process::exit(0);
            }
            CliResult::Ifaces(ifaces) => {
                println!("{}", $display_func(&ifaces).unwrap());
                process::exit(0);
            }
            CliResult::Routes(routes) => {
                println!("{}", $display_func(&routes).unwrap());
                process::exit(0);
            }
            CliResult::RouteRules(rules) => {
                println!("{}", $display_func(&rules).unwrap());
                process::exit(0);
            }
            CliResult::NisporError(e) => {
                eprintln!("{}", $display_func(&e).unwrap());
                process::exit(1);
            }
            CliResult::Error(e) => {
                eprintln!("{}", $display_func(&e).unwrap());
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

fn main() {
    let matches = clap_app!(npc =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: "Nispor CLI")
        (@arg ifname: [INTERFACE_NAME] "interface name")
        (@arg json: -j --json "Show in json format")
        (@subcommand route =>
            (@arg json: -j --json "Show in json format")
            (about: "Show routes")
        )
        (@subcommand rule =>
            (@arg json: -j --json "Show in json format")
            (about: "Show routes rules")
        )
    )
    .get_matches();

    let mut output_format = parse_arg_output_format(&matches);

    let result = match NetState::retrieve() {
        Ok(mut state) => {
            if let Some(ifname) = matches.value_of("ifname") {
                if let Some(iface) = state.ifaces.remove(ifname) {
                    CliResult::Ifaces(vec![iface])
                } else {
                    CliResult::Error(CliError {
                        msg: format!("Interface '{}' not found", ifname),
                    })
                }
            } else if let Some(m) = matches.subcommand_matches("route") {
                output_format = parse_arg_output_format(m);
                CliResult::Routes(state.routes)
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
