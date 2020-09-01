use std::process::Command;

pub fn clear_network_environment() {
    cmd_exec("../../tools/test_env", vec!["rm"]);
}

pub fn set_network_environment(env_type: &str) {
    assert!(cmd_exec("../../tools/test_env", vec![env_type]));
}

pub fn cmd_exec(command: &str, args: Vec<&str>) -> bool {
    let mut proc = Command::new(command);
    for argument in args.iter() {
        proc.arg(argument);
    }
    let status = proc.status().expect("failed to execute the command");

    return status.success();
}
