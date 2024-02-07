use std::borrow::Cow;
use std::env;
use std::ffi::OsStr;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use anyhow::{bail, Context};
use serde::de::Error;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Protocol {
    File = 1,
    Ram = 2,
    Google = 3,
    Http = 4,
}

impl Protocol {
    pub fn as_str(&self) -> &str {
        match &self {
            Protocol::File => "file",
            Protocol::Ram => "ram",
            Protocol::Google => "gs",
            Protocol::Http => "https",
        }
    }

    pub fn is_file(&self) -> bool {
        matches!(&self, Protocol::File | Protocol::Ram | Protocol::Http)
    }

    pub fn is_file_storage(&self) -> bool {
        matches!(&self, Protocol::File | Protocol::Ram)
    }

    pub fn is_object_storage(&self) -> bool {
        matches!(&self, Protocol::Google)
    }
}

impl Display for Protocol {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.as_str())
    }
}

impl FromStr for Protocol {
    type Err = anyhow::Error;

    fn from_str(protocol: &str) -> anyhow::Result<Self> {
        match protocol {
            "file" => Ok(Protocol::File),
            "ram" => Ok(Protocol::Ram),
            "gs" => Ok(Protocol::Google),
            "https" => Ok(Protocol::Http),
            _ => bail!("unknown URI protocol `{protocol}`"),
        }
    }
}

const PROTOCOL_SEPARATOR: &str = "://";

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Uri {
    uri: String,
    protocol: Protocol,
}

impl Uri {
    pub fn for_test(uri: &str) -> Self {
        Self::from_str(uri).unwrap()
    }

    /// Returns the extension of the URI.
    pub fn extension(&self) -> Option<&str> {
        Path::new(&self.uri).extension().and_then(OsStr::to_str)
    }

    pub fn as_str(&self) -> &str {
        &self.uri
    }

    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    /// Returns the parent URI.
    /// Does not apply to PostgreSQL URIs.
    pub fn parent(&self) -> Option<Uri> {
        let path = self._path();
        let protocol = self.protocol();

        if protocol == Protocol::Google && path.components().count() < 2 {
            return None;
        }
        let parent_path = path.parent()?;

        Some(Self {
            uri: format!("{protocol}{PROTOCOL_SEPARATOR}{}", parent_path.display()),
            protocol,
        })
    }

    fn _path(&self) -> &Path {
        Path::new(&self.uri[self.protocol.as_str().len() + PROTOCOL_SEPARATOR.len()..])
    }

    pub fn root_uri(&self) -> Self {
        let p = &self.uri[self.protocol.as_str().len() + PROTOCOL_SEPARATOR.len()..];
        Self::for_test(&format!(
            "{}{}{}",
            self.protocol.as_str(),
            PROTOCOL_SEPARATOR,
            p.split('/').next().unwrap_or_default()
        ))
    }

    pub fn root(&self) -> &str {
        let p = &self.uri[self.protocol.as_str().len() + PROTOCOL_SEPARATOR.len()..];
        p.split('/').next().unwrap_or_default()
    }

    pub fn path(&self) -> PathBuf {
        let p = &self.uri[self.protocol.as_str().len() + PROTOCOL_SEPARATOR.len()..];
        let sub_path = p
            .split('/')
            .skip(1)
            .collect::<Vec<&str>>()
            .join("/")
            .to_string();
        PathBuf::from(format!("/{}", sub_path))
    }

    pub fn file_name(&self) -> Option<&Path> {
        let path = self._path();
        if self.protocol() == Protocol::Google && path.components().count() < 2 {
            return None;
        }
        path.file_name().map(Path::new)
    }

    pub fn into_string(self) -> String {
        self.uri
    }

