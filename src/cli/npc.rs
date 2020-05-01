use nispor::get_state;
use serde_json;
use std::env::args;

fn main() {
    let argv: Vec<String> = args().collect();
    if let Ok(state) = get_state() {
        if argv.len() > 1 {
            match &state.ifaces.get(&argv[1]) {
                Some(iface) => {
                    println!("{}", serde_json::to_string_pretty(iface).unwrap())
                }
                None => eprintln!("Interface '{}' not found", argv[1]),
            }
        } else {
            println!("{}", serde_json::to_string_pretty(&state).unwrap());
        }
    }
}
