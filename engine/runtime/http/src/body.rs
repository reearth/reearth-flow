use std::io::Read;

/// Read `reader` to the end, stopping as soon as more than `max_size` bytes
/// arrive instead of buffering the whole input. Returns `None` when the limit
/// is exceeded.
pub fn read_to_end_capped(reader: impl Read, max_size: u64) -> std::io::Result<Option<Vec<u8>>> {
    let mut body = Vec::new();
    reader
        .take(max_size.saturating_add(1))
        .read_to_end(&mut body)?;
    Ok((body.len() as u64 <= max_size).then_some(body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_within_cap() {
        let data = vec![b'a'; 400];
        let body = read_to_end_capped(data.as_slice(), 400).unwrap();
        assert_eq!(body, Some(data));
    }

    #[test]
    fn test_read_stops_at_cap_on_endless_input() {
        let body = read_to_end_capped(std::io::repeat(b'a'), 500).unwrap();
        assert_eq!(body, None);
    }

    #[test]
    fn test_read_max_cap_does_not_overflow() {
        let data = vec![b'a'; 400];
        let body = read_to_end_capped(data.as_slice(), u64::MAX).unwrap();
        assert_eq!(body.map(|b| b.len()), Some(400));
    }
}
