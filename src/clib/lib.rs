use libc::{c_char, c_int};
use nispor;
use serde_json;
use std::ffi::CString;

#[no_mangle]
pub extern "C" fn nispor_state_get(
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

    match nispor::get_state() {
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

#[no_mangle]
pub extern "C" fn nispor_state_free(state: *mut c_char) {
    unsafe {
        if state != std::ptr::null_mut() {
            CString::from_raw(state);
        }
    }
}

#[no_mangle]
pub extern "C" fn nispor_err_kind_free(err_kind: *mut c_char) {
    unsafe {
        if err_kind != std::ptr::null_mut() {
            CString::from_raw(err_kind);
        }
    }
}

#[no_mangle]
pub extern "C" fn nispor_err_msg_free(err_msg: *mut c_char) {
    unsafe {
        if err_msg != std::ptr::null_mut() {
            CString::from_raw(err_msg);
        }
    }
}
