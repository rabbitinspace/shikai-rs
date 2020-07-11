use bytes::{Bytes, BytesMut};

pub enum Block {
    Empty,
    Text(TextBlock),
    Link(LinkBlock),
    Preformatted(PreformattedBlock),
    Header(HeaderBlock),
    List(ListBlock),
    Quote(QuoteBlock),
}

pub trait ContentBlock<'b>: From<&'b mut BytesMut> {
    fn matches(content: &[u8]) -> bool;
}

pub struct TextBlock {
    pub content: Bytes,
}

pub struct LinkBlock {
    pub url: Bytes,
    pub name: Bytes,
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
    fn matches(content: &[u8]) -> bool {
        !content.is_empty()
    }
}

impl From<&mut BytesMut> for TextBlock {
    fn from(content: &mut BytesMut) -> Self {
        todo!()
    }
}

// fn split_at_line_end(content: &mut BytesMut) -> BytesMut {

// }