#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct DocumentVersion(u64);

impl DocumentVersion {
    pub fn new(version: u64) -> Self {
        Self(version)
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn increment(&self) -> Self {
        Self(self.0 + 1)
    }
}
