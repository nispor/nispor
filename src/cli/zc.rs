use serde_json;
use std::env::args;
use zatel::get_state;

fn main() {
    let argv: Vec<String> = args().collect();
    if let Ok(state) = get_state() {
        if argv.len() > 1 {
            let iface = &state.ifaces[&argv[1]];
            println!("{}", serde_json::to_string_pretty(iface).unwrap());
        } else {
            println!("{}", serde_json::to_string_pretty(&state).unwrap());
        }
    }
}
