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

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Buf, BufMut, Bytes, BytesMut};
    use crate::errors::VgmError;

    // Mock struct for testing VgmParser trait
    #[derive(Debug, PartialEq, Clone)]
    struct MockData {
        value: u32,
        text: String,
    }

    impl VgmParser for MockData {
        fn from_bytes(data: &mut Bytes) -> VgmResult<Self> {
            if data.len() < 4 {
                return Err(VgmError::BufferUnderflow {
                    offset: 0,
                    needed: 4,
                    available: data.len(),
                });
            }

            let value = data.get_u32_le();
            
            if data.is_empty() {
                return Err(VgmError::BufferUnderflow {
                    offset: 4,
                    needed: 1,
                    available: 0,
                });
            }

            let text_len = data.get_u8() as usize;
            
            if data.len() < text_len {
                return Err(VgmError::BufferUnderflow {
                    offset: 5,
                    needed: text_len,
                    available: data.len(),
                });
            }

            let text_bytes = data.split_to(text_len);
            let text = String::from_utf8(text_bytes.to_vec())
                .map_err(|_| VgmError::InvalidDataFormat {
                    field: "text".to_string(),
                    details: "Invalid UTF-8 sequence".to_string(),
                })?;

            Ok(MockData { value, text })
        }
    }

    impl VgmWriter for MockData {
        fn to_bytes(&self, buffer: &mut BytesMut) -> VgmResult<()> {
            if self.text.len() > 255 {
                return Err(VgmError::DataSizeExceedsLimit {
                    field: "text".to_string(),
                    size: self.text.len(),
                    limit: 255,
                });
            }

            buffer.put_u32_le(self.value);
            buffer.put_u8(self.text.len() as u8);
            buffer.put_slice(self.text.as_bytes());

            Ok(())
        }
    }

    // Alternative mock for testing error conditions
    #[derive(Debug)]
    struct FailingParser;

    impl VgmParser for FailingParser {
        fn from_bytes(_data: &mut Bytes) -> VgmResult<Self> {
            Err(VgmError::InvalidDataFormat {
                field: "test".to_string(),
                details: "Intentional test failure".to_string(),
            })
        }
    }

    impl VgmWriter for FailingParser {
        fn to_bytes(&self, _buffer: &mut BytesMut) -> VgmResult<()> {
            Err(VgmError::InvalidDataFormat {
                field: "test".to_string(),
                details: "Intentional write failure".to_string(),
            })
        }
    }

    // Minimal parser for testing edge cases
    #[derive(Debug, PartialEq)]
    struct MinimalData {
        byte: u8,
    }

    impl VgmParser for MinimalData {
        fn from_bytes(data: &mut Bytes) -> VgmResult<Self> {
            if data.is_empty() {
                return Err(VgmError::BufferUnderflow {
                    offset: 0,
                    needed: 1,
                    available: 0,
                });
            }
            Ok(MinimalData { byte: data.get_u8() })
        }
    }

    impl VgmWriter for MinimalData {
        fn to_bytes(&self, buffer: &mut BytesMut) -> VgmResult<()> {
            buffer.put_u8(self.byte);
            Ok(())
        }
    }

    // Empty data structure
    #[derive(Debug, PartialEq)]
    struct EmptyData;

    impl VgmParser for EmptyData {
        fn from_bytes(_data: &mut Bytes) -> VgmResult<Self> {
            Ok(EmptyData)
        }
    }

    impl VgmWriter for EmptyData {
        fn to_bytes(&self, _buffer: &mut BytesMut) -> VgmResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_vgm_parser_trait_exists() {
        // Test that the VgmParser trait is accessible and can be used
        let _parser: fn(&mut Bytes) -> VgmResult<MockData> = MockData::from_bytes;
        // If this compiles, the trait is properly defined
    }

    #[test]
    fn test_vgm_writer_trait_exists() {
        // Test that the VgmWriter trait is accessible and can be used
        let mock = MockData { value: 42, text: "test".to_string() };
        let mut buffer = BytesMut::new();
        let _result = mock.to_bytes(&mut buffer);
        // If this compiles, the trait is properly defined
    }

    #[test]
    fn test_mock_data_parser_success() {
        // Test successful parsing of MockData
        let mut buffer = BytesMut::new();
        buffer.put_u32_le(12345);
        buffer.put_u8(5); // Text length
        buffer.put_slice(b"hello");
        
        let mut data = Bytes::from(buffer);
        let result = MockData::from_bytes(&mut data);
        
        assert!(result.is_ok());
        let mock = result.unwrap();
        assert_eq!(mock.value, 12345);
        assert_eq!(mock.text, "hello");
        assert!(data.is_empty()); // All data should be consumed
    }

    #[test]
    fn test_mock_data_parser_insufficient_data() {
        // Test parser error with insufficient data
        let mut data = Bytes::from(vec![1, 2]); // Only 2 bytes, need at least 4 for u32
        let result = MockData::from_bytes(&mut data);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::BufferUnderflow { offset: 0, needed: 4, available: 2 }));
    }

    #[test]
    fn test_mock_data_parser_missing_text_length() {
        // Test parser error when text length byte is missing
        let mut buffer = BytesMut::new();
        buffer.put_u32_le(12345);
        // Missing text length and text data
        
        let mut data = Bytes::from(buffer);
        let result = MockData::from_bytes(&mut data);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::BufferUnderflow { offset: 4, needed: 1, available: 0 }));
    }

    #[test]
    fn test_mock_data_parser_insufficient_text_data() {
        // Test parser error when declared text length exceeds available data
        let mut buffer = BytesMut::new();
        buffer.put_u32_le(12345);
        buffer.put_u8(10); // Declare 10 bytes of text
        buffer.put_slice(b"short"); // But only provide 5 bytes
        
        let mut data = Bytes::from(buffer);
        let result = MockData::from_bytes(&mut data);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::BufferUnderflow { offset: 5, needed: 10, available: 5 }));
    }

    #[test]
    fn test_mock_data_parser_invalid_utf8() {
        // Test parser error with invalid UTF-8 sequence
        let mut buffer = BytesMut::new();
        buffer.put_u32_le(12345);
        buffer.put_u8(3); // Text length
        buffer.put_slice(&[0xFF, 0xFE, 0xFD]); // Invalid UTF-8
        
        let mut data = Bytes::from(buffer);
        let result = MockData::from_bytes(&mut data);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::InvalidDataFormat { field, details } => {
                assert_eq!(field, "text");
                assert!(details.contains("Invalid UTF-8"));
            },
            _ => panic!("Expected InvalidDataFormat error"),
        }
    }

    #[test]
    fn test_mock_data_writer_success() {
        // Test successful writing of MockData
        let mock = MockData { value: 54321, text: "world".to_string() };
        let mut buffer = BytesMut::new();
        
        let result = mock.to_bytes(&mut buffer);
        assert!(result.is_ok());
        
        // Verify written data
        assert_eq!(buffer.len(), 4 + 1 + 5); // u32 + len byte + text
        
        let mut data = Bytes::from(buffer);
        assert_eq!(data.get_u32_le(), 54321);
        assert_eq!(data.get_u8(), 5);
        assert_eq!(&data[..], b"world");
    }

    #[test]
    fn test_mock_data_writer_text_too_long() {
        // Test writer error when text is too long
        let long_text = "x".repeat(256); // Exceeds 255 byte limit
        let mock = MockData { value: 42, text: long_text };
        let mut buffer = BytesMut::new();
        
        let result = mock.to_bytes(&mut buffer);
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::DataSizeExceedsLimit { field, size, limit } => {
                assert_eq!(field, "text");
                assert_eq!(size, 256);
                assert_eq!(limit, 255);
            },
            _ => panic!("Expected DataSizeExceedsLimit error"),
        }
    }

    #[test]
    fn test_round_trip_parsing() {
        // Test that parse -> write -> parse produces the same result
        let original = MockData { value: 98765, text: "roundtrip".to_string() };
        
        // Write to bytes
        let mut buffer = BytesMut::new();
        original.to_bytes(&mut buffer).unwrap();
        
        // Parse back
        let mut data = Bytes::from(buffer);
        let parsed = MockData::from_bytes(&mut data).unwrap();
        
        // Should be identical
        assert_eq!(original, parsed);
        assert!(data.is_empty()); // All data consumed
    }

    #[test]
    fn test_failing_parser() {
        // Test parser that always fails
        let mut data = Bytes::from(vec![1, 2, 3, 4]);
        let result = FailingParser::from_bytes(&mut data);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::InvalidDataFormat { field, details } => {
                assert_eq!(field, "test");
                assert!(details.contains("Intentional test failure"));
            },
            _ => panic!("Expected InvalidDataFormat error"),
        }
    }

    #[test]
    fn test_failing_writer() {
        // Test writer that always fails
        let failing = FailingParser;
        let mut buffer = BytesMut::new();
        let result = failing.to_bytes(&mut buffer);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::InvalidDataFormat { field, details } => {
                assert_eq!(field, "test");
                assert!(details.contains("Intentional write failure"));
            },
            _ => panic!("Expected InvalidDataFormat error"),
        }
    }

    #[test]
    fn test_minimal_data_parser() {
        // Test minimal parser with single byte
        let mut data = Bytes::from(vec![42]);
        let result = MinimalData::from_bytes(&mut data);
        
        assert!(result.is_ok());
        let minimal = result.unwrap();
        assert_eq!(minimal.byte, 42);
        assert!(data.is_empty());
    }

    #[test]
    fn test_minimal_data_parser_empty() {
        // Test minimal parser with no data
        let mut data = Bytes::new();
        let result = MinimalData::from_bytes(&mut data);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::BufferUnderflow { offset: 0, needed: 1, available: 0 }));
    }

    #[test]
    fn test_minimal_data_writer() {
        // Test minimal writer
        let minimal = MinimalData { byte: 123 };
        let mut buffer = BytesMut::new();
        
        let result = minimal.to_bytes(&mut buffer);
        assert!(result.is_ok());
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer[0], 123);
    }

    #[test]
    fn test_empty_data_parser() {
        // Test parser that doesn't consume any data
        let mut data = Bytes::from(vec![1, 2, 3, 4]);
        let result = EmptyData::from_bytes(&mut data);
        
        assert!(result.is_ok());
        assert_eq!(data.len(), 4); // No data consumed
    }

    #[test]
    fn test_empty_data_writer() {
        // Test writer that doesn't write any data
        let empty = EmptyData;
        let mut buffer = BytesMut::new();
        
        let result = empty.to_bytes(&mut buffer);
        assert!(result.is_ok());
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_trait_bounds_compilation() {
        // Test that trait bounds work correctly for generic functions
        fn parse_any<T: VgmParser>(data: &mut Bytes) -> VgmResult<T> {
            T::from_bytes(data)
        }
        
        fn write_any<T: VgmWriter>(item: &T, buffer: &mut BytesMut) -> VgmResult<()> {
            item.to_bytes(buffer)
        }
        
        // Test with MockData
        let mut data = Bytes::from(vec![42, 0, 0, 0, 0]); // u32_le(42) + empty string
        let parsed: MockData = parse_any(&mut data).unwrap();
        assert_eq!(parsed.value, 42);
        assert_eq!(parsed.text, "");
        
        let mut buffer = BytesMut::new();
        write_any(&parsed, &mut buffer).unwrap();
        assert_eq!(buffer.len(), 5); // 4 bytes + 1 length byte + 0 text bytes
        
        // Test with MinimalData
        let mut data = Bytes::from(vec![99]);
        let parsed: MinimalData = parse_any(&mut data).unwrap();
        assert_eq!(parsed.byte, 99);
        
        let mut buffer = BytesMut::new();
        write_any(&parsed, &mut buffer).unwrap();
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer[0], 99);
    }

    #[test]
    fn test_trait_object_usage() {
        // Test that traits can be used as trait objects (dynamic dispatch)
        let mock = MockData { value: 777, text: "trait_object".to_string() };
        let minimal = MinimalData { byte: 88 };
        let empty = EmptyData;
        
        // Create trait objects
        let writers: Vec<&dyn VgmWriter> = vec![&mock, &minimal, &empty];
        
        // Test writing through trait objects
        for (i, writer) in writers.iter().enumerate() {
            let mut buffer = BytesMut::new();
            let result = writer.to_bytes(&mut buffer);
            assert!(result.is_ok(), "Writer {} failed", i);
            
            // Verify different writers produce different output sizes
            match i {
                0 => assert_eq!(buffer.len(), 4 + 1 + 12), // MockData: u32 + len + "trait_object"
                1 => assert_eq!(buffer.len(), 1),           // MinimalData: single byte
                2 => assert_eq!(buffer.len(), 0),           // EmptyData: nothing
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_parser_with_remaining_data() {
        // Test parser behavior when data remains after parsing
        let mut buffer = BytesMut::new();
        buffer.put_u32_le(555);
        buffer.put_u8(3);
        buffer.put_slice(b"abc");
        buffer.put_slice(b"extra_data"); // Extra data that should remain
        
        let mut data = Bytes::from(buffer);
        let original_len = data.len();
        
        let result = MockData::from_bytes(&mut data);
        assert!(result.is_ok());
        
        let mock = result.unwrap();
        assert_eq!(mock.value, 555);
        assert_eq!(mock.text, "abc");
        
        // Verify remaining data
        assert_eq!(data.len(), original_len - (4 + 1 + 3)); // Original minus consumed
        assert_eq!(&data[..], b"extra_data");
    }

    #[test]
    fn test_writer_multiple_writes() {
        // Test multiple writes to the same buffer
        let mock1 = MockData { value: 1, text: "first".to_string() };
        let mock2 = MockData { value: 2, text: "second".to_string() };
        
        let mut buffer = BytesMut::new();
        
        // Write first item
        mock1.to_bytes(&mut buffer).unwrap();
        let first_len = buffer.len();
        
        // Write second item
        mock2.to_bytes(&mut buffer).unwrap();
        let total_len = buffer.len();
        
        // Verify both were written
        assert!(total_len > first_len);
        assert_eq!(first_len, 4 + 1 + 5); // u32 + len + "first"
        assert_eq!(total_len, first_len + 4 + 1 + 6); // + u32 + len + "second"
    }

    #[test]
    fn test_error_type_consistency() {
        // Test that both traits use the same error type (VgmResult<T>)
        let mut empty_data = Bytes::new();
        let parse_error = MockData::from_bytes(&mut empty_data).unwrap_err();
        
        let failing = FailingParser;
        let mut buffer = BytesMut::new();
        let write_error = failing.to_bytes(&mut buffer).unwrap_err();
        
        // Both should be VgmError types
        assert!(matches!(parse_error, VgmError::BufferUnderflow { .. }));
        assert!(matches!(write_error, VgmError::InvalidDataFormat { .. }));
    }

    #[test]
    fn test_trait_method_signatures() {
        // Test that trait method signatures are as expected
        use std::any::TypeId;
        
        // VgmParser::from_bytes should take &mut Bytes and return VgmResult<Self>
        let _fn_type: fn(&mut Bytes) -> VgmResult<MockData> = MockData::from_bytes;
        
        // VgmWriter::to_bytes should take &self and &mut BytesMut and return VgmResult<()>
        let mock = MockData { value: 0, text: String::new() };
        let mut buffer = BytesMut::new();
        let _result: VgmResult<()> = mock.to_bytes(&mut buffer);
        
        // Verify VgmResult is indeed Result<T, VgmError>
        assert_eq!(TypeId::of::<VgmResult<()>>(), TypeId::of::<Result<(), VgmError>>());
    }

    #[test]
    fn test_unicode_text_handling() {
        // Test that unicode text is handled correctly
        let unicode_text = "Hello 世界 🌍 café naïve résumé";
        let mock = MockData { value: 12345, text: unicode_text.to_string() };
        
        // Write to bytes
        let mut buffer = BytesMut::new();
        let write_result = mock.to_bytes(&mut buffer);
        
        if write_result.is_err() {
            // If text is too long, that's expected behavior
            let error = write_result.unwrap_err();
            assert!(matches!(error, VgmError::DataSizeExceedsLimit { .. }));
            return;
        }
        
        // If write succeeded, test round-trip
        let mut data = Bytes::from(buffer);
        let parsed = MockData::from_bytes(&mut data).unwrap();
        
        assert_eq!(parsed.value, 12345);
        assert_eq!(parsed.text, unicode_text);
    }

    #[test]
    fn test_edge_case_zero_length_text() {
        // Test zero-length text specifically
        let mock = MockData { value: 99999, text: String::new() };
        
        let mut buffer = BytesMut::new();
        mock.to_bytes(&mut buffer).unwrap();
        
        // Should have u32 + 1 length byte + 0 text bytes = 5 bytes
        assert_eq!(buffer.len(), 5);
        
        let mut data = Bytes::from(buffer);
        let parsed = MockData::from_bytes(&mut data).unwrap();
        
        assert_eq!(parsed.value, 99999);
        assert_eq!(parsed.text, "");
        assert!(data.is_empty());
    }
}