    pub fn join<P: AsRef<Path> + std::fmt::Debug>(&self, path: P) -> anyhow::Result<Self> {
        if path.as_ref().is_absolute() {
            bail!(
                "cannot join URI `{}` with absolute path `{:?}`",
                self.uri,
                path
            );
        }
        let joined = match self.protocol() {
            Protocol::File => Path::new(&self.uri)
                .join(path)
                .to_string_lossy()
                .to_string(),
            _ => format!(
                "{}{}{}",
                self.uri,
                if self.uri.ends_with('/') { "" } else { "/" },
                path.as_ref().display(),
            ),
        };
        Ok(Self {
            uri: joined,
            protocol: self.protocol,
        })
    }

    /// Attempts to construct a [`Uri`] from a string.
    /// A `file://` protocol is assumed if not specified.
    /// File URIs are resolved (normalized) relative to the current working directory
    /// unless an absolute path is specified.
    /// Handles special characters such as `~`, `.`, `..`.
    fn parse_str(uri_str: &str) -> anyhow::Result<Self> {
        if uri_str.is_empty() {
            bail!("failed to parse empty URI");
        }
        let (protocol, mut path) = match uri_str.split_once(PROTOCOL_SEPARATOR) {
            None => (Protocol::File, uri_str.to_string()),
            Some((protocol, path)) => (Protocol::from_str(protocol)?, path.to_string()),
        };
        if protocol == Protocol::File {
            if path.starts_with('~') {
                // We only accept `~` (alias to the home directory) and `~/path/to/something`.
                // If there is something following the `~` that is not `/`, we bail.
                if path.len() > 1 && !path.starts_with("~/") {
                    bail!("failed to normalize URI: tilde expansion is only partially supported");
                }

                let home_dir_path = home::home_dir()
                    .context("failed to normalize URI: could not resolve home directory")?
                    .to_string_lossy()
                    .to_string();

                path.replace_range(0..1, &home_dir_path);
            }
            if Path::new(&path).is_relative() {
                let current_dir = env::current_dir().context(
                    "failed to normalize URI: could not resolve current working directory. the \
                     directory does not exist or user has insufficient permissions",
                )?;
                path = current_dir.join(path).to_string_lossy().to_string();
            }
            path = normalize_path(Path::new(&path))
                .to_string_lossy()
                .to_string();
        }
        Ok(Self {
            uri: format!("{protocol}{PROTOCOL_SEPARATOR}{path}"),
            protocol,
        })
    }
}

impl AsRef<str> for Uri {
    fn as_ref(&self) -> &str {
        &self.uri
    }
}

impl Debug for Uri {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter
            .debug_struct("Uri")
            .field("uri", &self.as_str())
            .finish()
    }
}

impl Display for Uri {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.as_str())
    }
}

impl FromStr for Uri {
    type Err = anyhow::Error;

    fn from_str(uri_str: &str) -> anyhow::Result<Self> {
        Uri::parse_str(uri_str)
    }
}

impl PartialEq<&str> for Uri {
    fn eq(&self, other: &&str) -> bool {
        &self.uri == other
    }
}

impl PartialEq<String> for Uri {
    fn eq(&self, other: &String) -> bool {
        &self.uri == other
    }
}

impl<'de> Deserialize<'de> for Uri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let uri_str: Cow<'de, str> = Deserialize::deserialize(deserializer)?;
        let uri = Uri::from_str(&uri_str).map_err(D::Error::custom)?;
        Ok(uri)
    }
}

impl Serialize for Uri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.uri)
    }
}

