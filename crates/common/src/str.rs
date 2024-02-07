pub fn remove_bom(s: &str) -> &str {
    if s.as_bytes().starts_with(&[0xEF, 0xBB, 0xBF]) {
        &s[3..]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_bom() {
        // Test case with BOM
        let s = "\u{feff}Hello, World!";
        assert_eq!(remove_bom(s), "Hello, World!");

        // Test case without BOM
        let s = "Hello, World!";
        assert_eq!(remove_bom(s), "Hello, World!");
    }
}
