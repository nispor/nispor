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

use libc::{c_char, c_int};
use std::ffi::CString;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn nispor_net_state_retrieve(
    state: *mut *mut c_char,
    err_kind: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    assert!(!state.is_null());
    assert!(!err_kind.is_null());
    assert!(!err_msg.is_null());

    unsafe {
        *state = std::ptr::null_mut();
        *err_kind = std::ptr::null_mut();
        *err_msg = std::ptr::null_mut();
    }

    match nispor::NetState::retrieve() {
        Ok(s) => unsafe {
            *state = CString::new(serde_json::to_string(&s).unwrap())
                .unwrap()
                .into_raw();
            libc::EXIT_SUCCESS
        },
        Err(e) => unsafe {
            *err_msg = CString::new(e.msg).unwrap().into_raw();
            *err_kind =
                CString::new(format!("{}", &e.kind)).unwrap().into_raw();
            libc::EXIT_FAILURE
        },
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn nispor_net_state_free(state: *mut c_char) {
    unsafe {
        if !state.is_null() {
            CString::from_raw(state);
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn nispor_err_kind_free(err_kind: *mut c_char) {
    unsafe {
        if !err_kind.is_null() {
            CString::from_raw(err_kind);
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn nispor_err_msg_free(err_msg: *mut c_char) {
    unsafe {
        if !err_msg.is_null() {
            CString::from_raw(err_msg);
        }
    }
}
