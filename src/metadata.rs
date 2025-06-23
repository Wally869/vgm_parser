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

impl VgmParser for VgmMetadata {
    fn from_bytes(data: &mut Bytes) -> VgmResult<Self> {
        // validate version
        let version = data.slice(4..8); //.get_u32_le();
        let ver: &[u8] = &[0x0, 0x1, 0x0, 0x0];
        if version != ver {
            let actual_version = u32::from_le_bytes([version[0], version[1], version[2], version[3]]);
            return Err(VgmError::UnsupportedGd3Version {
                version: actual_version,
                supported_versions: vec![0x00000100], // Version 1.0
            });
        }

        let _data_length = data.slice(8..12).get_u32_le();

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

        // get length and write it
        let data_length = (buffer.len() - (index_length + 4)) as u32;
        let loc = &mut buffer[index_length..(index_length + 4)];
        loc.copy_from_slice(&data_length.to_le_bytes()[..]);
        
        Ok(())
    }
}
