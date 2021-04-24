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

use netlink_generic::new_connection;
use tokio;

#[test]
#[ignore] // Github Action does not have ethtool-netlink enabled
fn test_genl_ctrl_resolve_ethtool() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();
    rt.block_on(genl_ctrl_resolve_ethtool());
}

async fn genl_ctrl_resolve_ethtool() {
    let (connection, mut handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    let family_id = handle.resolve_family_name("ethtool").await.unwrap();
    println!("Family ID of ethtool is {}", family_id);
    assert!(family_id > 0);
}
