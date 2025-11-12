use std::borrow::Cow;
use std::env;
use std::ffi::OsStr;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::path::MAIN_SEPARATOR;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use serde::de::Error;
use serde::{Deserialize, Serialize, Serializer};
use url::Url;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Protocol {
    File = 1,
    Ram = 2,
    Google = 3,
    Http = 4,
    Https = 5,
}

impl Protocol {
    pub fn as_str(&self) -> &str {
        match &self {
            Protocol::File => "file",
            Protocol::Ram => "ram",
            Protocol::Google => "gs",
            Protocol::Http => "http",
            Protocol::Https => "https",
        }
    }

    pub fn is_file(&self) -> bool {
        matches!(
            &self,
            Protocol::File | Protocol::Ram | Protocol::Http | Protocol::Https
        )
    }

    pub fn is_file_storage(&self) -> bool {
        matches!(&self, Protocol::File | Protocol::Ram)
    }

    pub fn is_object_storage(&self) -> bool {
        matches!(&self, Protocol::Google)
    }

    pub fn as_str_with_separator(&self) -> &str {
        match &self {
            Protocol::File => "file://",
            Protocol::Ram => "ram://",
            Protocol::Google => "gs://",
            Protocol::Http => "http://",
            Protocol::Https => "https://",
        }
    }

    pub fn separator(&self) -> char {
        match &self {
            Protocol::File => MAIN_SEPARATOR,
            Protocol::Ram => '/',
            Protocol::Google => '/',
            Protocol::Http => '/',
            Protocol::Https => '/',
        }
    }
}

impl Display for Protocol {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.as_str())
    }
}

impl FromStr for Protocol {
    type Err = crate::Error;

    fn from_str(protocol: &str) -> crate::Result<Self> {
        match protocol {
            "file" => Ok(Protocol::File),
            "ram" => Ok(Protocol::Ram),
            "gs" => Ok(Protocol::Google),
            "http" => Ok(Protocol::Http),
            "https" => Ok(Protocol::Https),
            _ => Err(crate::Error::Uri(format!("Unknown protocol: {protocol}"))),
        }
    }
}

pub const PROTOCOL_SEPARATOR: &str = "://";

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Uri {
    uri: String,
    protocol: Protocol,
}

