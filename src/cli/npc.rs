use nispor::NetState;
use serde_json;
use std::env::args;

fn main() {
    let argv: Vec<String> = args().collect();
    match NetState::retrieve() {
        Ok(state) => {
            if argv.len() > 1 {
                match &state.ifaces.get(&argv[1]) {
                    Some(iface) => println!(
                        "{}",
                        serde_json::to_string_pretty(iface).unwrap()
                    ),
                    None => {
                        if argv[1] == "route" {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&state.routes)
                                    .unwrap()
                            );
                        } else {
                            eprintln!("Interface '{}' not found", argv[1]);
                        }
                    }
                }
            } else {
                println!("{}", serde_json::to_string_pretty(&state).unwrap());
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}
