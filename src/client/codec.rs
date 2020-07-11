pub mod response;
pub mod gemtext;

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
