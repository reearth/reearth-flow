use std::fmt::{Binary, Display, Formatter, Result};
use std::ops::{BitAnd, BitOr};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ProcessingOptions(u8);

#[derive(Clone, Debug)]
#[repr(u8)]
enum ProcessingOptionFlags {
    AssumeIDs = 0b0000_0001,
    ParseEntities = 0b0000_0010,
    AddNamespaces = 0b0000_0100,
}

impl Display for ProcessingOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "ProcessingOptions {{")?;

        let mut option_strings: Vec<&str> = Vec::new();
        if self.has_assume_ids() {
            option_strings.push("AssumeIDs");
        }
        if self.has_parse_entities() {
            option_strings.push("ParseEntities");
        }
        if self.has_add_namespaces() {
            option_strings.push("AddNamespaces");
        }
        write!(f, "{}", option_strings.join(", "))?;

        write!(f, "}}")
    }
}

impl Binary for ProcessingOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if f.alternate() {
            write!(f, "{:#010b}", self.0)
        } else {
            write!(f, "{:08b}", self.0)
        }
    }
}

impl BitAnd for ProcessingOptions {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for ProcessingOptions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl ProcessingOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn has_none(&self) -> bool {
        self.0 == 0
    }

    pub fn has_assume_ids(&self) -> bool {
        self.0 & (ProcessingOptionFlags::AssumeIDs as u8) != 0
    }

    pub fn has_parse_entities(&self) -> bool {
        self.0 & (ProcessingOptionFlags::ParseEntities as u8) != 0
    }

    pub fn has_add_namespaces(&self) -> bool {
        self.0 & (ProcessingOptionFlags::AddNamespaces as u8) != 0
    }

    pub fn set_assume_ids(&mut self) {
        self.0 |= ProcessingOptionFlags::AssumeIDs as u8
    }

    pub fn set_parse_entities(&mut self) {
        self.0 |= ProcessingOptionFlags::ParseEntities as u8
    }

    pub fn set_add_namespaces(&mut self) {
        self.0 |= ProcessingOptionFlags::AddNamespaces as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none() {
        let options = ProcessingOptions::default();

        assert!(options.has_none());
        assert!(!options.has_assume_ids());
        assert!(!options.has_parse_entities());
        assert!(!options.has_add_namespaces());

        assert_eq!(format!("{}", options), r"ProcessingOptions {}".to_string());
        assert_eq!(format!("{:b}", options), r"00000000".to_string());
        assert_eq!(format!("{:#b}", options), r"0b00000000".to_string());

        let new_options = ProcessingOptions::new();
        assert_eq!(options, new_options);
    }
}
