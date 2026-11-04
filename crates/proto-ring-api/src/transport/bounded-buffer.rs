use serde::Serialize;
use std::io::{self, Write};

#[cfg(test)]
#[path = "../tests/bounded-buffer.rs"]
mod tests;

pub(super) struct BoundedBuffer {
    bytes: Vec<u8>,
    limit: usize,
    overflow: bool,
}

impl BoundedBuffer {
    pub(super) fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
            overflow: false,
        }
    }
    pub(super) fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

impl Write for BoundedBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let Some(end) = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .filter(|n| *n <= self.limit)
        else {
            self.overflow = true;
            return Err(io::Error::other("encoded byte limit exceeded"));
        };
        if end > self.bytes.capacity() {
            let capacity = end
                .max(self.bytes.capacity().saturating_mul(2))
                .min(self.limit);
            self.bytes
                .try_reserve_exact(capacity - self.bytes.len())
                .map_err(io::Error::other)?;
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) enum EncodingFailure {
    Limit,
    Internal,
}

pub(super) fn serialize(value: &impl Serialize, limit: usize) -> Result<Vec<u8>, EncodingFailure> {
    let mut buffer = BoundedBuffer::new(limit);
    match serde_json::to_writer(&mut buffer, value) {
        Ok(()) => Ok(buffer.finish()),
        Err(_) if buffer.overflow => Err(EncodingFailure::Limit),
        Err(_) => Err(EncodingFailure::Internal),
    }
}
