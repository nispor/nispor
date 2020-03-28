use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum State {
    UP,
    DOWN,
    UNKNOWN,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct IfaceState {
    pub name: String,
    pub iface_type: String,
    pub state: State,
    pub mtu: i64,
}

pub fn get_ifaces() -> HashMap<String, IfaceState> {
    let mut iface_states: HashMap<String, IfaceState> = HashMap::new();
    let iface_state = IfaceState {
        name: "foo".to_string(),
        iface_type: "unknown".to_string(),
        state: State::UP,
        mtu: 0i64,
    };
    iface_states.insert("test".to_string(), iface_state);
    iface_states
}