impl Uri {
    pub fn separator(&self) -> String {
        let sep = self.protocol.separator();
        sep.to_string()
    }

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
            p.split(self.protocol.separator())
                .next()
                .unwrap_or_default()
        ))
    }

    pub fn root(&self) -> &str {
        let p = &self.uri[self.protocol.as_str().len() + PROTOCOL_SEPARATOR.len()..];
        p.split(self.protocol.separator())
            .next()
            .unwrap_or_default()
    }

    pub fn as_path(&self) -> PathBuf {
        self.path()
    }

    pub fn path(&self) -> PathBuf {
        let p = &self.uri[self.protocol.as_str().len() + PROTOCOL_SEPARATOR.len()..];
        match self.protocol {
            Protocol::Google | Protocol::Http | Protocol::Https => PathBuf::from(format!(
                "{}{}",
                self.separator().as_str(),
                p.split(self.protocol.separator())
                    .skip(1)
                    .collect::<Vec<&str>>()
                    .join(self.separator().as_str())
            )),
            Protocol::File | Protocol::Ram => {
                let sub_path = p
                    .split(self.protocol.separator())
                    .collect::<Vec<&str>>()
                    .join(self.separator().as_str())
                    .to_string();
                if self.is_dir() {
                    PathBuf::from(format!("{}{}", sub_path, self.separator().as_str()))
                } else {
                    PathBuf::from(sub_path)
                }
            }
        }
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

    pub fn is_dir(&self) -> bool {
        match self.protocol() {
            Protocol::File => self._path().is_dir(),
            Protocol::Ram => self.extension().is_none(),
            Protocol::Google => self.extension().is_none(),
            Protocol::Http => self.extension().is_none(),
            Protocol::Https => self.extension().is_none(),
        }
    }

    pub fn is_file(&self) -> bool {
        !self.is_dir()
    }

    pub fn dir(&self) -> Option<Uri> {
        if !self.is_dir() {
            return self.parent();
        }
        Some(self.clone())
    }

    pub fn join<P: AsRef<Path> + std::fmt::Debug>(&self, path: P) -> crate::Result<Self> {
        if path.as_ref().is_absolute() {
            return Err(crate::Error::Uri(format!(
                "cannot join URI `{}` with absolute path `{:?}`",
                self.uri, path
            )));
        }
        let joined = match self.protocol() {
            Protocol::File => {
                let p = Path::new(&self.uri)
                    .join(path)
                    .to_string_lossy()
                    .to_string();
                Self::parse_str(p.as_str())?.into_string()
            }
            _ => format!(
                "{}{}{}",
                self.uri,
                if self.uri.ends_with(self.protocol.separator()) {
                    "".to_string()
                } else {
                    self.protocol.separator().to_string()
                },
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
    fn parse_str(uri_str: &str) -> crate::Result<Self> {
        if uri_str.is_empty() {
            return Err(crate::Error::Uri("URI cannot be empty".to_string()));
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
                    return Err(crate::Error::Uri(
                        "failed to normalize URI: tilde expansion is only partially supported"
                            .to_string(),
                    ));
                }

                let home_dir_path = home::home_dir()
                    .ok_or(crate::Error::Uri(
                        "failed to normalize URI: could not resolve home directory".to_string(),
                    ))?
                    .to_string_lossy()
                    .to_string();

                path.replace_range(0..1, &home_dir_path);
            }
            if Path::new(&path).is_relative() {
                let current_dir =
                    env::current_dir().map_err(|e| crate::Error::Uri(format!("{e:?}")))?;
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
    type Err = crate::Error;

    fn from_str(uri_str: &str) -> crate::Result<Self> {
        Uri::parse_str(uri_str)
    }
}

impl TryFrom<Url> for Uri {
    type Error = crate::Error;

    fn try_from(url: Url) -> crate::Result<Self> {
        Self::parse_str(url.to_string().as_str())
    }
}

impl TryFrom<PathBuf> for Uri {
    type Error = crate::Error;

    fn try_from(path: PathBuf) -> crate::Result<Self> {
        let path = path.to_string_lossy().to_string();
        Uri::parse_str(&path)
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

impl From<Uri> for url::Url {
    fn from(uri: Uri) -> Self {
        url::Url::parse(uri.as_str()).unwrap()
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
            "URIUtilError: failed to normalize URI: tilde expansion is only partially supported"
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
            Uri::for_test("file:///foo/hoge/").join("../bar").unwrap(),
            "file:///foo/bar"
        );
        assert_eq!(
            Uri::for_test("file:///foo/hoge/fuga/")
                .join("..")
                .unwrap()
                .join("..")
                .unwrap(),
            "file:///foo"
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
        assert_eq!(
            Uri::for_test("file:///foo/bar/hoge.txt").parent().unwrap(),
            "file:///foo/bar"
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

    #[test]
    fn test_into_url() {
        let uri = Uri::for_test("gs://bucket/key");
        let url: url::Url = uri.into();
        assert_eq!(url.as_str(), "gs://bucket/key");
    }

    #[test]
    fn test_uri_japanese_city_path() {
        let uri = Uri::for_test("file:///plateau/08220_筑波市/bldg/53394525_bldg_6697_op.gml");
        assert_eq!(uri.protocol(), Protocol::File);
        assert!(uri.to_string().contains("筑波市"));
    }

    #[test]
    fn test_uri_japanese_codelists_path() {
        let uri = Uri::for_test("file:///codelists/Common_localPublicAuthorities.xml");
        let joined = uri.join("../Building_usage.xml").unwrap();
        assert!(joined.to_string().contains("Building_usage.xml"));
    }

    #[test]
    fn test_uri_plateau_mesh_code_pattern() {
        let uri = Uri::for_test("file:///data/53394525_bldg_6697_op.gml");
        let file_name = uri.file_name().unwrap();
        assert_eq!(file_name, Path::new("53394525_bldg_6697_op.gml"));
    }

    #[test]
    fn test_uri_with_spaces() {
        let uri = Uri::for_test("file:///path/with spaces/file.gml");
        assert!(uri.to_string().contains("with spaces"));
    }

    #[test]
    fn test_uri_nested_relative_paths() {
        let base = Uri::for_test("file:///plateau/schemas/");
        let result = base.join("../../codelists/Building.xml").unwrap();
        assert!(result.to_string().contains("codelists"));
    }

    #[test]
    fn test_uri_windows_path_conversion() {
        let uri = Uri::from_str("file:///C:/plateau/data/building.gml");
        assert!(uri.is_ok());
    }

    #[test]
    fn test_uri_http_plateau_download() {
        let uri = Uri::for_test("https://plateau.example.com/data/13101_tokyo/bldg.gml");
        assert_eq!(uri.protocol(), Protocol::Https);
        assert!(uri.to_string().contains("13101_tokyo"));
    }

    #[test]
    fn test_uri_special_characters_in_filename() {
        let uri = Uri::for_test("file:///data/building(1).gml");
        assert!(uri.to_string().contains("building(1)"));
    }

    #[test]
    fn test_uri_very_long_path() {
        let long_path = "a/".repeat(100);
        let uri_str = format!("file:///{long_path}file.gml");
        let uri = Uri::from_str(&uri_str);
        assert!(uri.is_ok());
    }

    #[test]
    fn test_uri_extension_plateau_files() {
        assert_eq!(
            Uri::for_test("file:///building.gml").extension().unwrap(),
            "gml"
        );
        assert_eq!(
            Uri::for_test("file:///codelist.xml").extension().unwrap(),
            "xml"
        );
        assert_eq!(
            Uri::for_test("file:///archive.7z").extension().unwrap(),
            "7z"
        );
    }

    #[test]
    fn test_uri_root_plateau_structure() {
        let uri = Uri::for_test("file:///08220_tsukuba-shi/udx/bldg/codelists/Building_class.xml");
        let parent = uri.parent().unwrap();
        assert!(parent.to_string().contains("codelists"));
    }

    #[test]
    fn test_uri_join_with_query_params() {
        let base = Uri::for_test("https://api.plateau.com/data");
        let result = base.join("buildings?city=tokyo");
        assert!(result.is_ok());
    }

    #[test]
    fn test_uri_multiple_dots_in_path() {
        let uri = Uri::for_test("file:///data/file.backup.gml");
        assert_eq!(uri.extension().unwrap(), "gml");
    }
}

