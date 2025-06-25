use crate::errors::{VgmError, VgmResult};
use bytes::{Bytes, BytesMut};

pub trait VgmParser {
    fn from_bytes(data: &mut Bytes) -> VgmResult<Self>
    where
        Self: Sized;
}

pub trait VgmWriter {
    fn to_bytes(&self, buffer: &mut BytesMut) -> VgmResult<()>;
}
