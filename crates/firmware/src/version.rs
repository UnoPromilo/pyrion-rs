use logging::info;

pub fn populate_version() {
    let version = parse_version(env!("CARGO_PKG_VERSION"));
    info!("Version: {}.{}.{}", version[0], version[1], version[2]);
    let controller_state = controller_shared::state::state();
    controller_state
        .version
        .major
        .store(version[0], core::sync::atomic::Ordering::Relaxed);
    controller_state
        .version
        .minor
        .store(version[1], core::sync::atomic::Ordering::Relaxed);
    controller_state
        .version
        .patch
        .store(version[2], core::sync::atomic::Ordering::Relaxed);
}

const fn parse_version(version: &str) -> [u8; 3] {
    let bytes = version.as_bytes();
    let mut major = 0u8;
    let mut minor = 0u8;
    let mut patch = 0u8;

    let mut i = 0;
    let mut part = 0u8;

    while i < bytes.len() {
        let byte = bytes[i];
        if byte >= b'0' && byte <= b'9' {
            let digit = byte - b'0';
            match part {
                0 => major = major * 10 + digit,
                1 => minor = minor * 10 + digit,
                2 => patch = patch * 10 + digit,
                _ => {}
            }
        } else if byte == b'.' {
            part += 1;
        }
        i += 1;
    }

    [major, minor, patch]
}
