use netlink_packet_utils::DecodeError;
use rtnetlink;
use serde_derive::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    InvalidArgument,
    NetlinkError,
    NisporBug,
    PermissionDeny,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NisporError {
    pub kind: ErrorKind,
    pub msg: String,
}

impl NisporError {
    pub fn bug(message: &str) -> NisporError {
        NisporError {
            kind: ErrorKind::NisporBug,
            msg: message.to_string(),
        }
    }
    pub fn permission_deny(message: &str) -> NisporError {
        NisporError {
            kind: ErrorKind::PermissionDeny,
            msg: message.to_string(),
        }
    }
    pub fn invalid_argument(message: &str) -> NisporError {
        NisporError {
            kind: ErrorKind::InvalidArgument,
            msg: message.to_string(),
        }
    }
}

impl std::fmt::Display for NisporError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for NisporError {
    /* TODO */
}

impl std::convert::From<rtnetlink::Error> for NisporError {
    fn from(e: rtnetlink::Error) -> Self {
        NisporError {
            kind: ErrorKind::NetlinkError,
            msg: e.to_string(),
        }
    }
}

impl std::convert::From<std::ffi::FromBytesWithNulError> for NisporError {
    fn from(e: std::ffi::FromBytesWithNulError) -> Self {
        NisporError {
            kind: ErrorKind::NisporBug,
            msg: format!("FromBytesWithNulError: {}", e),
        }
    }
}

impl std::convert::From<std::str::Utf8Error> for NisporError {
    fn from(e: std::str::Utf8Error) -> Self {
        NisporError {
            kind: ErrorKind::NisporBug,
            msg: format!("Utf8Error: {}", e),
        }
    }
}

impl std::convert::From<DecodeError> for NisporError {
    fn from(e: DecodeError) -> Self {
        NisporError {
            kind: ErrorKind::NetlinkError,
            msg: e.to_string(),
        }
    }
}

impl std::convert::From<std::io::Error> for NisporError {
    fn from(e: std::io::Error) -> Self {
        NisporError {
            kind: ErrorKind::NisporBug,
            msg: e.to_string(),
        }
    }
}

impl std::convert::From<std::net::AddrParseError> for NisporError {
    fn from(e: std::net::AddrParseError) -> Self {
        NisporError {
            kind: ErrorKind::InvalidArgument,
            msg: e.to_string(),
        }
    }
}
