// SPDX-License-Identifier: Apache-2.0

fn main() {
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-cdylib-link-arg=-Wl,-soname=libnispor.so.1");
}
