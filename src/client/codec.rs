pub mod gemtext;
pub mod response;

use async_trait::async_trait;
use bstr::{ByteSlice, CharIndices};
use tokio::io::AsyncRead;

#[async_trait]
pub trait Decoder {
    type Output;
    type Error;

    async fn decode<R: AsyncRead + Unpin + Send>(
        &mut self,
        source: &mut R,
    ) -> Result<Self::Output, Self::Error>;
}

struct FieldIndices<'b> {
    chars: CharIndices<'b>,
}

impl<'b> FieldIndices<'b> {
    fn new(bytes: &'b [u8]) -> Self {
        let chars = bytes.char_indices();
        Self { chars }
    }
}

impl Iterator for FieldIndices<'_> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (start, mut end);

        loop {
            // find a first non-whitespace character
            match self.chars.next() {
                Some((s, e, c)) => {
                    if !c.is_whitespace() {
                        start = s;
                        end = e;
                        break;
                    }
                }
                None => return None,
            }
        }

        // find a last non-whitespace character
        while let Some((_, e, c)) = self.chars.next() {
            if !c.is_whitespace() {
                break;
            }

            end = e
        }

        return Some((start, end));
    }
}
