use std::mem::transmute;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

pub(crate) const NL_ATTR_HDR_LEN: usize = 4;
pub(crate) const NLA_ALIGNTO: usize = 4;

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) struct NetLinkAttrHeader {
    pub data_len: usize,
    pub nla_len: usize,
    pub nla_type: u16,
}

pub(crate) fn parse_nla_header(data: *const u8) -> NetLinkAttrHeader {
    let mut data_len: usize =
        unsafe { u16::from_ne_bytes([*data, *(data.wrapping_offset(1))]) }
            .into();
    let nla_type: u16 = unsafe {
        transmute::<[u8; 2], u16>([
            *(data.wrapping_offset(2)),
            *(data.wrapping_offset(3)),
        ])
    };

    // Align nla_len by NLA_ALIGNTO
    let nla_len = ((data_len + NLA_ALIGNTO - 1) / NLA_ALIGNTO) * NLA_ALIGNTO;
    data_len = data_len - NL_ATTR_HDR_LEN;
    NetLinkAttrHeader {
        data_len,
        nla_len,
        nla_type,
    }
}

pub(crate) fn parse_as_u8(data: &[u8]) -> u8 {
    data[0]
}

pub(crate) fn parse_as_u16(data: &[u8]) -> u16 {
    u16::from_ne_bytes([data[0], data[1]])
}

pub(crate) fn parse_as_be16(data: &[u8]) -> u16 {
    u16::from_be_bytes([data[0], data[1]])
}

pub(crate) fn parse_as_u32(data: &[u8]) -> u32 {
    u32::from_ne_bytes([data[0], data[1], data[2], data[3]])
}

pub(crate) fn parse_as_be32(data: &[u8]) -> u32 {
    u32::from_be_bytes([data[0], data[1], data[2], data[3]])
}

pub(crate) fn parse_as_u64(data: &[u8]) -> u64 {
    u64::from_ne_bytes([
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ])
}

pub(crate) fn parse_as_ipv4(data: &[u8]) -> String {
    Ipv4Addr::from([data[0], data[1], data[2], data[3]]).to_string()
}

pub(crate) fn parse_as_ipv6(data: &[u8]) -> String {
    let mut addr_bytes = [0u8; 16];
    addr_bytes.copy_from_slice(&data[..16]);
    Ipv6Addr::from(addr_bytes).to_string()
}
