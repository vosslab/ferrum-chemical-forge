//! Checked little-endian byte reader shared by native adapter codecs.

use super::DecodeFailure;

pub(super) struct Reader<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) cursor: usize,
}
impl<'a> Reader<'a> {
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }
    pub(super) fn take(&mut self, length: usize) -> Result<&'a [u8], DecodeFailure> {
        let end = self
            .cursor
            .checked_add(length)
            .ok_or(DecodeFailure::Truncated)?;
        let result = self
            .bytes
            .get(self.cursor..end)
            .ok_or(DecodeFailure::Truncated)?;
        self.cursor = end;
        Ok(result)
    }
    pub(super) fn u8(&mut self) -> Result<u8, DecodeFailure> {
        Ok(self.take(1)?[0])
    }
    pub(super) fn u16(&mut self) -> Result<u16, DecodeFailure> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("fixed length"),
        ))
    }
    pub(super) fn u32(&mut self) -> Result<u32, DecodeFailure> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("fixed length"),
        ))
    }
    pub(super) fn i32(&mut self) -> Result<i32, DecodeFailure> {
        Ok(i32::from_le_bytes(
            self.take(4)?.try_into().expect("fixed length"),
        ))
    }
    pub(super) fn is_empty(&self) -> bool {
        self.cursor == self.bytes.len()
    }
}

pub(super) fn put_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}
pub(super) fn put_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}
pub(super) fn put_i32(output: &mut Vec<u8>, value: i32) {
    output.extend_from_slice(&value.to_le_bytes());
}
