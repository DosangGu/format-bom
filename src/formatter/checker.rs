pub fn is_buf_utf8(buf: &[u8]) -> bool {
    let utf8_check = String::from_utf8(buf.to_vec());
    if utf8_check.is_err() {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_ascii_and_multibyte_is_utf8() {
        assert!(is_buf_utf8("hello 안녕".as_bytes()));
    }

    #[test]
    fn invalid_bytes_are_not_utf8() {
        assert!(!is_buf_utf8(&[0xffu8, 0xfe]));
    }

    #[test]
    fn empty_buffer_is_utf8() {
        assert!(is_buf_utf8(&[]));
    }
}
