use bstr::ByteSlice;
use bytes::Bytes;

use super::ByteSliceExt;

pub enum Block {
    Empty,
    Text(TextBlock),
    Link(LinkBlock),
    Preformatted(PreformattedBlock),
    Header(HeaderBlock),
    List(ListBlock),
    Quote(QuoteBlock),
}

pub trait ContentBlock<'b>: From<&'b mut Bytes> {
    fn matches(buf: &Bytes) -> bool;
}

pub struct TextBlock {
    pub content: Bytes,
}

pub struct LinkBlock {
    pub url: Bytes,
    pub name: Option<Bytes>,
}

pub struct PreformattedBlock {
    pub content: Bytes,
    pub alt_content: Option<Bytes>,
}

pub struct HeaderBlock {
    pub name: Bytes,
    pub level: i32,
}

pub struct ListBlock {
    pub items: Vec<Bytes>,
}

pub struct QuoteBlock {
    pub content: Bytes,
}

// MARK: impl TextBlock

impl ContentBlock<'_> for TextBlock {
    fn matches(buf: &Bytes) -> bool {
        !buf.is_empty()
    }
}

impl From<&mut Bytes> for TextBlock {
    fn from(buf: &mut Bytes) -> Self {
        let (line_len, term_len) = match buf.lines_with_terminator_len().next() {
            None => {
                return Self {
                    content: Bytes::new(),
                }
            }
            Some(line) => (line.0.len(), line.1),
        };

        Self {
            content: buf.split_to(line_len + term_len),
        }
    }
}

// MARK: impl LinkBlock

impl ContentBlock<'_> for LinkBlock {
    fn matches(buf: &Bytes) -> bool {
        buf.len() > 2 && &buf[..2] == b"=>"
    }
}

impl From<&mut Bytes> for LinkBlock {
    fn from(buf: &mut Bytes) -> Self {
        let (line, term_len) = match buf[..2].lines_with_terminator_len().next() {
            Some(line) => line,
            None => {
                return LinkBlock {
                    url: Bytes::new(),
                    name: None,
                }
            }
        };

        let line_len = line.len();
        let (link_start, link_end) = match line.field_indices().next() {
            Some(range) => range,
            None => (0, line.len()),
        };

        // drop leading line characters that are not a URL
        let _ = buf.split_to(2 + link_start);

        let url = buf.split_to(link_end);
        let name = if line_len > link_end {
            Some(buf.split_to(line_len + term_len))
        } else {
            None
        };

        Self { url, name }
    }
}

// MAKR: impl PreformattedBlock

impl ContentBlock<'_> for PreformattedBlock {
    fn matches(buf: &Bytes) -> bool {
        buf.len() > 2 && &buf[..3] == b"```"
    }
}

impl From<&mut Bytes> for PreformattedBlock {
    fn from(buf: &mut Bytes) -> Self {
        let _ = buf.split_to(3);
        let (line_len, term_len) = match buf.lines_with_terminator_len().next() {
            None => {
                return PreformattedBlock {
                    content: Bytes::new(),
                    alt_content: None,
                }
            }
            Some(line) => (line.0.len(), line.1),
        };

        let mut alt_content = None;
        if line_len > 0 {
            let buf = buf.split_to(line_len);
            alt_content = Some(buf);
        }

        let _ = buf.split_to(term_len);
        let content = PreformattedBlock::split_at_block_end(buf);
        PreformattedBlock {
            content,
            alt_content,
        }
    }
}

impl PreformattedBlock {
    fn split_at_block_end(buf: &mut Bytes) -> Bytes {
        let mut end = 0;
        let mut window = &buf[..];
        while end < buf.len() {
            let next = match window.find_byte(b'`') {
                None => window.len(),
                Some(idx) => idx,
            };

            let last = std::cmp::min(window.len() - next, 3);
            end += next + last;

            if &window[next..last] == b"```" {
                break;
            }

            window = &window[end..];
        }

        let content = buf.split_to(end);
        let line_len = match buf.lines_with_terminator().next() {
            None => return content,
            Some(line) => line.len(),
        };

        let _ = buf.split_to(line_len);
        content
    }
}
