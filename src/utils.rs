use crate::errors::{VgmError, VgmResult};
use bytes::{BufMut, BytesMut};
use flate2::read::GzDecoder;
use std::io::Read;

/// Gzip magic bytes (RFC 1952)
pub const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

/// VGM magic bytes
pub const VGM_MAGIC: [u8; 4] = [0x56, 0x67, 0x6d, 0x20]; // "Vgm "

pub fn write_string_as_u16_bytes(buffer: &mut BytesMut, value: &str) {
    buffer.put(
        &value
            .encode_utf16()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<u8>>()[..],
    );
}

fn bcd_to_decimal(byte: u8) -> u32 {
    (((byte >> 4) * 10) + (byte & 0x0F)) as u32
}

/// Read bytes and return bcd version
/// For example [0x51, 0x01, 0x00, 0x00] will return 151
pub fn bcd_from_bytes(bytes_list: &[u8]) -> u32 {
    let mut bytes_list = bytes_list.to_vec();
    bytes_list.reverse();
    let mut version = 0;
    for byte in bytes_list {
        version = version * 100 + bcd_to_decimal(byte);
    }

    version
}

/// Read decimal and return bcd version as bytes
/// For example 151 will return [0x51, 0x01, 0x00, 0x00]
pub fn decimal_to_bcd(decimal: u32) -> Vec<u8> {
    //let decimal = decimal as u128;
    let mut bcd_bytes = Vec::new();
    let mut remaining = decimal;

    while remaining > 0 {
        let digit = remaining % 100;
        let bcd_byte = ((digit / 10) << 4) | (digit % 10);
        bcd_bytes.push(bcd_byte as u8);
        remaining /= 100;
    }

    //bcd_bytes.reverse();
    // pad bcd_bytes to get target length
    bcd_bytes.push(0x00);
    bcd_bytes.push(0x00);

    bcd_bytes
}

/// Detect if data is gzipped by checking magic bytes
pub fn is_gzipped(data: &[u8]) -> bool {
    data.len() >= 2 && data[0..2] == GZIP_MAGIC
}

/// Detect if data is a VGM file by checking magic bytes
pub fn is_vgm(data: &[u8]) -> bool {
    data.len() >= 4 && data[0..4] == VGM_MAGIC
}

/// Decompress gzipped data
pub fn decompress_gzip(compressed_data: &[u8]) -> VgmResult<Vec<u8>> {
    if !is_gzipped(compressed_data) {
        return Err(VgmError::InvalidDataFormat {
            field: "gzip_header".to_string(),
            details: "Data does not have valid gzip magic bytes".to_string(),
        });
    }

    let mut decoder = GzDecoder::new(compressed_data);
    let mut decompressed = Vec::new();

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| VgmError::InvalidDataFormat {
            field: "gzip_decompression".to_string(),
            details: format!("Failed to decompress gzip data: {}", e),
        })?;

    Ok(decompressed)
}

/// Detect file format and decompress if necessary
/// Returns the raw VGM data regardless of whether input was .vgm or .vgz
pub fn detect_and_decompress(data: &[u8]) -> VgmResult<Vec<u8>> {
    // First check if it's already a VGM file
    if is_vgm(data) {
        return Ok(data.to_vec());
    }

    // Check if it's gzipped
    if is_gzipped(data) {
        let decompressed = decompress_gzip(data)?;

        // Verify the decompressed data is a valid VGM file
        if !is_vgm(&decompressed) {
            return Err(VgmError::InvalidDataFormat {
                field: "decompressed_vgm".to_string(),
                details: "Decompressed data does not contain valid VGM magic bytes".to_string(),
            });
        }

        return Ok(decompressed);
    }

    // If neither VGM nor gzip, it's an unknown format
    Err(VgmError::InvalidDataFormat {
        field: "file_format".to_string(),
        details: "File is neither a valid VGM nor VGZ (gzipped VGM) format".to_string(),
    })
}

#[cfg(test)]
mod test_utils {
    use super::*;
    use crate::utils::decimal_to_bcd;
    use flate2::{write::GzEncoder, Compression};
    use std::io::Write;

