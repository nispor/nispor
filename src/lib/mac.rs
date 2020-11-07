pub(crate) fn parse_as_mac(mac_len: usize, data: &[u8]) -> String {
    let mut rt = String::new();
    for i in 0..mac_len {
        rt.push_str(&format!("{:02x}", data[i]));
        if i != mac_len - 1 {
            rt.push_str(":");
        }
    }
    rt
}
