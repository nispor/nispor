// SPDX-License-Identifier: Apache-2.0

use std::fmt::Write;

use crate::NisporError;

pub(crate) const ETH_ALEN: usize = libc::ETH_ALEN as usize;
pub(crate) const INFINIBAND_ALEN: usize = 20;

pub(crate) fn parse_as_mac(
    mac_len: usize,
    data: &[u8],
) -> Result<String, NisporError> {
    if data.len() < mac_len {
        return Err(NisporError::bug("wrong size at mac parsing".into()));
    }
    let mut rt = String::new();
    for (i, m) in data.iter().enumerate().take(mac_len) {
        write!(rt, "{m:02x}").ok();
        if i != mac_len - 1 {
            rt.push(':');
        }
    }
    Ok(rt)
}

pub(crate) fn mac_str_to_raw(mac_addr: &str) -> Result<Vec<u8>, NisporError> {
    let mac_addr = mac_addr.to_string().replace([':', '-'], "");

    let mut mac_raw: Vec<u8> = Vec::new();

    let mac_addr = mac_addr.replace(':', "");
    let mut chars = mac_addr.chars().peekable();

    while chars.peek().is_some() {
        let chunk: String = chars.by_ref().take(2).collect();
        match u8::from_str_radix(&chunk, 16) {
            Ok(i) => mac_raw.push(i),
            Err(e) => {
                return Err(NisporError::invalid_argument(format!(
                    "Invalid hex string for MAC address {mac_addr}: {e}"
                )));
            }
        }
    }

    Ok(mac_raw)
}
