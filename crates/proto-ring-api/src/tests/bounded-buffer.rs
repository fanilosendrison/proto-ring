use super::{BoundedBuffer, EncodingFailure, serialize};
use std::io::Write;

#[test]
fn serializer_counts_encoded_utf8_and_escapes_at_the_exact_sink_budget() {
    let expected = vec![b'"', 0xc3, 0xa9, b'\\', b'n', b'"'];
    assert_eq!(serialize(&"é\n", 6).ok(), Some(expected));
    assert!(matches!(serialize(&"é\n", 5), Err(EncodingFailure::Limit)));
    assert!(matches!(serialize(&"", 1), Err(EncodingFailure::Limit)));
    assert_eq!(serialize(&"", 2).ok(), Some(b"\"\"".to_vec()));
}
#[test]
fn failed_chunk_never_extends_buffer_or_capacity_beyond_budget() {
    let mut sink = BoundedBuffer::new(4);
    sink.write_all(b"ab").unwrap();
    assert!(sink.write_all(b"cde").is_err());
    assert!(sink.overflow);
    assert_eq!(sink.bytes, b"ab");
    assert!(sink.bytes.capacity() <= 4);
}
