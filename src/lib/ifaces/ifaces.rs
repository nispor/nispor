use crate::ifaces::bond::get_bond_conf;
use crate::Iface;
use crate::IfaceState;
use crate::IfaceType;
use futures::stream::TryStreamExt;
use netlink_packet_route::rtnl::link::nlas;
use netlink_packet_route::rtnl::LinkMessage;
use rtnetlink::packet::rtnl::link::nlas::Nla;
use rtnetlink::{new_connection, Error};
use std::collections::HashMap;
use tokio::runtime::Runtime;

async fn _get_ifaces() -> Result<HashMap<String, Iface>, Error> {
    let mut iface_states: HashMap<String, Iface> = HashMap::new();
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    let mut links = handle.link().get().execute();
    while let Some(nl_msg) = links.try_next().await? {
        let name = _get_iface_name(&nl_msg);
        if name.len() <= 0 {
            continue;
        }
        let mut iface_state = Iface {
            name: name.clone(),
            ..Default::default()
        };
        for nla in &nl_msg.nlas {
            // println!("{} {:?}", name, nla);
            if let Nla::Mtu(mtu) = nla {
                iface_state.mtu = *mtu as i64;
            }
            if let Nla::Address(mac) = nla {
                let mut mac_str = String::new();
                for octet in mac.iter() {
                    mac_str.push_str(&format!("{:02X?}:", octet));
                }
                mac_str.pop();
                iface_state.mac_address = mac_str;
            }
            if let Nla::OperState(state) = nla {
                iface_state.state = match state {
                    nlas::State::Up => IfaceState::Up,
                    nlas::State::Down => IfaceState::Down,
                    _ => IfaceState::Unknown,
                };
            }
            if let Nla::Info(infos) = nla {
                for info in infos {
                    if let nlas::Info::Kind(t) = info {
                        iface_state.iface_type = match t {
                            nlas::InfoKind::Bond => IfaceType::Bond,
                            _ => IfaceType::Unknown,
                        };
                    }
                }
                for info in infos {
                    if let nlas::Info::Data(d) = info {
                        match iface_state.iface_type {
                            IfaceType::Bond => {
                                iface_state.bond_conf = get_bond_conf(&d)
                            }
                            _ => (),
                        }
                    }
                }
            }
        }
        iface_states.insert(iface_state.name.clone(), iface_state);
    }
    Ok(iface_states)
}

pub(crate) fn get_ifaces() -> HashMap<String, Iface> {
    Runtime::new().unwrap().block_on(_get_ifaces()).unwrap()
}

fn _get_iface_name(nl_msg: &LinkMessage) -> String {
    for nla in &nl_msg.nlas {
        if let Nla::IfName(name) = nla {
            return name.clone();
        }
    }
    "".into()
}
