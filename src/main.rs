use getopts;

use std::collections::HashMap;
use std::env;
use varlink::{Connection, OrgVarlinkServiceInterface, VarlinkService};

use crate::info_grisge_zatel::*;
mod info_grisge_zatel;

struct MyInfoGrisgeZatel;

impl VarlinkInterface for MyInfoGrisgeZatel {
    fn get(&self, call: &mut dyn Call_Get) -> Result<()> {
        let iface_state = IfaceState {
            name: "test".into(),
            iface_type: "unknown".into(),
            state: "up".into(),
            mtu: 0u32,
        };
        let mut iface_states: HashMap<&str, IfaceState> = HashMap::new();
        iface_states.insert("test", iface_state);
        return call.reply(NetState {
            iface_states: iface_states,
        });
    }
}

fn print_usage(program: &str, opts: &getopts::Options) {
    let brief = format!("Usage: {} [--varlink=<address>]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optopt("", "varlink", "varlink address URL", "<address>");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("{}", f.to_string());
            print_usage(&program, &opts);
            return;
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return;
    }

    if let Some(address) = matches.opt_str("varlink") {
        run_server(&address, 10).map_err(|e| e.into())
    } else {
        print_usage(&program, &opts);
        eprintln!("Need varlink address in server mode.");
        exit(1);
    };

    exit(match ret {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}

struct MyInfoGrisgeZatelNetwork {
    pub state: Arc<RwLock<i64>>,
}

impl info_grisge_zatel_network::VarlinkInterface for MyInfoGrisgeZatelNetwork {
    fn info(
        &self,
        call: &mut dyn info_grisge_zatel_network::Call_Info,
        ifindex: i64,
    ) -> varlink::Result<()> {
        // State example
        {
            let mut number = self.state.write().unwrap();

            *number += 1;

            eprintln!("{}", *number);
        }

        match ifindex {
            1 => call.reply(info_grisge_zatel_network::NetdevInfo {
                ifindex: 1,
                ifname: "lo".into(),
            }),
            2 => call.reply(info_grisge_zatel_network::NetdevInfo {
                ifindex: 2,
                ifname: "eth".into(),
            }),
            3 => {
                call.reply_invalid_parameter("ifindex".into())?;
                Ok(())
            }
            _ => call.reply_unknown_network_if_index(ifindex),
        }
    }

    fn list(
        &self,
        call: &mut dyn info_grisge_zatel_network::Call_List,
    ) -> varlink::Result<()> {
        // State example
        {
            let mut number = self.state.write().unwrap();

            *number -= 1;

            eprintln!("{}", *number);
        }
        call.reply(vec![
            info_grisge_zatel_network::Netdev {
                ifindex: 1,
                ifname: "lo".into(),
            },
            info_grisge_zatel_network::Netdev {
                ifindex: 2,
                ifname: "eth0".into(),
            },
        ])
    }
}

fn run_server<S: ?Sized + AsRef<str>>(
    address: &S,
    timeout: u64,
) -> varlink::Result<()> {
    let state = Arc::new(RwLock::new(0));
    let myiosystemdnetwork = MyInfoGrisgeZatelNetwork { state };
    let myinterface =
        info_grisge_zatel_network::new(Box::new(myiosystemdnetwork));
    let service = VarlinkService::new(
        "org.varlink",
        "test service",
        "0.1",
        "http://varlink.org",
        vec![Box::new(myinterface)],
    );

    varlink::listen(
        service,
        address,
        &varlink::ListenConfig {
            idle_timeout: timeout,
            ..Default::default()
        },
    )?;
    Ok(())
}
