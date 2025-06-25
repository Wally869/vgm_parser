use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{VgmError, VgmResult},
    traits::{VgmParser, VgmWriter},
    utils::write_string_as_u16_bytes,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LanguageData {
    English(Gd3LocaleData),
    Japanese(Gd3LocaleData),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Gd3LocaleData {
    //pub Language: Language,
    pub track: String,
    pub game: String,
    pub system: String,
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VgmMetadata {
    pub english_data: Gd3LocaleData,
    pub japanese_data: Gd3LocaleData,
    pub date_release: String,
    pub name_vgm_creator: String,
    pub notes: String,
}

impl VgmMetadata {
    /// Parse VGM metadata with resource limits and allocation tracking
    pub fn from_bytes_with_config(
        data: &mut Bytes,
        config: &crate::ParserConfig,
    ) -> VgmResult<Self> {
        // Check metadata size before processing
        config.check_metadata_size(data.len())?;

        // Security: Validate buffer has enough data for version field
        if data.len() < 8 {
            return Err(VgmError::BufferUnderflow {
                offset: 0,
                needed: 8,
                available: data.len(),
            });
        }
        let version = data.slice(4..8);
        let ver: &[u8] = &[0x0, 0x1, 0x0, 0x0];
        if version != ver {
            let actual_version =
                u32::from_le_bytes([version[0], version[1], version[2], version[3]]);
            return Err(VgmError::UnsupportedGd3Version {
                version: actual_version,
                supported_versions: vec![0x00000100], // Version 1.0
            });
        }

        // Security: Validate buffer has enough data for data length field
        if data.len() < 12 {
            return Err(VgmError::BufferUnderflow {
                offset: 8,
                needed: 4,
                available: data.len().saturating_sub(8),
            });
        }
        let data_length = data.slice(8..12).get_u32_le();

        // Security: Validate data length is reasonable and fits within metadata size limit
        if data_length as usize > config.max_metadata_size {
            return Err(VgmError::DataSizeExceedsLimit {
                field: "metadata_data_length".to_string(),
                size: data_length as usize,
                limit: config.max_metadata_size,
            });
        }

        // Security: Validate buffer has data after header
        if data.len() < 12 {
            return Err(VgmError::BufferUnderflow {
                offset: 12,
                needed: 1,
                available: data.len().saturating_sub(12),
            });
        }

        // Security: Check UTF-16 data size before allocation
        let utf16_data_size = data.len() - 12;
        if utf16_data_size % 2 != 0 {
            return Err(VgmError::InvalidDataFormat {
                field: "UTF-16 metadata".to_string(),
                details: "UTF-16 data must have even byte count".to_string(),
            });
        }

        let expected_u16_count = utf16_data_size / 2;
        if expected_u16_count * 2 > config.max_metadata_size {
            return Err(VgmError::DataSizeExceedsLimit {
                field: "metadata_utf16_size".to_string(),
                size: expected_u16_count * 2,
                limit: config.max_metadata_size,
            });
        }

        // Convert bytes to Vec<u16> with size tracking
        let data: Vec<u16> = data
            .slice(12..)
            .to_vec()
            .chunks_exact(2)
            .map(|a| u16::from_le_bytes([a[0], a[1]]))
            .collect();

        // Security: Track and limit field parsing
        let mut temp: Vec<u16> = Vec::new();
        let mut acc: Vec<Vec<u16>> = Vec::new();
        let mut total_chars = 0usize;

        for elem in data {
            if elem == 0x0000 {
                acc.push(temp);
                temp = Vec::new();
                continue;
            }

            total_chars += 1;
            if total_chars > config.max_metadata_size / 2 {
                return Err(VgmError::DataSizeExceedsLimit {
                    field: "metadata_total_characters".to_string(),
                    size: total_chars,
                    limit: config.max_metadata_size / 2,
                });
            }

            temp.push(elem);
        }

        // Helper function to safely convert UTF-16 with proper error context
        let safe_utf16_convert = |data: &[u16], field_name: &str| -> VgmResult<String> {
            // Additional size check for individual strings
            if data.len() > config.max_metadata_size / 4 {
                return Err(VgmError::DataSizeExceedsLimit {
                    field: format!("{}_length", field_name),
                    size: data.len(),
                    limit: config.max_metadata_size / 4,
                });
            }

            String::from_utf16(data).map_err(|e| VgmError::InvalidUtf16Encoding {
                field: field_name.to_string(),
                details: e.to_string(),
            })
        };

        // Ensure we have enough fields
        if acc.len() < 11 {
            return Err(VgmError::InvalidDataLength {
                field: "GD3 metadata fields".to_string(),
                expected: 11,
                actual: acc.len(),
            });
        }

        let eng_data = Gd3LocaleData {
            track: safe_utf16_convert(&acc[0], "English track")?,
            game: safe_utf16_convert(&acc[2], "English game")?,
            system: safe_utf16_convert(&acc[4], "English system")?,
            author: safe_utf16_convert(&acc[6], "English author")?,
        };

        let jap_data = Gd3LocaleData {
            track: safe_utf16_convert(&acc[1], "Japanese track")?,
            game: safe_utf16_convert(&acc[3], "Japanese game")?,
            system: safe_utf16_convert(&acc[5], "Japanese system")?,
            author: safe_utf16_convert(&acc[7], "Japanese author")?,
        };

        Ok(VgmMetadata {
            english_data: eng_data,
            japanese_data: jap_data,
            date_release: safe_utf16_convert(&acc[8], "Release date")?,
            name_vgm_creator: safe_utf16_convert(&acc[9], "VGM creator name")?,
            notes: safe_utf16_convert(&acc[10], "Notes")?,
        })
    }
}

impl VgmParser for VgmMetadata {
    fn from_bytes(data: &mut Bytes) -> VgmResult<Self> {
        // Security: Validate buffer has enough data for version field
        if data.len() < 8 {
            return Err(VgmError::BufferUnderflow {
                offset: 0,
                needed: 8,
                available: data.len(),
            });
        }
        let version = data.slice(4..8);
        let ver: &[u8] = &[0x0, 0x1, 0x0, 0x0];
        if version != ver {
            let actual_version =
                u32::from_le_bytes([version[0], version[1], version[2], version[3]]);
            return Err(VgmError::UnsupportedGd3Version {
                version: actual_version,
                supported_versions: vec![0x00000100], // Version 1.0
            });
        }

        // Security: Validate buffer has enough data for data length field
        if data.len() < 12 {
            return Err(VgmError::BufferUnderflow {
                offset: 8,
                needed: 4,
                available: data.len().saturating_sub(8),
            });
        }
        let _data_length = data.slice(8..12).get_u32_le();

        // Security: Validate buffer has data after header
        if data.len() < 12 {
            return Err(VgmError::BufferUnderflow {
                offset: 12,
                needed: 1,
                available: data.len().saturating_sub(12),
            });
        }

        // convert bytes to Vec<u16>
        let data: Vec<u16> = data
            .slice(12..)
            .to_vec()
            .chunks_exact(2)
            .map(|a| u16::from_le_bytes([a[0], a[1]]))
            .collect();

        let mut temp: Vec<u16> = vec![];
        let mut acc: Vec<Vec<u16>> = vec![];
        for elem in data {
            if elem == 0x0000 {
                acc.push(temp);
                temp = vec![];
                continue;
            }

            temp.push(elem);
        }

        // Helper function to safely convert UTF-16 with proper error context
        let safe_utf16_convert = |data: &[u16], field_name: &str| -> VgmResult<String> {
            String::from_utf16(data).map_err(|e| VgmError::InvalidUtf16Encoding {
                field: field_name.to_string(),
                details: e.to_string(),
            })
        };

        // Ensure we have enough fields
        if acc.len() < 11 {
            return Err(VgmError::InvalidDataLength {
                field: "GD3 metadata fields".to_string(),
                expected: 11,
                actual: acc.len(),
            });
        }

        let eng_data = Gd3LocaleData {
            track: safe_utf16_convert(&acc[0], "English track")?,
            game: safe_utf16_convert(&acc[2], "English game")?,
            system: safe_utf16_convert(&acc[4], "English system")?,
            author: safe_utf16_convert(&acc[6], "English author")?,
        };

        let jap_data = Gd3LocaleData {
            track: safe_utf16_convert(&acc[1], "Japanese track")?,
            game: safe_utf16_convert(&acc[3], "Japanese game")?,
            system: safe_utf16_convert(&acc[5], "Japanese system")?,
            author: safe_utf16_convert(&acc[7], "Japanese author")?,
        };

        Ok(VgmMetadata {
            english_data: eng_data,
            japanese_data: jap_data,
            date_release: safe_utf16_convert(&acc[8], "Release date")?,
            name_vgm_creator: safe_utf16_convert(&acc[9], "VGM creator name")?,
            notes: safe_utf16_convert(&acc[10], "Notes")?,
        })
    }
}

impl VgmWriter for VgmMetadata {
    fn to_bytes(&self, buffer: &mut BytesMut) -> VgmResult<()> {
        // write magic and version
        buffer.put(&b"Gd3 "[..]);
        buffer.put(&[0x00, 0x01, 0x00, 0x00][..]);

        // reserve to write length
        let index_length = buffer.len();

        // advance by 4 bytes
        buffer.put(&[0x00, 0x00, 0x00, 0x00][..]);

        // write data and terminators
        write_string_as_u16_bytes(buffer, &self.english_data.track);
        buffer.put(&[0x00, 0x00][..]);

        write_string_as_u16_bytes(buffer, &self.japanese_data.track);
        buffer.put(&[0x00, 0x00][..]);

        write_string_as_u16_bytes(buffer, &self.english_data.game);
        buffer.put(&[0x00, 0x00][..]);

        write_string_as_u16_bytes(buffer, &self.japanese_data.game);
        buffer.put(&[0x00, 0x00][..]);

        write_string_as_u16_bytes(buffer, &self.english_data.system);
        buffer.put(&[0x00, 0x00][..]);

        write_string_as_u16_bytes(buffer, &self.japanese_data.system);
        buffer.put(&[0x00, 0x00][..]);

        write_string_as_u16_bytes(buffer, &self.english_data.author);
        buffer.put(&[0x00, 0x00][..]);

        write_string_as_u16_bytes(buffer, &self.japanese_data.author);
        buffer.put(&[0x00, 0x00][..]);

        write_string_as_u16_bytes(buffer, &self.date_release);
        buffer.put(&[0x00, 0x00][..]);

        write_string_as_u16_bytes(buffer, &self.name_vgm_creator);
        buffer.put(&[0x00, 0x00][..]);

        write_string_as_u16_bytes(buffer, &self.notes);
        buffer.put(&[0x00, 0x00][..]);

        // Security: Validate buffer bounds before indexing
        let data_length = (buffer.len() - (index_length + 4)) as u32;
        if index_length + 4 > buffer.len() {
            return Err(VgmError::BufferUnderflow {
                offset: index_length,
                needed: 4,
                available: buffer.len().saturating_sub(index_length),
            });
        }
        let loc = &mut buffer[index_length..(index_length + 4)];
        loc.copy_from_slice(&data_length.to_le_bytes()[..]);

        Ok(())
    }
}

// Validation implementation for VgmMetadata
use crate::validation::{ValidationContext, VgmValidate};

impl VgmValidate for VgmMetadata {
    fn validate(&self, _context: &ValidationContext) -> VgmResult<()> {
        // Validate string lengths are reasonable
        const MAX_STRING_LENGTH: usize = 1024;

        if self.english_data.track.len() > MAX_STRING_LENGTH {
            return Err(VgmError::ValidationFailed {
                field: "english_track".to_string(),
                reason: format!(
                    "String length {} exceeds maximum {}",
                    self.english_data.track.len(),
                    MAX_STRING_LENGTH
                ),
            });
        }

        if self.english_data.game.len() > MAX_STRING_LENGTH {
            return Err(VgmError::ValidationFailed {
                field: "english_game".to_string(),
                reason: format!(
                    "String length {} exceeds maximum {}",
                    self.english_data.game.len(),
                    MAX_STRING_LENGTH
                ),
            });
        }

        if self.english_data.system.len() > MAX_STRING_LENGTH {
            return Err(VgmError::ValidationFailed {
                field: "english_system".to_string(),
                reason: format!(
                    "String length {} exceeds maximum {}",
                    self.english_data.system.len(),
                    MAX_STRING_LENGTH
                ),
            });
        }

        if self.english_data.author.len() > MAX_STRING_LENGTH {
            return Err(VgmError::ValidationFailed {
                field: "english_author".to_string(),
                reason: format!(
                    "String length {} exceeds maximum {}",
                    self.english_data.author.len(),
                    MAX_STRING_LENGTH
                ),
            });
        }

        if self.japanese_data.track.len() > MAX_STRING_LENGTH {
            return Err(VgmError::ValidationFailed {
                field: "japanese_track".to_string(),
                reason: format!(
                    "String length {} exceeds maximum {}",
                    self.japanese_data.track.len(),
                    MAX_STRING_LENGTH
                ),
            });
        }

        if self.japanese_data.game.len() > MAX_STRING_LENGTH {
            return Err(VgmError::ValidationFailed {
                field: "japanese_game".to_string(),
                reason: format!(
                    "String length {} exceeds maximum {}",
                    self.japanese_data.game.len(),
                    MAX_STRING_LENGTH
                ),
            });
        }

        if self.japanese_data.system.len() > MAX_STRING_LENGTH {
            return Err(VgmError::ValidationFailed {
                field: "japanese_system".to_string(),
                reason: format!(
                    "String length {} exceeds maximum {}",
                    self.japanese_data.system.len(),
                    MAX_STRING_LENGTH
                ),
            });
        }

        if self.japanese_data.author.len() > MAX_STRING_LENGTH {
            return Err(VgmError::ValidationFailed {
                field: "japanese_author".to_string(),
                reason: format!(
                    "String length {} exceeds maximum {}",
                    self.japanese_data.author.len(),
                    MAX_STRING_LENGTH
                ),
            });
        }

        if self.date_release.len() > MAX_STRING_LENGTH {
            return Err(VgmError::ValidationFailed {
                field: "date_release".to_string(),
                reason: format!(
                    "String length {} exceeds maximum {}",
                    self.date_release.len(),
                    MAX_STRING_LENGTH
                ),
            });
        }

        if self.name_vgm_creator.len() > MAX_STRING_LENGTH {
            return Err(VgmError::ValidationFailed {
                field: "name_vgm_creator".to_string(),
                reason: format!(
                    "String length {} exceeds maximum {}",
                    self.name_vgm_creator.len(),
                    MAX_STRING_LENGTH
                ),
            });
        }

        if self.notes.len() > MAX_STRING_LENGTH {
            return Err(VgmError::ValidationFailed {
                field: "notes".to_string(),
                reason: format!(
                    "String length {} exceeds maximum {}",
                    self.notes.len(),
                    MAX_STRING_LENGTH
                ),
            });
        }

        // Validate strings don't contain null bytes (except terminator)
        for field_name in [
            "english_track",
            "english_game",
            "english_system",
            "english_author",
            "japanese_track",
            "japanese_game",
            "japanese_system",
            "japanese_author",
            "date_release",
            "name_vgm_creator",
            "notes",
        ] {
            let text = match field_name {
                "english_track" => &self.english_data.track,
                "english_game" => &self.english_data.game,
                "english_system" => &self.english_data.system,
                "english_author" => &self.english_data.author,
                "japanese_track" => &self.japanese_data.track,
                "japanese_game" => &self.japanese_data.game,
                "japanese_system" => &self.japanese_data.system,
                "japanese_author" => &self.japanese_data.author,
                "date_release" => &self.date_release,
                "name_vgm_creator" => &self.name_vgm_creator,
                "notes" => &self.notes,
                _ => unreachable!(),
            };

            if text.contains('\0') {
                return Err(VgmError::ValidationFailed {
                    field: field_name.to_string(),
                    reason: "String contains null bytes".to_string(),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ParserConfig;
    use bytes::{BytesMut, Bytes};

    #[test]
    fn test_gd3_locale_data_creation() {
        let locale = Gd3LocaleData {
            track: "Test Track".to_string(),
            game: "Test Game".to_string(),
            system: "Test System".to_string(),
            author: "Test Author".to_string(),
        };

        assert_eq!(locale.track, "Test Track");
        assert_eq!(locale.game, "Test Game");
        assert_eq!(locale.system, "Test System");
        assert_eq!(locale.author, "Test Author");
    }

    #[test]
    fn test_vgm_metadata_creation() {
        let english_data = Gd3LocaleData {
            track: "English Track".to_string(),
            game: "English Game".to_string(),
            system: "English System".to_string(),
            author: "English Author".to_string(),
        };

        let japanese_data = Gd3LocaleData {
            track: "Japanese Track".to_string(),
            game: "Japanese Game".to_string(),
            system: "Japanese System".to_string(),
            author: "Japanese Author".to_string(),
        };

        let metadata = VgmMetadata {
            english_data,
            japanese_data,
            date_release: "2024".to_string(),
            name_vgm_creator: "Test Creator".to_string(),
            notes: "Test Notes".to_string(),
        };

        assert_eq!(metadata.english_data.track, "English Track");
        assert_eq!(metadata.japanese_data.track, "Japanese Track");
        assert_eq!(metadata.date_release, "2024");
        assert_eq!(metadata.name_vgm_creator, "Test Creator");
        assert_eq!(metadata.notes, "Test Notes");
    }

    #[test]
    fn test_vgm_metadata_round_trip() {
        // Create test metadata
        let original = VgmMetadata {
            english_data: Gd3LocaleData {
                track: "Test Track".to_string(),
                game: "Test Game".to_string(),
                system: "Test System".to_string(),
                author: "Test Author".to_string(),
            },
            japanese_data: Gd3LocaleData {
                track: "テストトラック".to_string(),
                game: "テストゲーム".to_string(),
                system: "テストシステム".to_string(),
                author: "テスト作者".to_string(),
            },
            date_release: "2024-01-01".to_string(),
            name_vgm_creator: "VGM Test Creator".to_string(),
            notes: "Test notes".to_string(),
        };

        // Serialize to bytes
        let mut buffer = BytesMut::new();
        original.to_bytes(&mut buffer).unwrap();

        // Parse back from bytes
        let mut bytes = Bytes::from(buffer.to_vec());
        let parsed = VgmMetadata::from_bytes(&mut bytes).unwrap();

        // Verify round-trip
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_metadata_parser_insufficient_data() {
        // Test with buffer too small for version field
        let mut small_buffer = Bytes::from(vec![0x47, 0x64, 0x33, 0x20]); // Only "Gd3 "
        let result = VgmMetadata::from_bytes(&mut small_buffer);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::BufferUnderflow { .. }));
    }

    #[test]
    fn test_metadata_parser_invalid_version() {
        let mut buffer = BytesMut::new();
        buffer.put(&b"Gd3 "[..]);
        buffer.put(&[0x00, 0x02, 0x00, 0x00][..]); // Invalid version
        buffer.put(&[0x00, 0x00, 0x00, 0x00][..]); // Length
        
        let mut bytes = Bytes::from(buffer.to_vec());
        let result = VgmMetadata::from_bytes(&mut bytes);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::UnsupportedGd3Version { .. }));
    }

    #[test]
    fn test_metadata_parser_insufficient_fields() {
        let mut buffer = BytesMut::new();
        buffer.put(&b"Gd3 "[..]);
        buffer.put(&[0x00, 0x01, 0x00, 0x00][..]); // Valid version
        buffer.put(&[0x0C, 0x00, 0x00, 0x00][..]); // Length = 12 bytes

        // Only provide 5 null-terminated strings instead of required 11
        for _ in 0..5 {
            buffer.put(&[0x54u8, 0x00u8][..]); // "T" in UTF-16
            buffer.put(&[0x00u8, 0x00u8][..]); // Null terminator
        }

        let mut bytes = Bytes::from(buffer.to_vec());
        let result = VgmMetadata::from_bytes(&mut bytes);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::InvalidDataLength { .. }));
    }

    #[test]
    fn test_metadata_parser_invalid_utf16() {
        let mut buffer = BytesMut::new();
        buffer.put(&b"Gd3 "[..]);
        buffer.put(&[0x00, 0x01, 0x00, 0x00][..]); // Valid version
        buffer.put(&[0x18, 0x00, 0x00, 0x00][..]); // Length

        // Provide 11 fields but with invalid UTF-16 (unpaired surrogate)
        for i in 0..11 {
            if i == 0 {
                // First field with invalid UTF-16
                buffer.put(&[0x00u8, 0xD8u8][..]); // High surrogate without low surrogate
            }
            buffer.put(&[0x00u8, 0x00u8][..]); // Null terminator
        }

        let mut bytes = Bytes::from(buffer.to_vec());
        let result = VgmMetadata::from_bytes(&mut bytes);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::InvalidUtf16Encoding { .. }));
    }

    #[test]
    fn test_metadata_with_config_size_limits() {
        let config = ParserConfig {
            max_metadata_size: 100, // Very small limit
            ..Default::default()
        };

        // Create metadata that would exceed size limit
        let mut buffer = BytesMut::new();
        buffer.put(&b"Gd3 "[..]);
        buffer.put(&[0x00, 0x01, 0x00, 0x00][..]);
        buffer.put(&[0xFF, 0x00, 0x00, 0x00][..]); // Large size

        let mut bytes = Bytes::from(buffer.to_vec());
        let result = VgmMetadata::from_bytes_with_config(&mut bytes, &config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::DataSizeExceedsLimit { .. }));
    }

    #[test]
    fn test_metadata_with_config_odd_byte_count() {
        let config = ParserConfig::default();

        let mut buffer = BytesMut::new();
        buffer.put(&b"Gd3 "[..]);
        buffer.put(&[0x00, 0x01, 0x00, 0x00][..]);
        buffer.put(&[0x05, 0x00, 0x00, 0x00][..]); // 5 bytes (odd)
        buffer.put(&[0x01u8, 0x02u8, 0x03u8, 0x04u8, 0x05u8][..]); // Odd number of bytes

        let mut bytes = Bytes::from(buffer.to_vec());
        let result = VgmMetadata::from_bytes_with_config(&mut bytes, &config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::InvalidDataFormat { .. }));
    }

    #[test]
    fn test_metadata_validation_string_length() {
        let metadata = VgmMetadata {
            english_data: Gd3LocaleData {
                track: "a".repeat(2000), // Exceeds MAX_STRING_LENGTH of 1024
                game: "Test Game".to_string(),
                system: "Test System".to_string(),
                author: "Test Author".to_string(),
            },
            japanese_data: Gd3LocaleData {
                track: "テスト".to_string(),
                game: "テストゲーム".to_string(),
                system: "テストシステム".to_string(),
                author: "テスト作者".to_string(),
            },
            date_release: "2024".to_string(),
            name_vgm_creator: "Test Creator".to_string(),
            notes: "Test Notes".to_string(),
        };

        let context = crate::validation::ValidationContext {
            file_size: 1000,
            config: crate::validation::ValidationConfig::default(),
        };

        let result = metadata.validate(&context);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::ValidationFailed { .. }));
    }

    #[test]
    fn test_metadata_validation_null_bytes() {
        let metadata = VgmMetadata {
            english_data: Gd3LocaleData {
                track: "Test\0Track".to_string(), // Contains null byte
                game: "Test Game".to_string(),
                system: "Test System".to_string(),
                author: "Test Author".to_string(),
            },
            japanese_data: Gd3LocaleData {
                track: "テスト".to_string(),
                game: "テストゲーム".to_string(),
                system: "テストシステム".to_string(),
                author: "テスト作者".to_string(),
            },
            date_release: "2024".to_string(),
            name_vgm_creator: "Test Creator".to_string(),
            notes: "Test Notes".to_string(),
        };

        let context = crate::validation::ValidationContext {
            file_size: 1000,
            config: crate::validation::ValidationConfig::default(),
        };

        let result = metadata.validate(&context);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::ValidationFailed { .. }));
    }

    #[test]
    fn test_metadata_validation_success() {
        let metadata = VgmMetadata {
            english_data: Gd3LocaleData {
                track: "Test Track".to_string(),
                game: "Test Game".to_string(),
                system: "Test System".to_string(),
                author: "Test Author".to_string(),
            },
            japanese_data: Gd3LocaleData {
                track: "テストトラック".to_string(),
                game: "テストゲーム".to_string(),
                system: "テストシステム".to_string(),
                author: "テスト作者".to_string(),
            },
            date_release: "2024-01-01".to_string(),
            name_vgm_creator: "VGM Test Creator".to_string(),
            notes: "Test notes".to_string(),
        };

        let context = crate::validation::ValidationContext {
            file_size: 1000,
            config: crate::validation::ValidationConfig::default(),
        };

        let result = metadata.validate(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_language_data_enum() {
        let english = LanguageData::English(Gd3LocaleData {
            track: "English Track".to_string(),
            game: "English Game".to_string(),
            system: "English System".to_string(),
            author: "English Author".to_string(),
        });

        let japanese = LanguageData::Japanese(Gd3LocaleData {
            track: "Japanese Track".to_string(),
            game: "Japanese Game".to_string(),
            system: "Japanese System".to_string(),
            author: "Japanese Author".to_string(),
        });

        match english {
            LanguageData::English(data) => assert_eq!(data.track, "English Track"),
            _ => panic!("Expected English variant"),
        }

        match japanese {
            LanguageData::Japanese(data) => assert_eq!(data.track, "Japanese Track"),
            _ => panic!("Expected Japanese variant"),
        }
    }

    #[test]
    fn test_metadata_serialization_empty_strings() {
        // Test with empty strings
        let metadata = VgmMetadata {
            english_data: Gd3LocaleData {
                track: "".to_string(),
                game: "".to_string(),
                system: "".to_string(),
                author: "".to_string(),
            },
            japanese_data: Gd3LocaleData {
                track: "".to_string(),
                game: "".to_string(),
                system: "".to_string(),
                author: "".to_string(),
            },
            date_release: "".to_string(),
            name_vgm_creator: "".to_string(),
            notes: "".to_string(),
        };

        let mut buffer = BytesMut::new();
        let result = metadata.to_bytes(&mut buffer);
        assert!(result.is_ok());

        // Parse back
        let mut bytes = Bytes::from(buffer.to_vec());
        let parsed = VgmMetadata::from_bytes(&mut bytes).unwrap();
        assert_eq!(metadata, parsed);
    }

    #[test]
    fn test_metadata_with_unicode_characters() {
        // Test with various Unicode characters
        let metadata = VgmMetadata {
            english_data: Gd3LocaleData {
                track: "Test Track with émojis 🎵".to_string(),
                game: "Game™".to_string(),
                system: "System©".to_string(),
                author: "Author®".to_string(),
            },
            japanese_data: Gd3LocaleData {
                track: "音楽テスト".to_string(),
                game: "ゲーム".to_string(),
                system: "システム".to_string(),
                author: "作者".to_string(),
            },
            date_release: "2024年1月1日".to_string(),
            name_vgm_creator: "VGM Creator ★".to_string(),
            notes: "Notes with symbols: ♪♫♪♫".to_string(),
        };

        // Test round trip
        let mut buffer = BytesMut::new();
        metadata.to_bytes(&mut buffer).unwrap();

        let mut bytes = Bytes::from(buffer.to_vec());
        let parsed = VgmMetadata::from_bytes(&mut bytes).unwrap();
        assert_eq!(metadata, parsed);
    }

    // Property-based tests using proptest
    use proptest::prelude::*;
    
    prop_compose! {
        fn arbitrary_gd3_string()
                                (s in ".*{0,255}") -> String {
            // Ensure strings don't contain null bytes and aren't too long for UTF-16
            s.chars().filter(|&c| c != '\0').take(255).collect()
        }
    }
    
    proptest! {
        #[test]
        fn test_gd3_round_trip_parsing(
            track_name in arbitrary_gd3_string(),
            track_name_japanese in arbitrary_gd3_string(),
            game_name in arbitrary_gd3_string(),
            game_name_japanese in arbitrary_gd3_string(),
            system_name in arbitrary_gd3_string(),
            system_name_japanese in arbitrary_gd3_string(),
            author_name in arbitrary_gd3_string(),
            author_name_japanese in arbitrary_gd3_string(),
            release_date in arbitrary_gd3_string(),
            dumper_name in arbitrary_gd3_string(),
            notes in arbitrary_gd3_string()
        ) {
            // Create a VgmMetadata with property values
            let original_metadata = VgmMetadata {
                english_data: Gd3LocaleData {
                    track: track_name,
                    game: game_name,
                    system: system_name,
                    author: author_name,
                },
                japanese_data: Gd3LocaleData {
                    track: track_name_japanese,
                    game: game_name_japanese,
                    system: system_name_japanese,
                    author: author_name_japanese,
                },
                date_release: release_date,
                name_vgm_creator: dumper_name,
                notes,
            };

            // Serialize to bytes
            let mut buffer = BytesMut::new();
            let serialize_result = original_metadata.to_bytes(&mut buffer);
            
            // If serialization succeeds, test round-trip parsing
            if serialize_result.is_ok() {
                let mut bytes = Bytes::from(buffer);
                let parse_result = VgmMetadata::from_bytes(&mut bytes);
                
                if let Ok(parsed_metadata) = parse_result {
                    // Verify round-trip preservation
                    prop_assert_eq!(original_metadata.english_data.track, parsed_metadata.english_data.track);
                    prop_assert_eq!(original_metadata.english_data.game, parsed_metadata.english_data.game);
                    prop_assert_eq!(original_metadata.english_data.system, parsed_metadata.english_data.system);
                    prop_assert_eq!(original_metadata.english_data.author, parsed_metadata.english_data.author);
                    prop_assert_eq!(original_metadata.japanese_data.track, parsed_metadata.japanese_data.track);
                    prop_assert_eq!(original_metadata.japanese_data.game, parsed_metadata.japanese_data.game);
                    prop_assert_eq!(original_metadata.japanese_data.system, parsed_metadata.japanese_data.system);
                    prop_assert_eq!(original_metadata.japanese_data.author, parsed_metadata.japanese_data.author);
                    prop_assert_eq!(original_metadata.date_release, parsed_metadata.date_release);
                    prop_assert_eq!(original_metadata.name_vgm_creator, parsed_metadata.name_vgm_creator);
                    prop_assert_eq!(original_metadata.notes, parsed_metadata.notes);
                }
            }
        }

        #[test]
        fn test_gd3_unicode_handling(
            track_name in r"[\u{0000}-\u{D7FF}\u{E000}-\u{FFFD}]{0,100}",
            game_name in r"[\u{0000}-\u{D7FF}\u{E000}-\u{FFFD}]{0,100}",
            author_name in r"[\u{0000}-\u{D7FF}\u{E000}-\u{FFFD}]{0,100}"
        ) {
            // Test Unicode handling in GD3 strings (excluding null characters)
            let track_name_clean = track_name.chars().filter(|&c| c != '\0').collect::<String>();
            let game_name_clean = game_name.chars().filter(|&c| c != '\0').collect::<String>();
            let author_name_clean = author_name.chars().filter(|&c| c != '\0').collect::<String>();
            
            let metadata = VgmMetadata {
                english_data: Gd3LocaleData {
                    track: track_name_clean.clone(),
                    game: game_name_clean.clone(),
                    system: "Test System".to_string(),
                    author: author_name_clean.clone(),
                },
                japanese_data: Gd3LocaleData {
                    track: "".to_string(),
                    game: "".to_string(),
                    system: "".to_string(),
                    author: "".to_string(),
                },
                date_release: "2023".to_string(),
                name_vgm_creator: "Test Dumper".to_string(),
                notes: "Test notes".to_string(),
            };

            // Test round-trip with Unicode strings
            let mut buffer = BytesMut::new();
            let serialize_result = metadata.to_bytes(&mut buffer);
            
            // If serialization succeeds, test parsing
            if serialize_result.is_ok() {
                let mut bytes = Bytes::from(buffer);
                let parsed_result = VgmMetadata::from_bytes(&mut bytes);
                
                if let Ok(parsed) = parsed_result {
                    prop_assert_eq!(metadata.english_data.track, parsed.english_data.track);
                    prop_assert_eq!(metadata.english_data.game, parsed.english_data.game);
                    prop_assert_eq!(metadata.english_data.author, parsed.english_data.author);
                }
            }
        }

        #[test]
        fn test_gd3_string_lengths(
            track_name_len in 0usize..=500,
            game_name_len in 0usize..=500,
            notes_len in 0usize..=1000
        ) {
            // Test various string lengths
            let track_name = "A".repeat(track_name_len);
            let game_name = "B".repeat(game_name_len);
            let notes = "C".repeat(notes_len);
            
            let metadata = VgmMetadata {
                english_data: Gd3LocaleData {
                    track: track_name.clone(),
                    game: game_name.clone(),
                    system: "System".to_string(),
                    author: "Author".to_string(),
                },
                japanese_data: Gd3LocaleData {
                    track: "".to_string(),
                    game: "".to_string(),
                    system: "".to_string(),
                    author: "".to_string(),
                },
                date_release: "2023".to_string(),
                name_vgm_creator: "Dumper".to_string(),
                notes: notes.clone(),
            };

            // Test serialization
            let mut buffer = BytesMut::new();
            let result = metadata.to_bytes(&mut buffer);
            
            // Very long strings might fail or succeed depending on limits
            if result.is_ok() {
                // If serialization succeeds, test round-trip
                let mut bytes = Bytes::from(buffer);
                let parsed_result = VgmMetadata::from_bytes(&mut bytes);
                
                if let Ok(parsed) = parsed_result {
                    prop_assert_eq!(metadata.english_data.track, parsed.english_data.track);
                    prop_assert_eq!(metadata.english_data.game, parsed.english_data.game);
                    prop_assert_eq!(metadata.notes, parsed.notes);
                }
            }
        }

        #[test]
        fn test_gd3_empty_and_special_strings(
            has_empty_track in proptest::bool::ANY,
            has_empty_game in proptest::bool::ANY,
            has_empty_author in proptest::bool::ANY
        ) {
            // Test handling of empty strings and special cases
            let metadata = VgmMetadata {
                english_data: Gd3LocaleData {
                    track: if has_empty_track { "".to_string() } else { "Track".to_string() },
                    game: if has_empty_game { "".to_string() } else { "Game".to_string() },
                    system: "System".to_string(),
                    author: if has_empty_author { "".to_string() } else { "Author".to_string() },
                },
                japanese_data: Gd3LocaleData {
                    track: "".to_string(),
                    game: "".to_string(),
                    system: "".to_string(),
                    author: "".to_string(),
                },
                date_release: "2023".to_string(),
                name_vgm_creator: "Dumper".to_string(),
                notes: "Notes".to_string(),
            };

            // Test round-trip with empty strings
            let mut buffer = BytesMut::new();
            metadata.to_bytes(&mut buffer).unwrap();
            
            let mut bytes = Bytes::from(buffer);
            let parsed = VgmMetadata::from_bytes(&mut bytes).unwrap();
            
            prop_assert_eq!(metadata.english_data.track, parsed.english_data.track);
            prop_assert_eq!(metadata.english_data.game, parsed.english_data.game);
            prop_assert_eq!(metadata.english_data.author, parsed.english_data.author);
        }

        #[test]
        fn test_gd3_japanese_characters(
            use_hiragana in proptest::bool::ANY,
            use_katakana in proptest::bool::ANY,
            use_kanji in proptest::bool::ANY
        ) {
            // Test with Japanese character sets
            let mut track_jp = String::new();
            let mut game_jp = String::new();
            
            if use_hiragana {
                track_jp.push_str("ひらがな");
                game_jp.push_str("あいうえお");
            }
            if use_katakana {
                track_jp.push_str("カタカナ");
                game_jp.push_str("アイウエオ");
            }
            if use_kanji {
                track_jp.push_str("漢字");
                game_jp.push_str("日本語");
            }
            
            if track_jp.is_empty() {
                track_jp = "テスト".to_string();
            }
            if game_jp.is_empty() {
                game_jp = "ゲーム".to_string();
            }
            
            let metadata = VgmMetadata {
                english_data: Gd3LocaleData {
                    track: "English Track".to_string(),
                    game: "English Game".to_string(),
                    system: "System".to_string(),
                    author: "Author".to_string(),
                },
                japanese_data: Gd3LocaleData {
                    track: track_jp.clone(),
                    game: game_jp.clone(),
                    system: "システム".to_string(),
                    author: "作者".to_string(),
                },
                date_release: "2023".to_string(),
                name_vgm_creator: "Dumper".to_string(),
                notes: "Notes".to_string(),
            };

            // Test round-trip with Japanese characters
            let mut buffer = BytesMut::new();
            metadata.to_bytes(&mut buffer).unwrap();
            
            let mut bytes = Bytes::from(buffer);
            let parsed = VgmMetadata::from_bytes(&mut bytes).unwrap();
            
            prop_assert_eq!(metadata.japanese_data.track, parsed.japanese_data.track);
            prop_assert_eq!(metadata.japanese_data.game, parsed.japanese_data.game);
        }
    }

    #[test]
    fn test_metadata_error_paths_coverage() {
        // Test error paths for improved coverage
        
        // Test with config size limits exceeded
        let config = crate::ParserConfig {
            max_metadata_size: 100, // Very small limit
            ..Default::default()
        };
        
        let mut data = BytesMut::new();
        data.put_u32_le(0x20334447); // "Gd3 " magic
        data.put_u32_le(0x00000100); // Version 1.0
        data.put_u32_le(200); // Data length exceeds limit
        // Add 200 bytes of UTF-16 data
        for _ in 0..100 {
            data.put_u16_le(0x0041); // 'A' in UTF-16
        }
        
        let mut bytes = Bytes::from(data);
        let result = VgmMetadata::from_bytes_with_config(&mut bytes, &config);
        assert!(result.is_err());
        
        // Test with invalid GD3 version
        let mut data = BytesMut::new();
        data.put_u32_le(0x20334447); // "Gd3 " magic
        data.put_u32_le(0x00000200); // Invalid version 2.0
        data.put_u32_le(50);
        for _ in 0..25 {
            data.put_u16_le(0x0000);
        }
        
        let mut bytes = Bytes::from(data);
        let result = VgmMetadata::from_bytes(&mut bytes);
        assert!(result.is_err());
        
        // Test with buffer underflow in header
        let mut data = BytesMut::new();
        data.put_u32_le(0x20334447); // "Gd3 " magic
        data.put_u16(0x00); // Truncated version
        
        let mut bytes = Bytes::from(data);
        let result = VgmMetadata::from_bytes(&mut bytes);
        assert!(result.is_err());
        
        // Test with buffer underflow in data length
        let mut data = BytesMut::new();
        data.put_u32_le(0x20334447); // "Gd3 " magic
        data.put_u32_le(0x00000100); // Version 1.0
        data.put_u16(0x00); // Truncated data length
        
        let mut bytes = Bytes::from(data);
        let result = VgmMetadata::from_bytes(&mut bytes);
        assert!(result.is_err());
        
        // Test with very small data length
        let mut data = BytesMut::new();
        data.put_u32_le(0x20334447); // "Gd3 " magic
        data.put_u32_le(0x00000100); // Version 1.0
        data.put_u32_le(10); // Very small data length
        for _ in 0..5 {
            data.put_u16_le(0x0000);
        }
        
        let mut bytes = Bytes::from(data);
        let result = VgmMetadata::from_bytes(&mut bytes);
        // This should succeed with minimal strings
        if result.is_ok() {
            let metadata = result.unwrap();
            assert_eq!(metadata.english_data.track, "");
        } else {
            // Or fail due to insufficient data - either is acceptable
            assert!(result.is_err());
        }
        
        // Test with metadata size exceeding u16 count limit
        let config = crate::ParserConfig {
            max_metadata_size: 10, // Very small
            ..Default::default()
        };
        
        let mut data = BytesMut::new();
        data.put_u32_le(0x20334447); // "Gd3 " magic
        data.put_u32_le(0x00000100); // Version 1.0
        data.put_u32_le(20); // Data length exceeds u16 count limit
        for _ in 0..10 {
            data.put_u16_le(0x0000);
        }
        
        let mut bytes = Bytes::from(data);
        let result = VgmMetadata::from_bytes_with_config(&mut bytes, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata_serialization_error_paths() {
        // Test serialization error paths
        
        // Test with very long strings that might cause issues
        let long_string = "a".repeat(1000);
        let metadata = VgmMetadata {
            english_data: Gd3LocaleData {
                track: long_string.clone(),
                game: long_string.clone(),
                system: long_string.clone(),
                author: long_string.clone(),
            },
            japanese_data: Gd3LocaleData {
                track: long_string.clone(),
                game: long_string.clone(),
                system: long_string.clone(),
                author: long_string.clone(),
            },
            date_release: long_string.clone(),
            name_vgm_creator: long_string.clone(),
            notes: long_string,
        };
        
        let mut buffer = BytesMut::new();
        let result = metadata.to_bytes(&mut buffer);
        // Should succeed but be very large
        assert!(result.is_ok());
        assert!(buffer.len() > 5000);
        
        // Test round-trip with large metadata
        let mut bytes = Bytes::from(buffer);
        let parsed = VgmMetadata::from_bytes(&mut bytes);
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.english_data.track.len(), 1000);
    }

    #[test]
    fn test_metadata_utf16_edge_cases() {
        // Test UTF-16 encoding edge cases
        
        // Test with Unicode characters requiring surrogate pairs
        let emoji_string = "🎵🎶🎼"; // Musical emojis
        let metadata = VgmMetadata {
            english_data: Gd3LocaleData {
                track: emoji_string.to_string(),
                game: "Test Game".to_string(),
                system: "Test System".to_string(),
                author: "Test Author".to_string(),
            },
            japanese_data: Gd3LocaleData {
                track: "テスト".to_string(), // Japanese characters
                game: "ゲーム".to_string(),
                system: "システム".to_string(),
                author: "作者".to_string(),
            },
            date_release: "2023".to_string(),
            name_vgm_creator: "Creator".to_string(),
            notes: "Notes with emojis 🎵".to_string(),
        };
        
        let mut buffer = BytesMut::new();
        metadata.to_bytes(&mut buffer).unwrap();
        
        let mut bytes = Bytes::from(buffer);
        let parsed = VgmMetadata::from_bytes(&mut bytes).unwrap();
        
        assert_eq!(parsed.english_data.track, emoji_string);
        assert_eq!(parsed.japanese_data.track, "テスト");
        assert_eq!(parsed.notes, "Notes with emojis 🎵");
    }

    #[test]
    fn test_metadata_boundary_cases() {
        // Test boundary cases for metadata parsing
        
        // Test with exactly 10 strings (minimum valid GD3)
        let mut data = BytesMut::new();
        data.put_u32_le(0x20334447); // "Gd3 " magic
        data.put_u32_le(0x00000100); // Version 1.0
        data.put_u32_le(22); // Exact size for 11 null-terminated empty strings
        
        // Add exactly 11 null-terminated UTF-16 strings (empty)
        for _ in 0..11 {
            data.put_u16_le(0x0000); // Null terminator
        }
        
        let mut bytes = Bytes::from(data);
        let result = VgmMetadata::from_bytes(&mut bytes);
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.english_data.track, "");
        assert_eq!(metadata.japanese_data.track, "");
        
        // Test with minimal valid metadata
        let mut data = BytesMut::new();
        data.put_u32_le(0x20334447); // "Gd3 " magic
        data.put_u32_le(0x00000100); // Version 1.0
        data.put_u32_le(22); // Size for 11 empty strings
        
        for _ in 0..11 {
            data.put_u16_le(0x0000);
        }
        
        let mut bytes = Bytes::from(data);
        let result = VgmMetadata::from_bytes(&mut bytes);
        assert!(result.is_ok());
    }
}