use futures::stream::TryStreamExt;
use netlink_packet_route::rtnl::link::nlas::State;
use netlink_packet_route::rtnl::LinkMessage;
use rtnetlink::{new_connection, packet::rtnl::link::nlas::Nla, Error};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::runtime::Runtime;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum IfaceState {
    Up,
    Down,
    UNKNOWN,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Iface {
    pub name: String,
    pub iface_type: String,
    pub state: IfaceState,
    pub mtu: i64,
}

fn _get_iface_name(msg: &LinkMessage) -> String {
    for nla in &msg.nlas {
        if let Nla::IfName(name) = nla {
            return name.clone();
        }
    }
    "".into()
}

async fn _get_ifaces() -> Result<HashMap<String, Iface>, Error> {
    let mut iface_states: HashMap<String, Iface> = HashMap::new();
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    let mut links = handle.link().get().execute();
    while let Some(msg) = links.try_next().await? {
        let name = _get_iface_name(&msg);
        if name.len() > 0 {
            let mut iface_state = Iface {
                name: name.clone(),
                iface_type: "unknown".into(),
                state: IfaceState::UNKNOWN,
                mtu: 0i64,
            };
            for nla in msg.nlas.into_iter() {
                println!("{:?}", nla);
                if let Nla::Mtu(mtu) = nla {
                    iface_state.mtu = mtu as i64;
                }
                if let Nla::OperState(state) = nla {
                    iface_state.state = match state {
                        State::Up => IfaceState::Up,
                        State::Down => IfaceState::Down,
                        _ => IfaceState::UNKNOWN,
                    };
                }
            }
            iface_states.insert(name, iface_state);
        }
    }

    Ok(iface_states)
}

pub fn get_ifaces() -> HashMap<String, Iface> {
    Runtime::new().unwrap().block_on(_get_ifaces()).unwrap()
}
