use std::process::exit;
use varlink::VarlinkService;

use crate::info_grisge_zatel::*;
use zatel::get_state;

mod info_grisge_zatel;

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

struct MyInfoGrisgeZatel {}

impl VarlinkInterface for MyInfoGrisgeZatel {
    fn get(&self, call: &mut dyn Call_Get) -> varlink::Result<()> {
        match get_state() {
            Ok(s) => call.reply(s),
            Err(e) => call.fail(&e.msg),
        }
    }
}

fn run_server(address: &str) -> varlink::Result<()> {
    let my_varlink_iface =
        info_grisge_zatel::new(Box::new(MyInfoGrisgeZatel {}));
    let service = VarlinkService::new(
        "info.grisge.zatel",
        "Network status query service",
        "0.1",
        "http://zatel.grisge.info",
        vec![Box::new(my_varlink_iface)],
    );
    varlink::listen(
        service,
        &address,
        INITIAL_WORKER_THREADS,
        MAX_WORKER_THREADS,
        IDEAL_TIMEOUT,
    )?;
    Ok(())
}
