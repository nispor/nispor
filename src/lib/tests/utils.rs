use std::process::Command;

pub fn delete_dummy(iface_name: &str) {
    assert!(cmd_exec("ip", vec!["link", "delete", iface_name]));
}

pub fn create_dummy(iface_name: &str) {
    assert!(cmd_exec("ip", vec!["link", "add", iface_name, "type", "dummy"]));
}

pub fn cmd_exec(command: &str, args: Vec<&str>) -> bool {
    let mut proc = Command::new(command);
    for argument in args.iter() {
        proc.arg(argument);
    }
    let status = proc.status().expect("failed to execute the command");

    return status.success();
}
