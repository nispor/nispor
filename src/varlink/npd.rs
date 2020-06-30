use libc::umask;
use std::process::exit;
use varlink::VarlinkService;

use crate::info_nispor::*;
use nispor::get_state;

mod info_nispor;

const IDEAL_TIMEOUT: u64 = 0;
const INITIAL_WORKER_THREADS: usize = 1;
const MAX_WORKER_THREADS: usize = 10;

fn print_usage(program: &str) {
    println!("Usage: {} <varlink_address>", program);
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() <= 1 {
        print_usage(&args[0]);
        exit(1);
    }
    run_server(&args[1]).unwrap();
    exit(0);
}

struct MyInfoGrisgeNispor {}

impl VarlinkInterface for MyInfoGrisgeNispor {
    fn get(&self, call: &mut dyn Call_Get) -> varlink::Result<()> {
        match get_state() {
            Ok(s) => call.reply(s),
            Err(e) => call.fail(&e.msg),
        }
    }
}

fn run_server(address: &str) -> varlink::Result<()> {
    let my_varlink_iface = info_nispor::new(Box::new(MyInfoGrisgeNispor {}));
    let service = VarlinkService::new(
        "info.nispor",
        "Network status query service",
        "0.1",
        "http://nispor.info",
        vec![Box::new(my_varlink_iface)],
    );
    // Make sure the socket file been created with permission 0666.
    let old_umask = unsafe { umask(0o111) };
    varlink::listen(
        service,
        &address,
        INITIAL_WORKER_THREADS,
        MAX_WORKER_THREADS,
        IDEAL_TIMEOUT,
    )?;
    unsafe {
        umask(old_umask);
    }
    Ok(())
}
