pub mod gemtext;
pub mod response;

use async_trait::async_trait;
use bstr::{ByteSlice, CharIndices, LinesWithTerminator};
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

struct LinesWithTerminatorLen<'b> {
    lines: LinesWithTerminator<'b>,
}

trait ByteSliceExt {
    fn field_indices(&self) -> FieldIndices<'_>;
    fn lines_with_terminator_len(&self) -> LinesWithTerminatorLen<'_>;
}

impl ByteSliceExt for [u8] {
    fn field_indices(&self) -> FieldIndices<'_> {
        FieldIndices::new(self.as_bytes())
    }

    fn lines_with_terminator_len(&self) -> LinesWithTerminatorLen<'_> {
        LinesWithTerminatorLen::new(self.as_bytes())
    }
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

        Some((start, end))
    }
}

impl<'b> LinesWithTerminatorLen<'b> {
    fn new(bytes: &'b [u8]) -> Self {
        Self {
            lines: bytes.lines_with_terminator(),
        }
    }
}

impl<'b> Iterator for LinesWithTerminatorLen<'b> {
    type Item = (&'b [u8], usize);

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = self.lines.next()?;
        let mut term_len = 0;

        if line.last_byte() == Some(b'\n') {
            line = &line[..line.len() - 1];
            term_len += 1;

            if line.last_byte() == Some(b'\t') {
                line = &line[..line.len() - 1];
                term_len += 1;
            }
        }

        Some((line, term_len))
    }
}
