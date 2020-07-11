use bstr::ByteSlice;
use bytes::Bytes;

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
    pub alt_content: Bytes,
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

impl ContentBlock<'_> for TextBlock {
    fn matches(buf: &Bytes) -> bool {
        !buf.is_empty()
    }
}

impl From<&mut Bytes> for TextBlock {
    fn from(buf: &mut Bytes) -> Self {
        let line_len = match buf.lines().next() {
            None => {
                return Self {
                    content: Bytes::new(),
                }
            }
            Some(line) => line.len(),
        };

        Self {
            content: buf.split_to(line_len),
        }
    }
}

impl ContentBlock<'_> for LinkBlock {
    fn matches(buf: &Bytes) -> bool {
        buf.len() > 2 && &buf[..2] == b"=>"
    }
}

impl From<&mut Bytes> for LinkBlock {
    fn from(buf: &mut Bytes) -> Self {
        let line = match buf[..2].lines().next() {
            Some(line) => line,
            None => {
                return LinkBlock {
                    url: Bytes::new(),
                    name: None,
                }
            }
        };

        let line_len = line.len();
        let (link_start, link_end) = match super::FieldIndices::new(line).next() {
            Some(range) => range,
            None => (0, line.len()),
        };

        // drop leading line characters that are not a URL
        let _ = buf.split_to(2 + link_start);

        let url = buf.split_to(link_end);
        let name = if line_len > link_end {
            Some(buf.split_to(line_len))
        } else {
            None
        };

        Self { url, name }
    }
}
