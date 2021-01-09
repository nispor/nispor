use crate::NisporError;

pub(crate) fn parse_as_mac(
    mac_len: usize,
    data: &[u8],
) -> Result<String, NisporError> {
    let mut rt = String::new();
    for i in 0..mac_len {
        rt.push_str(&format!(
            "{:02x}",
            *data
                .get(i)
                .ok_or(NisporError::bug("wrong index at mac parsing".into()))?
        ));
        if i != mac_len - 1 {
            rt.push_str(":");
        }
    }
    Ok(rt)
}