    #[test]
    fn test_bcd_to_decimal() {
        // Test basic BCD to decimal conversion
        assert_eq!(bcd_to_decimal(0x00), 0);
        assert_eq!(bcd_to_decimal(0x01), 1);
        assert_eq!(bcd_to_decimal(0x09), 9);
        assert_eq!(bcd_to_decimal(0x10), 10);
        assert_eq!(bcd_to_decimal(0x19), 19);
        assert_eq!(bcd_to_decimal(0x51), 51);
        assert_eq!(bcd_to_decimal(0x99), 99);
    }

    #[test]
    fn test_bcd_from_bytes() {
        // Test basic cases
        assert_eq!(bcd_from_bytes(&[0x00]), 0);
        assert_eq!(bcd_from_bytes(&[0x01]), 1);
        assert_eq!(bcd_from_bytes(&[0x51]), 51);
        
        // Test multi-byte cases
        assert_eq!(bcd_from_bytes(&[0x51, 0x01]), 151);
        assert_eq!(bcd_from_bytes(&[0x71, 0x01]), 171);
        assert_eq!(bcd_from_bytes(&[0x24, 0x01]), 124);
        
        // Test VGM standard 4-byte format
        assert_eq!(bcd_from_bytes(&[0x51, 0x01, 0x00, 0x00]), 151);
        assert_eq!(bcd_from_bytes(&[0x71, 0x01, 0x00, 0x00]), 171);
        assert_eq!(bcd_from_bytes(&[0x24, 0x01, 0x00, 0x00]), 124);
        
        // Test edge cases
        assert_eq!(bcd_from_bytes(&[0x00, 0x00, 0x00, 0x00]), 0);
        assert_eq!(bcd_from_bytes(&[0x99, 0x99]), 9999);
    }

    #[test]
    fn test_decimal_to_bcd() {
        // Test basic cases - note: decimal_to_bcd(0) returns [0x00, 0x00] because the while loop doesn't execute
        assert_eq!(decimal_to_bcd(0), vec![0x00, 0x00]);
        assert_eq!(decimal_to_bcd(1), vec![0x01, 0x00, 0x00]);
        assert_eq!(decimal_to_bcd(51), vec![0x51, 0x00, 0x00]);
        assert_eq!(decimal_to_bcd(99), vec![0x99, 0x00, 0x00]);
        
        // Test multi-byte cases
        assert_eq!(decimal_to_bcd(151), vec![0x51, 0x01, 0x00, 0x00]);
        assert_eq!(decimal_to_bcd(171), vec![0x71, 0x01, 0x00, 0x00]);
        assert_eq!(decimal_to_bcd(124), vec![0x24, 0x01, 0x00, 0x00]);
        
        // Test larger numbers
        assert_eq!(decimal_to_bcd(9999), vec![0x99, 0x99, 0x00, 0x00]);
        assert_eq!(decimal_to_bcd(12345), vec![0x45, 0x23, 0x01, 0x00, 0x00]);
    }

    #[test]
    fn bcd_cycle() {
        let version_bytes: [u8; 4] = [0x51, 0x01, 0x00, 0x00];
        let version = bcd_from_bytes(&version_bytes[..]);
        assert_eq!(version, 151);
        let out_bytes = decimal_to_bcd(version);
        assert_eq!(version_bytes.to_vec(), out_bytes);

        let version_bytes: [u8; 4] = [0x71, 0x01, 0x00, 0x00];
        let version = bcd_from_bytes(&version_bytes[..]);
        assert_eq!(version, 171);
        let out_bytes = decimal_to_bcd(version);
        assert_eq!(version_bytes.to_vec(), out_bytes);

        let version_bytes: [u8; 4] = [0x24, 0x01, 0x00, 0x00];
        let version = bcd_from_bytes(&version_bytes[..]);
        assert_eq!(version, 124);
        let out_bytes = decimal_to_bcd(version);
        assert_eq!(version_bytes.to_vec(), out_bytes);
    }

