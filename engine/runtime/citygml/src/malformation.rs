//! Present-but-malformed input, collected (not just warned about) so a strict
//! caller can fail the read naming the offending location, per Action Standard
//! §4.3. See `pipeline::build_features_reporting`.

use std::fmt;

/// A place where present input was malformed. Collected rather than only warned
/// about, so a strict caller can fail the read naming the offending location,
/// per Action Standard §4.3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Malformation {
    /// The source file it was found in, when known.
    pub file: String,
    /// The `gml:id` or element name locating it, when known. Empty when neither
    /// is available at the site.
    pub location: String,
    /// What was wrong, in the same words as the site's existing warning.
    pub reason: String,
}

impl fmt::Display for Malformation {
    /// Formats as much of `file`/`location`/`reason` as is known. Either or both
    /// of `file`/`location` may be empty (some sites have no way to recover
    /// them), so this never prints a bare empty placeholder for them.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.file.is_empty(), self.location.is_empty()) {
            (false, false) => write!(f, "{} at {}: {}", self.file, self.location, self.reason),
            (false, true) => write!(f, "{}: {}", self.file, self.reason),
            (true, false) => write!(f, "at {}: {}", self.location, self.reason),
            (true, true) => write!(f, "{}", self.reason),
        }
    }
}
