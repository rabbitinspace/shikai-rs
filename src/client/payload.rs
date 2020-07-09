use crate::url::Url;
use crate::mime::Mime;

#[derive(Debug, Clone, Copy)]
pub enum Status {
    Input = 10,
    SensitiveInput = 11,

    Success = 20,

    RedirectTemporary = 30,
    RedirectPermanent = 31,

    TemporaryFailure = 40,
    ServerUnavailable = 41,
    CgiError = 42,
    ProxyError = 43,
    SlowDown = 44,

    PermanentFailure = 50,
    NotFound = 51,
    Gone = 52,
    ProxyRequestRefused = 53,
    BadRequest = 54,

    ClientCertificateRequired = 60,
    CertificateNotAuthorised = 61,
    CertificateNotValid = 62,
}

#[derive(Debug, Clone, Copy)]
pub enum StatusType {
    Input = 1,
    Success,
    Redirect,
    TemporaryFailure,
    PermanentFailure,
    CertificateRequired,
}

pub struct Header {
    pub status: Status,
    pub meta: String,
}

pub struct Request {
    pub url: Url,
    _priv: (),
}

pub struct Response {
    pub header: Header,
    pub mime: Mime,
    pub body: Vec<u8>,
}

impl Status {
    pub fn status_type(&self) -> StatusType {
        use StatusType::*;

        // TODO: macro
        let value = (*self as i32) / 10;
        match value {
            value if value == Input as i32 => Input,
            value if value == Success as i32 => Success,
            value if value == Redirect as i32 => Redirect,
            value if value == TemporaryFailure as i32 => TemporaryFailure,
            value if value == PermanentFailure as i32 => PermanentFailure,
            value if value == CertificateRequired as i32 => CertificateRequired,
            _ => panic!("unexpected status code type"),
        }
    }
}

impl Request {
    pub fn new(mut url: Url) -> Self {
        assert_eq!(url.scheme(), "gemini", "only gemini scheme is suported");
        assert!(url.has_host(), "url must have a host");
        assert!(!url.host_str().unwrap().is_empty(), "url must have a host");

        if matches!(url.port(), None) {
            url.set_port(Some(1965))
                .expect("failed to set default port");
        }

        Self { url, _priv: () }
    }
}