    #[test]
    fn test_bcd_round_trip_comprehensive() {
        // Test a range of values for round-trip consistency
        let test_values = [0, 1, 9, 10, 51, 99, 100, 150, 151, 171, 200, 999, 1234, 9999];
        
        for &value in &test_values {
            let bcd_bytes = decimal_to_bcd(value);
            let recovered_value = bcd_from_bytes(&bcd_bytes);
            assert_eq!(value, recovered_value, "Round-trip failed for value {}", value);
        }
    }

    #[test]
    fn test_write_string_as_u16_bytes() {
        let mut buffer = BytesMut::new();
        
        // Test empty string
        write_string_as_u16_bytes(&mut buffer, "");
        assert_eq!(buffer.len(), 0);
        
        // Test basic ASCII string
        buffer.clear();
        write_string_as_u16_bytes(&mut buffer, "A");
        assert_eq!(buffer.to_vec(), vec![0x41, 0x00]); // 'A' in little-endian UTF-16
        
        // Test multi-character ASCII string
        buffer.clear();
        write_string_as_u16_bytes(&mut buffer, "AB");
        assert_eq!(buffer.to_vec(), vec![0x41, 0x00, 0x42, 0x00]); // "AB" in little-endian UTF-16
        
        // Test Unicode characters
        buffer.clear();
        write_string_as_u16_bytes(&mut buffer, "♪");
        assert_eq!(buffer.to_vec(), vec![0x6A, 0x26]); // Musical note symbol in little-endian UTF-16
        
        // Test mixed ASCII and Unicode
        buffer.clear();
        write_string_as_u16_bytes(&mut buffer, "A♪");
        assert_eq!(buffer.to_vec(), vec![0x41, 0x00, 0x6A, 0x26]);
    }

