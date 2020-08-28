use clap::{clap_app, crate_authors, crate_version};
use nispor::{Iface, NetState, NisporError, Route, RouteRule};
use serde_derive::Serialize;
use serde_json;
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

#[derive(Serialize)]
enum CliResult {
    Full(NetState),
    Ifaces(Vec<Iface>),
    Routes(Vec<Route>),
    RouteRules(Vec<RouteRule>),
    Error(CliError),
    NisporError(NisporError),
}

fn main() {
    let matches = clap_app!(npc =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: "Nispor CLI")
        (@arg ifname: [INTERFACE_NAME] "interface name")
        (@subcommand route =>
            (about: "Show routes")
        )
        (@subcommand rule =>
            (about: "Show routes rules")
        )
    )
    .get_matches();

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
            } else if let Some(_) = matches.subcommand_matches("route") {
                CliResult::Routes(state.routes)
            } else if let Some(_) = matches.subcommand_matches("rule") {
                CliResult::RouteRules(state.rules)
            } else {
                /* Show everything if no cmdline arg has been supplied */
                CliResult::Full(state)
            }
        }
        Err(e) => CliResult::NisporError(e),
    };

    match result {
        CliResult::Full(netstate) => {
            println!("{}", serde_json::to_string_pretty(&netstate).unwrap());
            process::exit(0);
        }
        CliResult::Ifaces(ifaces) => {
            println!("{}", serde_json::to_string_pretty(&ifaces).unwrap());
            process::exit(0);
        }
        CliResult::Routes(routes) => {
            println!("{}", serde_json::to_string_pretty(&routes).unwrap());
            process::exit(0);
        }
        CliResult::RouteRules(rules) => {
            println!("{}", serde_json::to_string_pretty(&rules).unwrap());
            process::exit(0);
        }
        CliResult::NisporError(e) => {
            eprintln!("{}", serde_json::to_string_pretty(&e).unwrap());
            process::exit(1);
        }
        CliResult::Error(e) => {
            eprintln!("{}", serde_json::to_string_pretty(&e).unwrap());
            process::exit(1);
        }
    }
}