/// Normalizes a path by resolving the components like (., ..).
/// This helper does the same thing as `Path::canonicalize`.
/// It only differs from `Path::canonicalize` by not checking file existence
/// during resolution.
/// <https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs#L61>
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut resulting_path_buf =
        if let Some(component @ Component::Prefix(_)) = components.peek().cloned() {
            components.next();
            PathBuf::from(component.as_os_str())
        } else {
            PathBuf::new()
        };

    for component in components {
        match component {
            Component::Prefix(_) => unreachable!(),
            Component::RootDir => {
                resulting_path_buf.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                resulting_path_buf.pop();
            }
            Component::Normal(inner_component) => {
                resulting_path_buf.push(inner_component);
            }
        }
    }
    resulting_path_buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_new_uri() {
        Uri::from_str("").unwrap_err();

        let home_dir = home::home_dir().unwrap();
        let current_dir = env::current_dir().unwrap();

        let uri = Uri::from_str("file:///home/foo/bar").unwrap();
        assert_eq!(uri.protocol(), Protocol::File);
        assert_eq!(uri, "file:///home/foo/bar");
        assert_eq!(uri, "file:///home/foo/bar".to_string());
        assert_eq!(
            Uri::from_str("file:///foo./bar..").unwrap(),
            "file:///foo./bar.."
        );
        assert_eq!(
            Uri::from_str("home/homer/docs/dognuts").unwrap(),
            format!("file://{}/home/homer/docs/dognuts", current_dir.display())
        );
        assert_eq!(
            Uri::from_str("home/homer/docs/../dognuts").unwrap(),
            format!("file://{}/home/homer/dognuts", current_dir.display())
        );
        assert_eq!(
            Uri::from_str("home/homer/docs/../../dognuts").unwrap(),
            format!("file://{}/home/dognuts", current_dir.display())
        );
        assert_eq!(
            Uri::from_str("/home/homer/docs/dognuts").unwrap(),
            "file:///home/homer/docs/dognuts"
        );
        assert_eq!(
            Uri::from_str("~").unwrap(),
            format!("file://{}", home_dir.display())
        );
        assert_eq!(
            Uri::from_str("~/").unwrap(),
            format!("file://{}", home_dir.display())
        );
        assert_eq!(
            Uri::from_str("~anything/bar").unwrap_err().to_string(),
            "failed to normalize URI: tilde expansion is only partially supported"
        );
        assert_eq!(
            Uri::from_str("~/.").unwrap(),
            format!("file://{}", home_dir.display())
        );
        assert_eq!(
            Uri::from_str("~/..").unwrap(),
            format!("file://{}", home_dir.parent().unwrap().display())
        );
        assert_eq!(
            Uri::from_str("file://").unwrap(),
            format!("file://{}", current_dir.display())
        );
        assert_eq!(Uri::from_str("file:///").unwrap(), "file:///");
        assert_eq!(
            Uri::from_str("file://.").unwrap(),
            format!("file://{}", current_dir.display())
        );
        assert_eq!(
            Uri::from_str("file://..").unwrap(),
            format!("file://{}", current_dir.parent().unwrap().display())
        );
        assert_eq!(
            Uri::from_str("gs://bucket/docs/dognuts").unwrap(),
            "gs://bucket/docs/dognuts"
        );
        assert_eq!(
            Uri::from_str("gs://bucket/homer/docs/../dognuts").unwrap(),
            "gs://bucket/homer/docs/../dognuts"
        );
    }

    #[test]
    fn test_uri_protocol() {
        assert_eq!(Uri::for_test("file:///home").protocol(), Protocol::File);
        assert_eq!(Uri::for_test("ram:///in-memory").protocol(), Protocol::Ram);
        assert_eq!(
            Uri::for_test("gs://bucket/key").protocol(),
            Protocol::Google
        );
    }

    #[test]
    fn test_uri_extension() {
        assert!(Uri::for_test("gs://").extension().is_none());

        assert_eq!(
            Uri::for_test("gs://config.json").extension().unwrap(),
            "json"
        );
    }

    #[test]
    fn test_uri_join() {
        assert_eq!(
            Uri::for_test("file:///").join("foo").unwrap(),
            "file:///foo"
        );
        assert_eq!(
            Uri::for_test("file:///foo").join("bar").unwrap(),
            "file:///foo/bar"
        );
        assert_eq!(
            Uri::for_test("file:///foo/").join("bar").unwrap(),
            "file:///foo/bar"
        );
        assert_eq!(
            Uri::for_test("ram://foo").join("bar").unwrap(),
            "ram://foo/bar"
        );
        assert_eq!(
            Uri::for_test("gs://bucket").join("key").unwrap(),
            "gs://bucket/key"
        );
    }

    #[test]
    fn test_uri_parent() {
        assert!(Uri::for_test("file:///").parent().is_none());
        assert_eq!(Uri::for_test("file:///foo").parent().unwrap(), "file:///");
        assert_eq!(Uri::for_test("file:///foo/").parent().unwrap(), "file:///");
        assert_eq!(
            Uri::for_test("file:///foo/bar").parent().unwrap(),
            "file:///foo"
        );
        assert!(Uri::for_test("ram:///").parent().is_none());
        assert_eq!(Uri::for_test("ram:///foo").parent().unwrap(), "ram:///");
        assert_eq!(Uri::for_test("ram:///foo/").parent().unwrap(), "ram:///");
        assert_eq!(
            Uri::for_test("ram:///foo/bar").parent().unwrap(),
            "ram:///foo"
        );
        assert!(Uri::for_test("gs://bucket").parent().is_none());
        assert!(Uri::for_test("gs://bucket/").parent().is_none());
        assert_eq!(
            Uri::for_test("gs://bucket/foo").parent().unwrap(),
            "gs://bucket"
        );
        assert_eq!(
            Uri::for_test("gs://bucket/foo/").parent().unwrap(),
            "gs://bucket"
        );
        assert_eq!(
            Uri::for_test("gs://bucket/foo/bar").parent().unwrap(),
            "gs://bucket/foo"
        );
        assert_eq!(
            Uri::for_test("gs://bucket/foo/bar/").parent().unwrap(),
            "gs://bucket/foo"
        );
    }

    #[test]
    fn test_uri_file_name() {
        assert!(Uri::for_test("file:///").file_name().is_none());
        assert_eq!(
            Uri::for_test("file:///foo").file_name().unwrap(),
            Path::new("foo")
        );
        assert_eq!(
            Uri::for_test("file:///foo/").file_name().unwrap(),
            Path::new("foo")
        );
        assert!(Uri::for_test("ram:///").file_name().is_none());
        assert_eq!(
            Uri::for_test("ram:///foo").file_name().unwrap(),
            Path::new("foo")
        );
        assert_eq!(
            Uri::for_test("ram:///foo/").file_name().unwrap(),
            Path::new("foo")
        );
        assert!(Uri::for_test("gs://bucket").file_name().is_none());
        assert!(Uri::for_test("gs://bucket/").file_name().is_none());
        assert_eq!(
            Uri::for_test("gs://bucket/foo").file_name().unwrap(),
            Path::new("foo"),
        );
        assert_eq!(
            Uri::for_test("gs://bucket/foo/").file_name().unwrap(),
            Path::new("foo"),
        );
    }

    #[test]
    fn test_root() {
        assert_eq!(Uri::for_test("file:///foo").root(), "");
        assert_eq!(Uri::for_test("ram:///").root(), "");
        assert_eq!(Uri::for_test("gs://bucket/").root(), "bucket");
    }

    #[test]
    fn test_sub_path() {
        assert_eq!(
            Uri::for_test("file:///foo/hoge").path(),
            PathBuf::from_str("/foo/hoge").unwrap()
        );
        assert_eq!(
            Uri::for_test("ram:///").path(),
            PathBuf::from_str("/").unwrap()
        );
        assert_eq!(
            Uri::for_test("gs://bucket/hogefuga/piyo").path(),
            PathBuf::from_str("/hogefuga/piyo").unwrap(),
        );
    }

    #[test]
    fn test_uri_serialize() {
        let uri = Uri::for_test("gs://bucket/key");
        assert_eq!(
            serde_json::to_value(uri).unwrap(),
            serde_json::Value::String("gs://bucket/key".to_string())
        );
    }
}
