use std::borrow::Cow;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("BadSignature:")]
    BadSignature([u8; 6]),
    #[error("Unsupported Version")]
    UnsupportedVersion { major: u8, minor: u8 },
    #[error("ChecksumVerificationFailed")]
    ChecksumVerificationFailed,
    #[error("NextHeaderCrcMismatch")]
    NextHeaderCrcMismatch,
    #[error("FileIoError: {0}")]
    Io(std::io::Error, Cow<'static, str>),
    #[error("Other")]
    Other(Cow<'static, str>),
    #[error("Unknown: {0}")]
    Unknown(String),
    #[error("BadTerminatedStreamsInfo")]
    BadTerminatedStreamsInfo(u8),
    #[error("BadTerminatedUnpackInfo")]
    BadTerminatedUnpackInfo,
    #[error("BadTerminatedPackInfo")]
    BadTerminatedPackInfo(u8),
    #[error("BadTerminatedSubStreamsInfo")]
    BadTerminatedSubStreamsInfo,
    #[error("BadTerminatedheader")]
    BadTerminatedheader(u8),
    #[error("ExternalUnsupported")]
    ExternalUnsupported,
    #[error("MaxMemLimited")]
    MaxMemLimited { max_kb: usize, actual_kb: usize },
    #[error("Unsupported")]
    Unsupported(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::io(value)
    }
}

impl Error {
    #[inline]
    pub fn other<S: Into<Cow<'static, str>>>(s: S) -> Self {
        Self::Other(s.into())
    }
    #[inline]
    pub fn unsupported<S: ToString>(s: S) -> Self {
        Self::Unsupported(s.to_string())
    }

    #[inline]
    pub fn io(e: std::io::Error) -> Self {
        Self::io_msg(e, "")
    }
    #[inline]
    pub fn io_msg(e: std::io::Error, msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Io(e, msg.into())
    }

    #[inline]
    pub(crate) fn file_open(e: std::io::Error, filename: impl Into<Cow<'static, str>>) -> Self {
        Self::Io(e, filename.into())
    }
}
