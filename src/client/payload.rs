use std::convert::TryFrom;
use std::str::FromStr;

use bytes::Bytes;

use crate::mime::Mime;
use crate::url::Url;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
    pub mime: Option<Mime>,
    pub body: Bytes,
    _priv: (),
}

impl Status {
    pub fn status_type(&self) -> StatusType {
        let value = (*self as i32) / 10;
        StatusType::try_from(value)
            .expect("unexpected status code type")
    }
}

impl TryFrom<i32> for Status {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        use Status::*;

        // TODO: write macro
        let cases = [
            Input,
            SensitiveInput,
            Success,
            RedirectTemporary,
            RedirectPermanent,
            TemporaryFailure,
            ServerUnavailable,
            CgiError,
            ProxyError,
            SlowDown,
            PermanentFailure,
            NotFound,
            Gone,
            ProxyRequestRefused,
            BadRequest,
            ClientCertificateRequired,
            CertificateNotAuthorised,
            CertificateNotValid,
        ];

        for case in cases.iter() {
            if (*case as i32) == value {
                return Ok(*case);
            }
        }
        
        Err(())
    }
}

impl TryFrom<i32> for StatusType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        use StatusType::*;

        // TODO: write macro
        let cases = [
            Input,
            Success,
            Redirect,
            TemporaryFailure,
            PermanentFailure,
            CertificateRequired,
        ];

        for case in cases.iter() {
            if (*case as i32) == value {
                return Ok(*case);
            }
        }

        Err(())
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

impl Response {
    pub fn new(header: Header, body: Bytes) -> Result<Self, mime::FromStrError> {
        let mut mime = None;
        if header.status.status_type() == StatusType::Success {
            mime = Some(Mime::from_str(&header.meta)?);
        }

        Ok(Response { header, mime, body, _priv: () })
    }
}