    #[test]
    fn test_write_string_as_u16_bytes_comprehensive() {
        let mut buffer = BytesMut::new();
        
        // Test common VGM metadata strings
        let test_strings = [
            "Test Track",
            "Sonic the Hedgehog",
            "SEGA Genesis",
            "Yuzo Koshiro",
            "Streets of Rage 2",
            "1991",
        ];
        
        for test_string in &test_strings {
            buffer.clear();
            write_string_as_u16_bytes(&mut buffer, test_string);
            
            // Verify length is correct (each character should be 2 bytes)
            let expected_len = test_string.chars().count() * 2;
            assert_eq!(buffer.len(), expected_len, "Incorrect length for string: {}", test_string);
            
            // Verify we can decode back to UTF-16
            let u16_chars: Vec<u16> = buffer.chunks_exact(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                .collect();
            let decoded = String::from_utf16(&u16_chars).unwrap();
            assert_eq!(decoded, *test_string);
        }
    }

    #[test]
    fn test_write_string_as_u16_bytes_unicode() {
        let mut buffer = BytesMut::new();
        
        // Test various Unicode characters commonly found in VGM metadata
        // Use a more robust approach that verifies round-trip encoding rather than hardcoded bytes
        let unicode_tests = [
            "日本", // "Japan" in Japanese
            "©",    // Copyright symbol  
            "™",    // Trademark symbol
            "♪",    // Musical note
            "🎵",   // Musical note emoji
            "Ñ",    // Spanish ñ
            "ü",    // Umlaut
        ];
        
        for input in &unicode_tests {
            buffer.clear();
            write_string_as_u16_bytes(&mut buffer, input);
            
            // Verify we can decode back to the original string
            let u16_chars: Vec<u16> = buffer.chunks_exact(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                .collect();
            let decoded = String::from_utf16(&u16_chars).unwrap();
            assert_eq!(decoded, *input, "Round-trip failed for Unicode string: {}", input);
            
            // Verify the length is correct (each UTF-16 code unit is 2 bytes)
            let expected_byte_len = input.encode_utf16().count() * 2;
            assert_eq!(buffer.len(), expected_byte_len, "Incorrect byte length for: {}", input);
        }
    }

    #[test]
    fn test_magic_bytes_constants() {
        // Verify the magic byte constants are correct
        assert_eq!(VGM_MAGIC, [0x56, 0x67, 0x6d, 0x20]); // "Vgm "
        assert_eq!(GZIP_MAGIC, [0x1f, 0x8b]);
        
        // Test that the constants work with the detection functions
        assert!(is_vgm(&VGM_MAGIC));
        assert!(is_gzipped(&GZIP_MAGIC));
    }

    #[test]
    fn test_magic_bytes_detection() {
        // Test VGM magic bytes
        let vgm_data = b"Vgm \x00\x00\x00\x00"; // VGM header start
        assert!(is_vgm(vgm_data));
        assert!(!is_gzipped(vgm_data));

        // Test gzip magic bytes
        let gzip_data = [0x1f, 0x8b, 0x08, 0x00]; // Standard gzip header start
        assert!(is_gzipped(&gzip_data));
        assert!(!is_vgm(&gzip_data));

        // Test invalid data
        let invalid_data = b"INVALID_DATA";
        assert!(!is_vgm(invalid_data));
        assert!(!is_gzipped(invalid_data));
    }

    #[test]
    fn test_magic_bytes_detection_comprehensive() {
        // Test VGM variations
        assert!(is_vgm(b"Vgm "));
        assert!(is_vgm(b"Vgm \x12\x34\x56\x78")); // VGM with additional data
        assert!(!is_vgm(b"vgm ")); // Wrong case
        assert!(!is_vgm(b"VGM ")); // Wrong case
        assert!(!is_vgm(b"Vgn ")); // Wrong character
        
        // Test gzip variations
        assert!(is_gzipped(&[0x1f, 0x8b])); // Minimal valid gzip header
        assert!(is_gzipped(&[0x1f, 0x8b, 0x08])); // Common gzip header
        assert!(!is_gzipped(&[0x1f, 0x8c])); // Wrong second byte
        assert!(!is_gzipped(&[0x1e, 0x8b])); // Wrong first byte
    }

    #[test]
    fn test_is_vgm_edge_cases() {
        // Test insufficient data
        assert!(!is_vgm(&[]));
        assert!(!is_vgm(&[0x56]));
        assert!(!is_vgm(&[0x56, 0x67]));
        assert!(!is_vgm(&[0x56, 0x67, 0x6d])); // 3 bytes, need 4
        
        // Test exact length
        assert!(is_vgm(&[0x56, 0x67, 0x6d, 0x20])); // Exactly 4 bytes
    }

    #[test]
    fn test_is_gzipped_edge_cases() {
        // Test insufficient data
        assert!(!is_gzipped(&[]));
        assert!(!is_gzipped(&[0x1f])); // Only 1 byte, need 2
        
        // Test exact length
        assert!(is_gzipped(&[0x1f, 0x8b])); // Exactly 2 bytes
    }

    #[test]
    fn test_gzip_compression_decompression() {
        // Create mock VGM data
        let mut vgm_data = Vec::new();
        vgm_data.extend_from_slice(&VGM_MAGIC); // VGM magic bytes
        vgm_data.extend_from_slice(&[0x00; 60]); // Pad to minimum size

        // Compress with gzip
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&vgm_data).unwrap();
        let compressed = encoder.finish().unwrap();

        // Verify it's detected as gzipped
        assert!(is_gzipped(&compressed));

        // Decompress and verify
        let decompressed = decompress_gzip(&compressed).unwrap();
        assert_eq!(decompressed, vgm_data);
        assert!(is_vgm(&decompressed));
    }

    #[test]
    fn test_decompress_gzip_various_compression_levels() {
        let test_data = b"Vgm \x00\x00\x00\x00\x01\x02\x03\x04\x05\x06\x07\x08";
        
        // Test different compression levels
        let compression_levels = [
            Compression::none(),
            Compression::fast(),
            Compression::default(),
            Compression::best(),
        ];
        
        for compression in &compression_levels {
            let mut encoder = GzEncoder::new(Vec::new(), *compression);
            encoder.write_all(test_data).unwrap();
            let compressed = encoder.finish().unwrap();
            
            assert!(is_gzipped(&compressed));
            let decompressed = decompress_gzip(&compressed).unwrap();
            assert_eq!(decompressed, test_data);
        }
    }

    #[test]
    fn test_detect_and_decompress_vgm() {
        // Test with raw VGM data
        let mut vgm_data = Vec::new();
        vgm_data.extend_from_slice(&VGM_MAGIC);
        vgm_data.extend_from_slice(&[0x00; 60]);

        let result = detect_and_decompress(&vgm_data).unwrap();
        assert_eq!(result, vgm_data);
    }

    #[test]
    fn test_detect_and_decompress_vgz() {
        // Create mock VGM data
        let mut vgm_data = Vec::new();
        vgm_data.extend_from_slice(&VGM_MAGIC);
        vgm_data.extend_from_slice(&[0x00; 60]);

        // Compress with gzip
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&vgm_data).unwrap();
        let compressed = encoder.finish().unwrap();

        // Test decompression
        let result = detect_and_decompress(&compressed).unwrap();
        assert_eq!(result, vgm_data);
    }

