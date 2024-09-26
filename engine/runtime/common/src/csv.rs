use strum_macros::Display;

#[derive(Debug, Clone, Display, PartialEq, Eq)]
pub enum Delimiter {
    Comma,
    Tab,
}

impl From<u8> for Delimiter {
    fn from(value: u8) -> Self {
        match value {
            b',' => Self::Comma,
            b'\t' => Self::Tab,
            _ => unreachable!(),
        }
    }
}

impl From<Delimiter> for u8 {
    fn from(value: Delimiter) -> Self {
        match value {
            Delimiter::Comma => b',',
            Delimiter::Tab => b'\t',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delimiter_from_u8() {
        assert_eq!(Delimiter::from(b','), Delimiter::Comma);
        assert_eq!(Delimiter::from(b'\t'), Delimiter::Tab);
    }

    #[test]
    #[should_panic(expected = "unreachable")]
    fn test_delimiter_from_u8_unreachable() {
        let _ = Delimiter::from(b'#');
    }

    #[test]
    fn test_delimiter_into_u8() {
        assert_eq!(u8::from(Delimiter::Comma), b',');
        assert_eq!(u8::from(Delimiter::Tab), b'\t');
    }
}
