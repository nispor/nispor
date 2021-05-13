use futures::stream::TryStreamExt;
use netlink_ethtool;
use netlink_generic;
use tokio;

// Once we find a way to load netsimdev kernel module in CI, we can convert this
// to a test
fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();
    let family_id = rt.block_on(genl_ctrl_resolve_ethtool());
    rt.block_on(get_ring(family_id, None));
}

async fn genl_ctrl_resolve_ethtool() -> u16 {
    let (connection, mut handle, _) =
        netlink_generic::new_connection().unwrap();
    tokio::spawn(connection);

    let family_id = handle.resolve_family_name("ethtool").await.unwrap();
    println!("Family ID of ethtool is {}", family_id);
    family_id
}

async fn get_ring(family_id: u16, iface_name: Option<&str>) {
    let (connection, mut handle, _) =
        netlink_ethtool::new_connection(family_id).unwrap();
    tokio::spawn(connection);

    let mut ring_handle = handle.ring().get(iface_name).execute();

    let mut msgs = Vec::new();
    while let Some(msg) = ring_handle.try_next().await.unwrap() {
        msgs.push(msg);
    }
    assert!(msgs.len() > 0);
    for msg in msgs {
        println!("{:?}", msg);
    }
}
