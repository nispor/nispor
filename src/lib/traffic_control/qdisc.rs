use crate::error::ErrorKind;
use crate::netlink::parse_as_i32;
use crate::netlink::parse_as_u16;
use crate::netlink::parse_as_u32;
use crate::netlink::parse_as_u8;
use crate::Iface;
use crate::NisporError;
use netlink_packet_route::rtnl::tc::nlas::Nla as TcNla;
use netlink_packet_route::rtnl::tc::nlas::Stats;
use netlink_packet_route::rtnl::tc::nlas::Stats2;
use netlink_packet_route::rtnl::TcMessage;
use netlink_packet_utils::nla::Nla;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

const TCA_INGRESS_BLOCK: u16 = 13;
const TCA_EGRESS_BLOCK: u16 = 14;

const TC_H_MAJ_MASK: u32 = 0xFFFF0000;
const TC_H_MIN_MASK: u32 = 0x0000FFFF;
const TC_H_UNSPEC: u32 = 0;
const TC_H_ROOT: u32 = 0xFFFFFFFF;

#[derive(Debug, PartialEq, Clone, Default)]
struct TcStats2 {
    app: Option<Vec<u8>>,
    requeues: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct TcSizeOptions {
    pub cell_log: u8,
    pub size_log: u8,
    pub cell_align: u16,
    pub overhead: i32,
    pub linklayer: u32,
    pub mpu: u32,
    pub mtu: u32,
    pub tsize: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct TcStats {
    // Number of enqueued bytes
    pub bytes: u64,
    // Number of enqueued packets
    pub packets: u32,
    // Packets dropped because of lack of resources
    pub drops: u32,
    // Number of throttle events when this flow goes out of allocated bandwidth
    pub overlimits: u32,
    // Current flow byte rate
    pub bps: u32,
    // Current flow packet rate
    pub pps: u32,
    pub qlen: u32,
    pub backlog: u32,
    // requeues is provided by TCA_STATS_QUEUE
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requeues: Option<u32>,
    // app is provided by TCA_STATS_APP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<Vec<u8>>,
    // TODO: Prase xstats for each kind of qdiscs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xstats: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct TcQueueingDiscipline {
    #[serde(skip_serializing)]
    pub iface_index: u32,
    pub handle: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub refcnt: u32,
    pub kind: String,
    pub chain: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingress_block: Option<u32>, // TODO: Need test
    #[serde(skip_serializing_if = "Option::is_none")]
    pub egress_block: Option<u32>, // TODO: Need test
    pub hw_offload: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_options: Option<TcSizeOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<TcStats>,
    // TODO: Each kind of qdisc has their own TCA_OPTIONS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<u8>>,
}

pub(crate) fn get_tc_qdisc(
    tc_msg: TcMessage,
) -> Result<TcQueueingDiscipline, NisporError> {
    let mut qdisc = TcQueueingDiscipline::default();
    if tc_msg.header.index > 0 {
        qdisc.iface_index = tc_msg.header.index as u32;
    } else {
        return Err(NisporError {
            kind: ErrorKind::NetlinkError,
            msg: format!(
                "Got qdisc with negative interface index number: {}",
                qdisc.iface_index
            ),
        });
    }
    qdisc.handle = parse_handle(tc_msg.header.handle);
    qdisc.refcnt = tc_msg.header.info;
    if tc_msg.header.parent != std::u32::MAX {
        qdisc.parent = Some(parse_handle(tc_msg.header.parent));
    }

    let mut stats2 = None;
    let mut xstats = None;

    for nla in tc_msg.nlas {
        if let TcNla::Kind(d) = nla {
            qdisc.kind = d;
        } else if let TcNla::HwOffload(d) = nla {
            qdisc.hw_offload = d > 0;
        } else if let TcNla::Stab(d) = nla {
            qdisc.size_options = Some(parse_stab(&d));
        } else if let TcNla::Stats(d) = nla {
            qdisc.stats = Some(parse_stats(&d));
        } else if let TcNla::Stats2(d) = nla {
            stats2 = Some(parse_stats2(d));
        } else if let TcNla::XStats(d) = nla {
            if !is_all_zero(&d) {
                xstats = Some(d);
            }
        } else if let TcNla::Options(d) = nla {
            if !is_all_zero(&d) {
                qdisc.options = Some(d);
            }
        } else if let TcNla::Other(d) = nla {
            match d.kind() {
                TCA_INGRESS_BLOCK => {
                    let mut buffer = [0u8; 4];
                    d.emit_value(&mut buffer);
                    qdisc.ingress_block = Some(parse_as_u32(&buffer));
                }
                TCA_EGRESS_BLOCK => {
                    let mut buffer = [0u8; 4];
                    d.emit_value(&mut buffer);
                    qdisc.egress_block = Some(parse_as_u32(&buffer));
                }
                _ => {
                    eprintln!("Unhandled RTM_GETQDISC NLA {:?}", d);
                }
            }
        } else {
            eprintln!("Unhandled RTM_GETQDISC NLA {:?}", nla);
        }
    }

    if let Some(stats) = &mut qdisc.stats {
        stats.xstats = xstats;
        if let Some(s2) = stats2 {
            stats.requeues = Some(s2.requeues);
            stats.app = s2.app;
        }
    }

    Ok(qdisc)
}

pub(crate) fn save_qdiscs_into_iface(
    iface_states: &mut HashMap<String, Iface>,
    qdiscs: Vec<TcQueueingDiscipline>,
) {
    let mut index_to_qdiscs: HashMap<u32, Vec<TcQueueingDiscipline>> =
        HashMap::new();

    for qdisc in qdiscs.into_iter() {
        match index_to_qdiscs.get_mut(&qdisc.iface_index) {
            Some(qdiscs) => {
                qdiscs.push(qdisc);
            }
            None => {
                index_to_qdiscs.insert(qdisc.iface_index, vec![qdisc.clone()]);
            }
        }
    }
    for iface in iface_states.values_mut() {
        iface.qdiscs = index_to_qdiscs.remove(&iface.index);
    }
}

fn parse_stab(raw: &[u8]) -> TcSizeOptions {
    let mut opt = TcSizeOptions::default();
    opt.cell_log = parse_as_u8(raw);
    opt.size_log = parse_as_u8(&raw[1..]);
    opt.cell_align = parse_as_u16(&raw[2..]);
    opt.overhead = parse_as_i32(&raw[4..]);
    opt.linklayer = parse_as_u32(&raw[8..]);
    opt.mpu = parse_as_u32(&raw[12..]);
    opt.mtu = parse_as_u32(&raw[16..]);
    opt.tsize = parse_as_u32(&raw[20..]);
    opt
}

fn parse_stats(s: &Stats) -> TcStats {
    TcStats {
        bytes: s.bytes,
        packets: s.packets,
        drops: s.drops,
        overlimits: s.overlimits,
        bps: s.bps,
        pps: s.pps,
        qlen: s.qlen,
        backlog: s.backlog,
        ..Default::default()
    }
}

fn parse_stats2(stats: Vec<Stats2>) -> TcStats2 {
    let mut s = TcStats2::default();
    for stat in stats.into_iter() {
        match stat {
            Stats2::StatsApp(d) => {
                // TODO: Each kind of qdisc has their own TCA_STATS_APP
                //       which require a lot work for parsing them
                if !is_all_zero(&d) {
                    s.app = Some(d);
                }
            }
            Stats2::StatsBasic(_) => {
                // The TCA_STATS_BASIC only provides bytes and packets
                // which is already provided by TCA_STATS
                ()
            }
            Stats2::StatsQueue(d) => {
                if !is_all_zero(&d) {
                    println!("queue {:?}", d);
                }
                s.requeues = parse_as_u32(&d[12..]);
            }
            // TODO: Handle TCA_STATS_PKT64
            // TODO: Handle TCA_STATS_RATE_EST
            _ => {
                eprintln!("Unhandled TCA_STATS2 {:?}", &stat);
            }
        }
    }
    s
}

fn is_all_zero(d: &[u8]) -> bool {
    for b in d {
        if *b != 0 {
            return false;
        }
    }
    true
}

fn parse_handle(h: u32) -> String {
    if h == TC_H_ROOT {
        "root".into()
    } else if h == TC_H_UNSPEC {
        "none".into()
    } else if (h & TC_H_MAJ_MASK) == 0 {
        format!(":{:x}", h & TC_H_MIN_MASK)
    } else if (h & TC_H_MIN_MASK) == 0 {
        format!("{:x}:", (h & TC_H_MAJ_MASK) >> 16)
    } else {
        format!("{:x}:{:x}", (h & TC_H_MAJ_MASK) >> 16, h & TC_H_MIN_MASK)
    }
}
