use bytes::{Bytes, BytesMut};

pub trait VgmParser {
    fn from_bytes(data: &mut Bytes) -> Self;
}

pub trait VgmWriter {
    fn to_bytes(&self, buffer: &mut BytesMut);
}