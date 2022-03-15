use netlink_packet_route::rtnl::link::nlas::InfoData;
use netlink_packet_route::rtnl::link::nlas::InfoIpoib;
use serde::{Deserialize, Serialize};

const IPOIB_MODE_DATAGRAM: u16 = 0;
const IPOIB_MODE_CONNECTED: u16 = 1;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum IpoibMode {
    /* using unreliable datagram QPs */
    Datagram,
    /* using connected QPs */
    Connected,
    Other(u16),
    Unknown,
}

impl Default for IpoibMode {
    fn default() -> Self {
        IpoibMode::Unknown
    }
}

impl From<u16> for IpoibMode {
    fn from(d: u16) -> Self {
        match d {
            IPOIB_MODE_DATAGRAM => Self::Datagram,
            IPOIB_MODE_CONNECTED => Self::Connected,
            _ => Self::Other(d),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
#[non_exhaustive]
pub struct IpoibInfo {
    pub pkey: u16,
    pub mode: IpoibMode,
    pub umcast: bool,
}

pub(crate) fn get_ipoib_info(data: &InfoData) -> Option<IpoibInfo> {
    if let InfoData::Ipoib(infos) = data {
        let mut ipoib_info = IpoibInfo::default();
        for info in infos {
            if let InfoIpoib::Pkey(d) = *info {
                ipoib_info.pkey = d;
            } else if let InfoIpoib::Mode(d) = *info {
                ipoib_info.mode = d.into();
            } else if let InfoIpoib::UmCast(d) = *info {
                ipoib_info.umcast = d > 0;
            } else {
                log::warn!("Unknown IPoIB info {:?}", info)
            }
        }
        Some(ipoib_info)
    } else {
        None
    }
}
