//! One D-Bus destination per independently managed user mount. Interface stays v1.
pub fn bus_name(instance: Option<&str>) -> std::io::Result<String> {
    match instance.filter(|v| !v.is_empty()) {
        None => Ok(crate::desktop::BUS_NAME.into()),
        Some(v) if v.len() == 32 && v.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) =>
            Ok(format!("{}.i{v}", crate::desktop::BUS_NAME)),
        _ => Err(std::io::Error::other("invalid mount instance")),
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn separate_instances_and_legacy_never_share_a_destination() {
        let a=bus_name(Some("0123456789abcdef0123456789abcdef")).unwrap();
        let b=bus_name(Some("1123456789abcdef0123456789abcdef")).unwrap();
        assert_ne!(a,b);assert_ne!(a,bus_name(None).unwrap());
        assert!(bus_name(Some("../bad")).is_err());assert!(bus_name(Some("Uppercase")).is_err());
    }
}
