use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};

use crate::{
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
    fn from_bytes(data: &mut Bytes) -> Self {
        // validate version
        let version = data.slice(4..8); //.get_u32_le();
        let ver: &[u8] = &[0x0, 0x1, 0x0, 0x0];
        if version != ver {
            //return Err(LibError::UnsupportedGd3Version);
            panic!("Unsupported Gd3 Version");
        }

        let data_length = data.slice(8..12).get_u32_le();

        // convert bytes to Vec<u16>
        let data: Vec<u16> = data
            .slice(12..)
            .to_vec()
            .chunks_exact(2)
            .into_iter()
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

        let eng_data = Gd3LocaleData {
            track: String::from_utf16(&acc[0]).unwrap(),
            game: String::from_utf16(&acc[2]).unwrap(),
            system: String::from_utf16(&acc[4]).unwrap(),
            author: String::from_utf16(&acc[6]).unwrap(),
        };

        let jap_data = Gd3LocaleData {
            track: String::from_utf16(&acc[1]).unwrap(),
            game: String::from_utf16(&acc[3]).unwrap(),
            system: String::from_utf16(&acc[5]).unwrap(),
            author: String::from_utf16(&acc[7]).unwrap(),
        };

        return VgmMetadata {
            english_data: eng_data,
            japanese_data: jap_data,
            date_release: String::from_utf16(&acc[8]).unwrap(),
            name_vgm_creator: String::from_utf16(&acc[9]).unwrap(),
            notes: String::from_utf16(&acc[10]).unwrap(),
        };
    }
}

impl VgmWriter for VgmMetadata {
    fn to_bytes(&self, buffer: &mut BytesMut) {
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
    }
}
