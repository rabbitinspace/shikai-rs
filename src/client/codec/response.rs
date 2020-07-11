use std::convert::TryFrom;

use async_trait::async_trait;
use bstr::ByteSlice;
use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::client::{
    codec::Decoder,
    connector,
    payload::{Header, Response, Status},
};

pub enum ResponseDecoderError {
    Socket(connector::SocketError),
    BadStatus(Bytes),
    BadMeta(Bytes),
    BadMime(mime::FromStrError),
}

pub struct ResponseDecoder {}

#[async_trait]
impl Decoder for ResponseDecoder {
    type Output = Response;
    type Error = ResponseDecoderError;

    async fn decode<R: AsyncRead + Unpin + Send>(
        &mut self,
        source: &mut R,
    ) -> Result<Self::Output, Self::Error> {
        let mut buf = Vec::with_capacity(64);
        source.read_to_end(&mut buf).await?;

        let mut buf = Bytes::from(buf);
        let status = self.parse_status(&mut buf)?;
        let meta = self.parse_meta(&mut buf)?;

        let header = Header { status, meta };
        let response = Response::new(header, buf)?;
        Ok(response)
    }
}

impl ResponseDecoder {
    fn parse_status(&self, buf: &mut Bytes) -> Result<Status, ResponseDecoderError> {
        if buf.len() < 3 {
            return Err(ResponseDecoderError::BadStatus(buf.slice(..)));
        }

        let raw = buf.split_to(3);
        let raw_status = (raw[0] * 10 + raw[1]) as i32;
        Status::try_from(raw_status).map_err(|_| ResponseDecoderError::BadStatus(raw))
    }

    fn parse_meta(&self, buf: &mut Bytes) -> Result<String, ResponseDecoderError> {
        let len = std::cmp::min(1026, buf.len());
        let mut start = 0;
        let mut end: Option<usize> = None;
        while start < len {
            let window = &buf[start..len];
            match window.find_byte(b'\n') {
                None => break,
                Some(maybe_end) => {
                    if maybe_end > 0 && window[maybe_end - 1] == b'\r' {
                        end = Some(maybe_end + 1);
                        break;
                    }

                    start = maybe_end + 1;
                }
            }
        }

        match end {
            None => Err(ResponseDecoderError::BadMeta(Bytes::new())),
            Some(end) => {
                let buf = buf.split_to(end);
                String::from_utf8(buf[..buf.len() - 2].to_vec())
                    .map_err(|_| ResponseDecoderError::BadMeta(buf))
            }
        }
    }
}

impl From<connector::SocketError> for ResponseDecoderError {
    fn from(err: connector::SocketError) -> Self {
        ResponseDecoderError::Socket(err)
    }
}

impl From<mime::FromStrError> for ResponseDecoderError {
    fn from(err: mime::FromStrError) -> Self {
        ResponseDecoderError::BadMime(err)
    }
}
