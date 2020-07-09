use std::convert::TryFrom;

use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::client::payload::{Response, Status, Header};

#[async_trait]
pub trait Decoder {
    type Output;
    type Error;

    async fn decode<R: AsyncRead + Unpin + Send>(&mut self, source: &mut R) -> Result<Self::Output, Self::Error>;
}

pub enum ResponseDecoderError {
    Socket(super::connector::SocketError),
    BadHeader(Vec<u8>),
    BadMime(crate::mime::FromStrError),
}

pub struct ResponseDecoder {}

#[async_trait]
impl Decoder for ResponseDecoder {
    type Output = Response;
    type Error = ResponseDecoderError;

    async fn decode<R: AsyncRead + Unpin + Send>(&mut self, source: &mut R) -> Result<Self::Output, Self::Error> {
        let mut buf = vec![];
        let mut consumed = 0;
        buf.reserve(64);
        source.read_to_end(&mut buf).await?;

        let (status, status_len) = self.parse_status(&buf[consumed..])
            .ok_or_else(|| ResponseDecoderError::BadHeader(buf.clone()))?;
        consumed += status_len;

        let (meta, meta_len) = self.parse_meta(&buf[consumed..])
            .ok_or_else(|| ResponseDecoderError::BadHeader(buf.clone()))?;
        consumed += meta_len;
        buf.split_off(consumed);

        let header = Header { status, meta };
        let response = Response::new(header, buf)?;
        Ok(response)
    }
}

impl ResponseDecoder {
    fn parse_status(&self, header: &[u8]) -> Option<(Status, usize)> {
        let len = 3;
        if header.len() < len {
            return None;
        }

        let raw_status = (header[0] * 10 + header[1]) as i32;
        Status::try_from(raw_status)
            .ok()
            .map(|s| (s, len))
    }

    fn parse_meta(&self, header: &[u8]) -> Option<(String, usize)> {
        let max_len = 1024;
        let cr = 13;
        let lf = 10;
        
        let mut end = 1;
        let mut found = false;
        while end <= max_len && end < header.len() {
            // TODO: this can be get_unchecked()
            if header[end] != cr && header[end] != lf {
                end += 2;
                continue;
            }

            if header[end] == cr && end + 1 < header.len() && header[end + 1] == lf {
                end += 1;
                found = true;
                break
            } else if header[end] == lf && header[end - 1] == cr {
                found = true;
                break
            } 

            end += 2;
        }

        if end < 1 || !found {
            return None;
        }

        String::from_utf8(header[0..=end-2].into())
            .ok()
            .map(|s| (s, end))
    }
}

impl From<super::connector::SocketError> for ResponseDecoderError {
    fn from(err: super::connector::SocketError) -> Self {
        ResponseDecoderError::Socket(err)
    }
}

impl From<crate::mime::FromStrError> for ResponseDecoderError {
    fn from(err: crate::mime::FromStrError) -> Self {
        ResponseDecoderError::BadMime(err)
    }
}