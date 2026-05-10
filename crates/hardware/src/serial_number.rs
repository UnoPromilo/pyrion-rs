pub fn get_serial_number_as_hex() -> [u8; 24] {
   let uid = embassy_stm32::uid::uid();
    fn hex(n: u8) -> u8 {
        b"0123456789abcdef"[n as usize]
    }

    let mut out = [0u8; 24];
    let mut i = 0;
    for &byte in &uid {
        out[i] = hex(byte >> 4);
        out[i + 1] = hex(byte & 0x0F);
        i += 2;
    }
    out
}
