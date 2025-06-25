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