    #[test]
    fn test_detect_and_decompress_large_file() {
        // Create a larger VGM file for testing
        let mut vgm_data = Vec::new();
        vgm_data.extend_from_slice(&VGM_MAGIC);
        vgm_data.extend_from_slice(&vec![0xAA; 10000]); // 10KB of test data
        
        // Test raw VGM
        let result = detect_and_decompress(&vgm_data).unwrap();
        assert_eq!(result, vgm_data);
        
        // Test compressed VGM
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&vgm_data).unwrap();
        let compressed = encoder.finish().unwrap();
        
        let result = detect_and_decompress(&compressed).unwrap();
        assert_eq!(result, vgm_data);
    }

    #[test]
    fn test_detect_and_decompress_invalid_format() {
        let invalid_data = b"INVALID_DATA_FORMAT";
        let result = detect_and_decompress(invalid_data);
        assert!(result.is_err());

        match result.unwrap_err() {
            VgmError::InvalidDataFormat { field, .. } => {
                assert_eq!(field, "file_format");
            },
            _ => panic!("Expected InvalidDataFormat error"),
        }
    }

    #[test]
    fn test_decompress_gzip_invalid_data() {
        let invalid_gzip = b"NOT_GZIP_DATA";
        let result = decompress_gzip(invalid_gzip);
        assert!(result.is_err());

        match result.unwrap_err() {
            VgmError::InvalidDataFormat { field, .. } => {
                assert_eq!(field, "gzip_header");
            },
            _ => panic!("Expected InvalidDataFormat error"),
        }
    }

    #[test]
    fn test_decompress_gzip_corrupted_data() {
        // Create valid gzip header but corrupted data
        let mut corrupted_gzip = vec![0x1f, 0x8b, 0x08, 0x00]; // Valid gzip header
        corrupted_gzip.extend_from_slice(&[0xFF; 20]); // Corrupted payload
        
        let result = decompress_gzip(&corrupted_gzip);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            VgmError::InvalidDataFormat { field, details } => {
                assert_eq!(field, "gzip_decompression");
                assert!(details.contains("Failed to decompress"));
            },
            _ => panic!("Expected InvalidDataFormat error for gzip_decompression"),
        }
    }

    #[test]
    fn test_gzipped_non_vgm_data() {
        // Compress non-VGM data
        let non_vgm_data = b"NOT_A_VGM_FILE_DATA";
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(non_vgm_data).unwrap();
        let compressed = encoder.finish().unwrap();

        // Should fail because decompressed data is not VGM
        let result = detect_and_decompress(&compressed);
        assert!(result.is_err());

        match result.unwrap_err() {
            VgmError::InvalidDataFormat { field, .. } => {
                assert_eq!(field, "decompressed_vgm");
            },
            _ => panic!("Expected InvalidDataFormat error for decompressed_vgm"),
        }
    }

    #[test]
    fn test_edge_cases() {
        // Test empty data
        assert!(!is_vgm(&[]));
        assert!(!is_gzipped(&[]));

        // Test too short data
        assert!(!is_vgm(&[0x56, 0x67])); // Only 2 bytes of VGM magic
        assert!(!is_gzipped(&[0x1f])); // Only 1 byte of gzip magic

        // Test partial magic matches
        assert!(!is_vgm(b"Vgx ")); // Wrong 3rd byte
        assert!(!is_gzipped(&[0x1f, 0x8c])); // Wrong 2nd byte
    }

    #[test]
    fn test_detect_and_decompress_empty_input() {
        let result = detect_and_decompress(&[]);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            VgmError::InvalidDataFormat { field, .. } => {
                assert_eq!(field, "file_format");
            },
            _ => panic!("Expected InvalidDataFormat error"),
        }
    }

    #[test]
    fn test_detect_and_decompress_minimal_input() {
        // Test with minimal data that doesn't match any format
        let minimal_data = [0x00];
        let result = detect_and_decompress(&minimal_data);
        assert!(result.is_err());
        
        // Test with data that partially matches but is too short
        let partial_vgm = [0x56, 0x67]; // First 2 bytes of VGM magic
        let result = detect_and_decompress(&partial_vgm);
        assert!(result.is_err());
        
        let partial_gzip = [0x1f]; // First byte of gzip magic
        let result = detect_and_decompress(&partial_gzip);
        assert!(result.is_err());
    }

    #[test]
    fn test_constants_usage() {
        // Test that our constants work in practice
        let mut vgm_file = Vec::new();
        vgm_file.extend_from_slice(&VGM_MAGIC);
        vgm_file.extend_from_slice(&[0x00; 60]);
        
        assert!(is_vgm(&vgm_file));
        assert_eq!(&vgm_file[0..4], &VGM_MAGIC);
        
        // Test with minimal gzip file
        let mut gzip_data = Vec::new();
        gzip_data.extend_from_slice(&GZIP_MAGIC);
        gzip_data.extend_from_slice(&[0x08, 0x00]); // Basic gzip header continuation
        
        assert!(is_gzipped(&gzip_data));
        assert_eq!(&gzip_data[0..2], &GZIP_MAGIC);
    }

    #[test]
    fn test_utf16_encoding_edge_cases() {
        let mut buffer = BytesMut::new();
        
        // Test strings with special characters that might cause issues
        let special_strings = [
            "\0", // Null character
            "\u{FEFF}", // Byte Order Mark
            "\u{FFFF}", // Special Unicode character
            "Test\nNewline", // String with newline
            "Tab\tCharacter", // String with tab
        ];
        
        for test_string in &special_strings {
            buffer.clear();
            write_string_as_u16_bytes(&mut buffer, test_string);
            
            // Verify we can decode back
            let u16_chars: Vec<u16> = buffer.chunks_exact(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                .collect();
            let decoded = String::from_utf16(&u16_chars).unwrap();
            assert_eq!(decoded, *test_string, "Failed for special string: {:?}", test_string);
        }
    }

    #[test]
    fn test_bcd_boundary_values() {
        // Test BCD conversion with boundary values
        let boundary_tests = [
            (0x00, 0),
            (0x09, 9),
            (0x10, 10),
            (0x90, 90),
            (0x99, 99),
        ];
        
        for (bcd_byte, expected_decimal) in &boundary_tests {
            assert_eq!(bcd_to_decimal(*bcd_byte), *expected_decimal);
        }
        
        // Test decimal to BCD boundaries
        let decimal_tests = [0, 1, 9, 10, 51, 99, 100, 999, 1000];
        for &decimal in &decimal_tests {
            let bcd_bytes = decimal_to_bcd(decimal);
            let recovered = bcd_from_bytes(&bcd_bytes);
            assert_eq!(decimal, recovered, "BCD round-trip failed for {}", decimal);
        }
    }

    #[test]
    fn test_function_isolation() {
        // Test that functions work independently and don't affect each other
        
        // Test VGM detection doesn't interfere with gzip detection
        let mixed_data = b"Vgm \x1f\x8b\x08\x00";
        assert!(is_vgm(mixed_data)); // Should detect VGM at start
        assert!(!is_gzipped(mixed_data)); // Should not detect gzip when VGM is at start
        
        let gzip_first = b"\x1f\x8bVgm ";
        assert!(is_gzipped(gzip_first)); // Should detect gzip at start
        assert!(!is_vgm(gzip_first)); // Should not detect VGM when gzip is at start
    }
}
