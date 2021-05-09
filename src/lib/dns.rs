// Copyright 2021 Red Hat, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::ffi::CStr;
use std::net::IpAddr;

use libc;
use serde_derive::{Deserialize, Serialize};

use crate::error::NisporError;

const MAXNS: usize = 3;
const MAXDNSRCH: usize = 6;
const MAXRESOLVSORT: usize = 10;

#[repr(C)]
#[derive(Copy, Clone)]
struct ResStateSortList {
    pub addr: libc::in_addr,
    pub mask: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
union ResStateU {
    pad: [libc::c_char; 52],
    _ext: ResStateExt,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct ResStateExt {
    nscount: u16,
    nsmap: [u16; MAXNS],
    nssocks: [libc::c_int; MAXNS],
    nscount6: u16,
    nsaddrs: [*mut libc::sockaddr_in6; MAXNS],
    __glibc_reserved: [libc::c_uint; 2],
}

#[repr(C)]
#[derive(Clone)]
struct ResState {
    retrans: libc::c_int,
    retry: libc::c_int,
    options: libc::c_ulong,
    nscount: libc::c_int,
    nsaddr_list: [libc::sockaddr_in; MAXNS],
    id: libc::c_ushort,
    dnsrch: [*mut libc::c_char; MAXDNSRCH + 1],
    defdname: [libc::c_char; 256], // deprecated
    pfcode: libc::c_ulong,
    ndots_nsort_ipv6_unavail_unused: u32,
    sort_list: [ResStateSortList; MAXRESOLVSORT],
    __glibc_unused_qhook: *mut libc::c_void,
    __glibc_unused_rhook: *mut libc::c_void,
    res_h_errno: libc::c_int,
    _vcsock: libc::c_int,
    _flags: libc::c_uint,
    _u: ResStateU,
}

#[link(name = "c")]
extern "C" {
    fn __res_ninit(state: *mut ResState) -> libc::c_int;
    fn __res_nclose(state: *mut ResState);
}

impl Drop for ResState {
    fn drop(&mut self) {
        unsafe { __res_nclose(self) }
    }
}

impl ResState {
    fn new() -> Self {
        let mut state: ResState = unsafe { std::mem::zeroed() };
        unsafe {
            __res_ninit(&mut state);
        }
        state
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DnsResolver {
    name_servers: Vec<IpAddr>,
    searches: Vec<String>,
}

pub fn get_dns_resolver() -> Result<DnsResolver, NisporError> {
    let state = ResState::new();
    let mut name_servers = Vec::new();
    for index in 0..(state.nscount as usize) {
        if state.nsaddr_list[index].sin_family == libc::AF_INET as u16 {
            name_servers.push(IpAddr::V4(std::net::Ipv4Addr::from(
                u32::from_be(state.nsaddr_list[index].sin_addr.s_addr),
            )));
        } else {
            let sock_addr6 = unsafe { *state._u._ext.nsaddrs[index] };
            if sock_addr6.sin6_family == libc::AF_INET6 as u16 {
                name_servers.push(IpAddr::V6(std::net::Ipv6Addr::from(
                    sock_addr6.sin6_addr.s6_addr,
                )));
            }
        }
    }
    let mut searches = Vec::new();
    for index in 0..MAXDNSRCH + 1 {
        if !state.dnsrch[index].is_null() {
            let search_cstr = unsafe { CStr::from_ptr(state.dnsrch[index]) };
            match search_cstr.to_str() {
                Ok(s) => searches.push(s.into()),
                Err(e) => {
                    eprintln!(
                        "WARN: Got error when concerting DNS search \
                        to String: {}",
                        e
                    );
                }
            }
        }
    }

    Ok(DnsResolver {
        name_servers,
        searches,
    })
}
