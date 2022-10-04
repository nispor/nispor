// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

use pretty_assertions::assert_eq;

pub(crate) fn clear_network_environment() {
    cmd_exec("../../tools/test_env", vec!["rm"]);
}

pub(crate) fn set_network_environment(env_type: &str) {
    assert!(cmd_exec("../../tools/test_env", vec![env_type]));
}

pub(crate) fn cmd_exec(command: &str, args: Vec<&str>) -> bool {
    let mut proc = Command::new(command);
    for argument in args.iter() {
        proc.arg(argument);
    }
    let status = proc.status().expect("failed to execute the command");

    status.success()
}

pub(crate) fn assert_value_match<T>(expected_state: &str, current_state: &T)
where
    T: serde::Serialize,
{
    let current = serde_json::to_value(current_state).unwrap();
    let expected: serde_json::Value =
        serde_yaml::from_str(expected_state).unwrap();
    _assert_value_match(&expected, &current);
}

fn _assert_value_match(
    expected: &serde_json::Value,
    current: &serde_json::Value,
) {
    println!("Asserting expected: {:?}", expected);
    println!("Asserting current:  {:?}", current);
    match expected {
        serde_json::Value::Object(expected_map) => {
            for (k, v) in expected_map.iter() {
                println!("Asserting key '{}'", k);
                _assert_value_match(v, current.get(k).unwrap());
            }
        }
        serde_json::Value::Array(expected_array) => {
            let current_array = current.as_array().unwrap();
            assert_eq!(expected_array.len(), current_array.len());

            for (i, v) in expected_array.iter().enumerate() {
                _assert_value_match(v, &current_array[i]);
            }
        }
        _ => {
            assert_eq!(expected, current)
        }
    }
}
