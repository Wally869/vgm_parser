use bytes::{BufMut, BytesMut};
use flate2::read::GzDecoder;
use std::io::Read;
use crate::errors::{VgmError, VgmResult};

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
    
    decoder.read_to_end(&mut decompressed).map_err(|e| {
        VgmError::InvalidDataFormat {
            field: "gzip_decompression".to_string(),
            details: format!("Failed to decompress gzip data: {}", e),
        }
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
    use flate2::{write::GzEncoder, Compression};
    use std::io::Write;
    use crate::utils::decimal_to_bcd;
    use super::*;

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
}
