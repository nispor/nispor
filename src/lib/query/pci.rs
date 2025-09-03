// SPDX-License-Identifier: Apache-2.0

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::{Iface, NisporError};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub struct PciAddress {
    pub domain: u32,
    pub bus: u8,
    pub slot: u8,
    pub function: u8,
}

impl std::fmt::Display for PciAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{:02x}:{:02x}.{}",
            if self.domain > u16::MAX.into() {
                format!("{:08x}", self.domain)
            } else {
                format!("{:04x}", self.domain)
            },
            self.bus,
            self.slot,
            self.function
        )
    }
}

impl TryFrom<String> for PciAddress {
    type Error = NisporError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl From<PciAddress> for String {
    fn from(v: PciAddress) -> String {
        v.to_string()
    }
}

fn parse_pci_addr(pci_addr_str: &str) -> Option<PciAddress> {
    let mut items: Vec<&str> = pci_addr_str.split(":").collect();
    if items.len() == 2 || items.len() == 3 {
        // It is safe to unwrap as we already checked the length.
        let slot_function_str: Vec<&str> =
            items.pop().unwrap().split(".").collect();
        if slot_function_str.len() != 2 {
            return None;
        }

        let slot = u8::from_str_radix(slot_function_str[0], 16).ok()?;

        let function = slot_function_str[1].parse::<u8>().ok()?;

        let bus = u8::from_str_radix(items.pop().unwrap(), 16).ok()?;

        let domain = if let Some(item) = items.pop() {
            u32::from_str_radix(item, 16).ok()?
        } else {
            0
        };
        Some(PciAddress {
            domain,
            bus,
            slot,
            function,
        })
    } else {
        None
    }
}

impl FromStr for PciAddress {
    type Err = NisporError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_pci_addr(s).ok_or_else(|| {
            NisporError::invalid_argument(format!(
                "Invalid PCI address, expecting string like 00:1b.0 or \
                 0000:07:00.1, but got {s}"
            ))
        })
    }
}

impl Iface {
    // The PCI address only available via ethtool IOCTL `ETHTOOL_GDRVINFO`,
    // there is no netlink API for this yet. Hence using sysfs required.
    // The systemd `src/udev/udev-builtin-path_id.c` is also using this
    // information for ID_PATH.
    pub(crate) fn fill_pci_address(&mut self) {
        if let Some(dev_path) =
            PathBuf::from(format!("/sys/class/net/{}/device", self.name))
                .read_link()
                .ok()
                .as_deref()
                .and_then(Path::file_name)
                .and_then(std::ffi::OsStr::to_str)
        {
            if let Ok(pci_addr) = PciAddress::from_str(dev_path) {
                self.pci_address = Some(pci_addr);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::PciAddress;

    #[test]
    fn parse_pci_addr() {
        let expected = PciAddress {
            domain: 0,
            bus: 0x0a,
            slot: 0x9,
            function: 7,
        };
        let expected_str = "0000:0a:09.7";

        let parsed = PciAddress::from_str("0a:09.7").unwrap();
        let parsed2 = PciAddress::from_str("0000:0a:09.7").unwrap();
        assert_eq!(expected, parsed);
        assert_eq!(expected, parsed2);
        assert_eq!(expected_str, parsed.to_string());
        assert_eq!(expected_str, parsed2.to_string());
    }
}
