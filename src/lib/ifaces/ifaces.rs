use crate::ifaces::bond::get_bond_info;
use crate::ifaces::bond::get_bond_slave_info;
use crate::ifaces::bond::bond_iface_tidy_up;
use crate::Iface;
use crate::IfaceState;
use crate::IfaceType;
use crate::MasterType;
use futures::stream::TryStreamExt;
use netlink_packet_route::rtnl::link::nlas;
use netlink_packet_route::rtnl::LinkMessage;
use rtnetlink::packet::rtnl::link::nlas::Nla;
use rtnetlink::{new_connection, Error};
use std::collections::HashMap;
use std::str;
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
        iface_state.index = nl_msg.header.index;
        for nla in &nl_msg.nlas {
            //println!("{} {:?}", name, nla);
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
            if let Nla::Master(master) = nla {
                iface_state.master = Some(format!("{}", master));
            }
            if let Nla::Info(infos) = nla {
                for info in infos {
                    if let nlas::Info::Kind(t) = info {
                        iface_state.iface_type = match t {
                            nlas::InfoKind::Bond => IfaceType::Bond,
                            nlas::InfoKind::Veth => IfaceType::Veth,
                            nlas::InfoKind::Bridge => IfaceType::Bridge,
                            nlas::InfoKind::Vlan => IfaceType::Vlan,
                            _ => IfaceType::Unknown,
                        };
                    }
                    if let nlas::Info::Data(d) = info {
                        match iface_state.iface_type {
                            IfaceType::Bond => {
                                iface_state.bond_info = get_bond_info(&d)
                            }
                            _ => (),
                        }
                    }
                }
                for info in infos {
                    if let nlas::Info::SlaveKind(d) = info {
                        // Remove the tailing \0
                        match str::from_utf8(&(d.as_slice()[0..(d.len() - 1)]))
                        {
                            Ok(master_type) => {
                                iface_state.master_type =
                                    Some(master_type.into())
                            }
                            _ => (),
                        }
                    }
                }
                if let Some(master_type) = &iface_state.master_type {
                    for info in infos {
                        if let nlas::Info::SlaveData(d) = info {
                            match master_type {
                                MasterType::Bond => {
                                    iface_state.bond_slave_info =
                                        get_bond_slave_info(&d);
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }
        }
        iface_states.insert(iface_state.name.clone(), iface_state);
    }
    tidy_up(&mut iface_states);
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

fn tidy_up(iface_states: &mut HashMap<String, Iface>) {
    convert_iface_index_to_name(iface_states);
    bond_iface_tidy_up(iface_states);
}

fn convert_iface_index_to_name(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }
    for iface in iface_states.values_mut() {
        if let Some(master) = &iface.master {
            if let Some(name) = index_to_name.get(master) {
                iface.master = Some(name.to_string());
            }
        }
    }
}
