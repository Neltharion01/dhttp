pub(crate) fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        const HEX: &[u8] = b"0123456789abcdef";
        out.push(HEX[b as usize >> 4] as char);
        out.push(HEX[b as usize & 0xF] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::hex;
    #[test]
    fn testhex() {
        assert_eq!("123456", &hex(&[0x12, 0x34, 0x56]));
    }
}
