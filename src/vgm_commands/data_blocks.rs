//! Data Block Processing Module
//!
//! Handles parsing and decompression of VGM data blocks including streaming data,
//! ROM dumps, and RAM writes for various sound chips.

use super::compression::{decompress_bit_packing, decompress_dpcm};
use crate::errors::{VgmError, VgmResult};
use bytes::{Buf, Bytes};
use serde::{Deserialize, Serialize};

/// Compression types for compressed data blocks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompressionType {
    BitPacking {
        bits_decompressed: u8,
        bits_compressed: u8,
        sub_type: u8, // 00=copy, 01=shift left, 02=use table
        add_value: u16,
    },
    DPCM {
        bits_decompressed: u8,
        bits_compressed: u8,
        start_value: u16,
    },
}

/// Data block content based on block type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataBlockContent {
    // Uncompressed streaming data (0x00-0x3F)
    UncompressedStream {
        chip_type: StreamChipType,
        data: Vec<u8>,
    },

    // Compressed streaming data (0x40-0x7E)
    CompressedStream {
        chip_type: StreamChipType,
        compression: CompressionType,
        uncompressed_size: u32,
        data: Vec<u8>,
    },

    // Decompression table (0x7F)
    DecompressionTable {
        compression_type: u8,
        sub_type: u8,
        bits_decompressed: u8,
        bits_compressed: u8,
        value_count: u16,
        table_data: Vec<u8>,
    },

    // ROM/RAM dumps (0x80-0xBF)
    ROMDump {
        chip_type: ROMDumpChipType,
        total_size: u32,
        start_address: u32,
        data: Vec<u8>,
    },

    // RAM writes ≤64KB (0xC0-0xDF)
    RAMWriteSmall {
        chip_type: RAMWriteChipType,
        start_address: u16,
        data: Vec<u8>,
    },

    // RAM writes >64KB (0xE0-0xFF)
    RAMWriteLarge {
        chip_type: RAMWriteChipType,
        start_address: u32,
        data: Vec<u8>,
    },

    // Unknown/Reserved block type
    Unknown {
        data: Vec<u8>,
    },
}

/// Chip types for streaming data blocks (uncompressed/compressed PCM streams)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StreamChipType {
    YM2612,       // 0x00/0x40 - Yamaha YM2612 (SegaPCM streaming)
    RF5C68,       // 0x01/0x41 - Ricoh RF5C68 PCM
    RF5C164,      // 0x02/0x42 - Ricoh RF5C164 PCM
    PWM,          // 0x03/0x43 - Sega PWM
    OKIM6258,     // 0x04/0x44 - OKI MSM6258 ADPCM
    HuC6280,      // 0x05/0x45 - Hudson HuC6280 PCM
    SCSP,         // 0x06/0x46 - Yamaha SCSP PCM
    NESAPU,       // 0x07/0x47 - NES APU DPCM
    Mikey,        // 0x08/0x48 - Atari Lynx Mikey PCM
    Reserved(u8), // 0x09-0x3F - Reserved for future use
}

/// Chip types for ROM/RAM dump blocks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ROMDumpChipType {
    SegaPCM,       // 0x80 - Sega PCM ROM data
    YM2608DeltaT,  // 0x81 - Yamaha YM2608 DELTA-T ROM
    YM2610ADPCM,   // 0x82 - Yamaha YM2610 ADPCM ROM
    YM2610DeltaT,  // 0x83 - Yamaha YM2610 DELTA-T ROM
    YMF278B,       // 0x84 - Yamaha YMF278B ROM
    YMF271,        // 0x85 - Yamaha YMF271 ROM
    YMZ280B,       // 0x86 - Yamaha YMZ280B ROM
    YMF278BRAM,    // 0x87 - Yamaha YMF278B RAM data
    Y8950DeltaT,   // 0x88 - Yamaha Y8950 DELTA-T ROM
    MultiPCM,      // 0x89 - Sega MultiPCM ROM
    UPD7759,       // 0x8A - NEC uPD7759 ROM
    OKIM6295,      // 0x8B - OKI MSM6295 ROM
    K054539,       // 0x8C - Konami K054539 ROM
    C140,          // 0x8D - Namco C140 ROM
    K053260,       // 0x8E - Konami K053260 ROM
    QSound,        // 0x8F - Capcom Q-Sound ROM
    ES5505_ES5506, // 0x90 - Ensoniq ES5505/ES5506 ROM
    X1010,         // 0x91 - Seta X1-010 ROM
    C352,          // 0x92 - Namco C352 ROM
    GA20,          // 0x93 - Irem GA20 ROM
    Reserved(u8),  // 0x94-0xBF - Reserved for future use
}

/// Chip types for RAM write blocks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RAMWriteChipType {
    RF5C68,       // 0xC0/0xE0 - Ricoh RF5C68 RAM
    RF5C164,      // 0xC1 - Ricoh RF5C164 RAM
    NESAPU,       // 0xC2 - NES APU RAM
    SCSP,         // 0xE0 - Yamaha SCSP RAM (>64KB)
    ES5503,       // 0xE1 - Ensoniq ES5503 RAM (>64KB)
    Reserved(u8), // Other values - Reserved for future use
}

impl StreamChipType {
    pub fn from_block_type(block_type: u8) -> Self {
        match block_type & 0x3F {
            // Remove compression bit
            0x00 => StreamChipType::YM2612,
            0x01 => StreamChipType::RF5C68,
            0x02 => StreamChipType::RF5C164,
            0x03 => StreamChipType::PWM,
            0x04 => StreamChipType::OKIM6258,
            0x05 => StreamChipType::HuC6280,
            0x06 => StreamChipType::SCSP,
            0x07 => StreamChipType::NESAPU,
            0x08 => StreamChipType::Mikey,
            other => StreamChipType::Reserved(other),
        }
    }
}

impl ROMDumpChipType {
    pub fn from_block_type(block_type: u8) -> Self {
        match block_type {
            0x80 => ROMDumpChipType::SegaPCM,
            0x81 => ROMDumpChipType::YM2608DeltaT,
            0x82 => ROMDumpChipType::YM2610ADPCM,
            0x83 => ROMDumpChipType::YM2610DeltaT,
            0x84 => ROMDumpChipType::YMF278B,
            0x85 => ROMDumpChipType::YMF271,
            0x86 => ROMDumpChipType::YMZ280B,
            0x87 => ROMDumpChipType::YMF278BRAM,
            0x88 => ROMDumpChipType::Y8950DeltaT,
            0x89 => ROMDumpChipType::MultiPCM,
            0x8A => ROMDumpChipType::UPD7759,
            0x8B => ROMDumpChipType::OKIM6295,
            0x8C => ROMDumpChipType::K054539,
            0x8D => ROMDumpChipType::C140,
            0x8E => ROMDumpChipType::K053260,
            0x8F => ROMDumpChipType::QSound,
            0x90 => ROMDumpChipType::ES5505_ES5506,
            0x91 => ROMDumpChipType::X1010,
            0x92 => ROMDumpChipType::C352,
            0x93 => ROMDumpChipType::GA20,
            other => ROMDumpChipType::Reserved(other),
        }
    }
}

impl RAMWriteChipType {
    pub fn from_block_type(block_type: u8) -> Self {
        match block_type {
            0xC0 => RAMWriteChipType::RF5C68,
            0xC1 => RAMWriteChipType::RF5C164,
            0xC2 => RAMWriteChipType::NESAPU,
            0xE0 => RAMWriteChipType::SCSP,
            0xE1 => RAMWriteChipType::ES5503,
            other => RAMWriteChipType::Reserved(other),
        }
    }
}

impl DataBlockContent {
    pub fn parse_from_bytes(block_type: u8, data_size: u32, bytes: &mut Bytes) -> VgmResult<Self> {
        match block_type {
            // Uncompressed streaming data (0x00-0x3F)
            0x00..=0x3F => {
                let chip_type = StreamChipType::from_block_type(block_type);
                let data: Vec<u8> = (0..data_size as usize).map(|_| bytes.get_u8()).collect();
                Ok(DataBlockContent::UncompressedStream { chip_type, data })
            },

            // Compressed streaming data (0x40-0x7E)
            0x40..=0x7E => {
                let chip_type = StreamChipType::from_block_type(block_type);
                let compression_type = bytes.get_u8();
                let uncompressed_size = bytes.get_u32_le();

                let compression = match compression_type {
                    0x00 => {
                        // Bit packing
                        let bits_decompressed = bytes.get_u8();
                        let bits_compressed = bytes.get_u8();
                        let sub_type = bytes.get_u8();
                        let add_value = bytes.get_u16_le();
                        CompressionType::BitPacking {
                            bits_decompressed,
                            bits_compressed,
                            sub_type,
                            add_value,
                        }
                    },
                    0x01 => {
                        // DPCM
                        let bits_decompressed = bytes.get_u8();
                        let bits_compressed = bytes.get_u8();
                        let _reserved = bytes.get_u8(); // Must be 00
                        let start_value = bytes.get_u16_le();
                        CompressionType::DPCM {
                            bits_decompressed,
                            bits_compressed,
                            start_value,
                        }
                    },
                    _ => {
                        return Err(VgmError::InvalidDataFormat {
                            field: "compression_type".to_string(),
                            details: format!(
                                "Unknown compression type: 0x{:02X}",
                                compression_type
                            ),
                        });
                    },
                };

                let remaining_size = data_size.saturating_sub(10); // 1 + 4 + 5 bytes consumed (compression header)
                let data: Vec<u8> = (0..remaining_size as usize)
                    .map(|_| bytes.get_u8())
                    .collect();

                Ok(DataBlockContent::CompressedStream {
                    chip_type,
                    compression,
                    uncompressed_size,
                    data,
                })
            },

            // Decompression table (0x7F)
            0x7F => {
                let compression_type = bytes.get_u8();
                let sub_type = bytes.get_u8();
                let bits_decompressed = bytes.get_u8();
                let bits_compressed = bytes.get_u8();
                let value_count = bytes.get_u16_le();
                let table_size = data_size - 6; // 6 bytes consumed
                let table_data: Vec<u8> =
                    (0..table_size as usize).map(|_| bytes.get_u8()).collect();

                Ok(DataBlockContent::DecompressionTable {
                    compression_type,
                    sub_type,
                    bits_decompressed,
                    bits_compressed,
                    value_count,
                    table_data,
                })
            },

            // ROM/RAM dumps (0x80-0xBF)
            0x80..=0xBF => {
                let chip_type = ROMDumpChipType::from_block_type(block_type);
                let total_size = bytes.get_u32_le();
                let start_address = bytes.get_u32_le();
                let data_size_remaining = data_size - 8; // 8 bytes consumed
                let data: Vec<u8> = (0..data_size_remaining as usize)
                    .map(|_| bytes.get_u8())
                    .collect();

                Ok(DataBlockContent::ROMDump {
                    chip_type,
                    total_size,
                    start_address,
                    data,
                })
            },

            // RAM writes ≤64KB (0xC0-0xDF)
            0xC0..=0xDF => {
                let chip_type = RAMWriteChipType::from_block_type(block_type);
                let start_address = bytes.get_u16_le();
                let data_size_remaining = data_size - 2; // 2 bytes consumed
                let data: Vec<u8> = (0..data_size_remaining as usize)
                    .map(|_| bytes.get_u8())
                    .collect();

                Ok(DataBlockContent::RAMWriteSmall {
                    chip_type,
                    start_address,
                    data,
                })
            },

            // RAM writes >64KB (0xE0-0xFF)
            0xE0..=0xFF => {
                let chip_type = RAMWriteChipType::from_block_type(block_type);
                let start_address = bytes.get_u32_le();
                let data_size_remaining = data_size - 4; // 4 bytes consumed
                let data: Vec<u8> = (0..data_size_remaining as usize)
                    .map(|_| bytes.get_u8())
                    .collect();

                Ok(DataBlockContent::RAMWriteLarge {
                    chip_type,
                    start_address,
                    data,
                })
            },
        }
    }

    /// Get decompressed data for compressed streams
    pub fn decompress_data(&self, decompression_table: Option<&[u8]>) -> VgmResult<Vec<u8>> {
        match self {
            DataBlockContent::UncompressedStream { data, .. } => Ok(data.clone()),
            DataBlockContent::CompressedStream {
                compression,
                uncompressed_size,
                data,
                ..
            } => match compression {
                CompressionType::BitPacking {
                    bits_decompressed,
                    bits_compressed,
                    sub_type,
                    add_value,
                } => decompress_bit_packing(
                    data,
                    *bits_compressed,
                    *bits_decompressed,
                    *sub_type,
                    *add_value,
                    *uncompressed_size,
                    decompression_table,
                ),
                CompressionType::DPCM {
                    bits_decompressed,
                    bits_compressed,
                    start_value,
                } => {
                    let table = decompression_table.ok_or_else(|| VgmError::InvalidDataFormat {
                        field: "decompression_table".to_string(),
                        details: "DPCM decompression requires a decompression table".to_string(),
                    })?;
                    decompress_dpcm(
                        data,
                        *bits_compressed,
                        *bits_decompressed,
                        *start_value,
                        *uncompressed_size,
                        table,
                    )
                },
            },
            _ => Err(VgmError::InvalidDataFormat {
                field: "data_block".to_string(),
                details: "Cannot decompress non-stream data blocks".to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_stream_chip_type_from_block_type() {
        assert_eq!(StreamChipType::from_block_type(0x00), StreamChipType::YM2612);
        assert_eq!(StreamChipType::from_block_type(0x01), StreamChipType::RF5C68);
        assert_eq!(StreamChipType::from_block_type(0x02), StreamChipType::RF5C164);
        assert_eq!(StreamChipType::from_block_type(0x03), StreamChipType::PWM);
        assert_eq!(StreamChipType::from_block_type(0x04), StreamChipType::OKIM6258);
        assert_eq!(StreamChipType::from_block_type(0x05), StreamChipType::HuC6280);
        assert_eq!(StreamChipType::from_block_type(0x06), StreamChipType::SCSP);
        assert_eq!(StreamChipType::from_block_type(0x07), StreamChipType::NESAPU);
        assert_eq!(StreamChipType::from_block_type(0x08), StreamChipType::Mikey);
        assert_eq!(StreamChipType::from_block_type(0x09), StreamChipType::Reserved(0x09));
        assert_eq!(StreamChipType::from_block_type(0x3F), StreamChipType::Reserved(0x3F));
        
        // Test with compression bit set (should be masked off)
        assert_eq!(StreamChipType::from_block_type(0x40), StreamChipType::YM2612);
        assert_eq!(StreamChipType::from_block_type(0x7F), StreamChipType::Reserved(0x3F));
    }

    #[test]
    fn test_rom_dump_chip_type_from_block_type() {
        assert_eq!(ROMDumpChipType::from_block_type(0x80), ROMDumpChipType::SegaPCM);
        assert_eq!(ROMDumpChipType::from_block_type(0x81), ROMDumpChipType::YM2608DeltaT);
        assert_eq!(ROMDumpChipType::from_block_type(0x82), ROMDumpChipType::YM2610ADPCM);
        assert_eq!(ROMDumpChipType::from_block_type(0x83), ROMDumpChipType::YM2610DeltaT);
        assert_eq!(ROMDumpChipType::from_block_type(0x84), ROMDumpChipType::YMF278B);
        assert_eq!(ROMDumpChipType::from_block_type(0x85), ROMDumpChipType::YMF271);
        assert_eq!(ROMDumpChipType::from_block_type(0x86), ROMDumpChipType::YMZ280B);
        assert_eq!(ROMDumpChipType::from_block_type(0x87), ROMDumpChipType::YMF278BRAM);
        assert_eq!(ROMDumpChipType::from_block_type(0x88), ROMDumpChipType::Y8950DeltaT);
        assert_eq!(ROMDumpChipType::from_block_type(0x89), ROMDumpChipType::MultiPCM);
        assert_eq!(ROMDumpChipType::from_block_type(0x8A), ROMDumpChipType::UPD7759);
        assert_eq!(ROMDumpChipType::from_block_type(0x8B), ROMDumpChipType::OKIM6295);
        assert_eq!(ROMDumpChipType::from_block_type(0x8C), ROMDumpChipType::K054539);
        assert_eq!(ROMDumpChipType::from_block_type(0x8D), ROMDumpChipType::C140);
        assert_eq!(ROMDumpChipType::from_block_type(0x8E), ROMDumpChipType::K053260);
        assert_eq!(ROMDumpChipType::from_block_type(0x8F), ROMDumpChipType::QSound);
        assert_eq!(ROMDumpChipType::from_block_type(0x90), ROMDumpChipType::ES5505_ES5506);
        assert_eq!(ROMDumpChipType::from_block_type(0x91), ROMDumpChipType::X1010);
        assert_eq!(ROMDumpChipType::from_block_type(0x92), ROMDumpChipType::C352);
        assert_eq!(ROMDumpChipType::from_block_type(0x93), ROMDumpChipType::GA20);
        assert_eq!(ROMDumpChipType::from_block_type(0x94), ROMDumpChipType::Reserved(0x94));
        assert_eq!(ROMDumpChipType::from_block_type(0xBF), ROMDumpChipType::Reserved(0xBF));
    }

    #[test]
    fn test_ram_write_chip_type_from_block_type() {
        assert_eq!(RAMWriteChipType::from_block_type(0xC0), RAMWriteChipType::RF5C68);
        assert_eq!(RAMWriteChipType::from_block_type(0xC1), RAMWriteChipType::RF5C164);
        assert_eq!(RAMWriteChipType::from_block_type(0xC2), RAMWriteChipType::NESAPU);
        assert_eq!(RAMWriteChipType::from_block_type(0xE0), RAMWriteChipType::SCSP);
        assert_eq!(RAMWriteChipType::from_block_type(0xE1), RAMWriteChipType::ES5503);
        assert_eq!(RAMWriteChipType::from_block_type(0xC3), RAMWriteChipType::Reserved(0xC3));
        assert_eq!(RAMWriteChipType::from_block_type(0xDF), RAMWriteChipType::Reserved(0xDF));
        assert_eq!(RAMWriteChipType::from_block_type(0xE2), RAMWriteChipType::Reserved(0xE2));
        assert_eq!(RAMWriteChipType::from_block_type(0xFF), RAMWriteChipType::Reserved(0xFF));
    }

    #[test]
    fn test_parse_uncompressed_stream() {
        let mut data = Bytes::from(vec![0x01, 0x02, 0x03, 0x04]);
        let result = DataBlockContent::parse_from_bytes(0x00, 4, &mut data).unwrap();
        
        match result {
            DataBlockContent::UncompressedStream { chip_type, data } => {
                assert_eq!(chip_type, StreamChipType::YM2612);
                assert_eq!(data, vec![0x01, 0x02, 0x03, 0x04]);
            },
            _ => panic!("Expected UncompressedStream"),
        }
    }

    #[test]
    fn test_parse_compressed_stream_bit_packing() {
        let mut bytes = Vec::new();
        bytes.push(0x00); // Bit packing compression type
        bytes.extend_from_slice(&1000u32.to_le_bytes()); // Uncompressed size
        bytes.push(16); // bits_decompressed
        bytes.push(8);  // bits_compressed
        bytes.push(0); // sub_type
        bytes.extend_from_slice(&100u16.to_le_bytes()); // add_value
        bytes.extend_from_slice(&[0xAA, 0xBB, 0xCC]); // compressed data
        
        let mut data = Bytes::from(bytes);
        let result = DataBlockContent::parse_from_bytes(0x40, 13, &mut data).unwrap();
        
        match result {
            DataBlockContent::CompressedStream { chip_type, compression, uncompressed_size, data } => {
                assert_eq!(chip_type, StreamChipType::YM2612);
                assert_eq!(uncompressed_size, 1000);
                assert_eq!(data, vec![0xAA, 0xBB, 0xCC]);
                match compression {
                    CompressionType::BitPacking { bits_decompressed, bits_compressed, sub_type, add_value } => {
                        assert_eq!(bits_decompressed, 16);
                        assert_eq!(bits_compressed, 8);
                        assert_eq!(sub_type, 0);
                        assert_eq!(add_value, 100);
                    },
                    _ => panic!("Expected BitPacking compression"),
                }
            },
            _ => panic!("Expected CompressedStream"),
        }
    }

    #[test]
    fn test_parse_compressed_stream_dpcm() {
        let mut bytes = Vec::new();
        bytes.push(0x01); // DPCM compression type
        bytes.extend_from_slice(&2000u32.to_le_bytes()); // Uncompressed size
        bytes.push(8);  // bits_decompressed
        bytes.push(4);  // bits_compressed
        bytes.push(0x00); // reserved byte
        bytes.extend_from_slice(&256u16.to_le_bytes()); // start_value
        bytes.extend_from_slice(&[0x11, 0x22]); // compressed data
        
        let mut data = Bytes::from(bytes);
        let result = DataBlockContent::parse_from_bytes(0x41, 12, &mut data).unwrap();
        
        match result {
            DataBlockContent::CompressedStream { chip_type, compression, uncompressed_size, data } => {
                assert_eq!(chip_type, StreamChipType::RF5C68);
                assert_eq!(uncompressed_size, 2000);
                assert_eq!(data, vec![0x11, 0x22]);
                match compression {
                    CompressionType::DPCM { bits_decompressed, bits_compressed, start_value } => {
                        assert_eq!(bits_decompressed, 8);
                        assert_eq!(bits_compressed, 4);
                        assert_eq!(start_value, 256);
                    },
                    _ => panic!("Expected DPCM compression"),
                }
            },
            _ => panic!("Expected CompressedStream"),
        }
    }

    #[test]
    fn test_parse_compressed_stream_invalid_compression() {
        let mut bytes = Vec::new();
        bytes.push(0xFF); // Invalid compression type
        bytes.extend_from_slice(&1000u32.to_le_bytes()); // Uncompressed size
        bytes.extend_from_slice(&[0; 8]); // Padding
        
        let mut data = Bytes::from(bytes);
        let result = DataBlockContent::parse_from_bytes(0x40, 13, &mut data);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::InvalidDataFormat { field, details } => {
                assert_eq!(field, "compression_type");
                assert!(details.contains("Unknown compression type: 0xFF"));
            },
            _ => panic!("Expected InvalidDataFormat error"),
        }
    }

    #[test]
    fn test_parse_decompression_table() {
        let mut bytes = Vec::new();
        bytes.push(0x00); // compression_type
        bytes.push(0x01); // sub_type
        bytes.push(16);   // bits_decompressed
        bytes.push(8);    // bits_compressed
        bytes.extend_from_slice(&10u16.to_le_bytes()); // value_count
        bytes.extend_from_slice(&[0x00, 0x01, 0x02, 0x03]); // table_data
        
        let mut data = Bytes::from(bytes);
        let result = DataBlockContent::parse_from_bytes(0x7F, 10, &mut data).unwrap();
        
        match result {
            DataBlockContent::DecompressionTable { 
                compression_type, sub_type, bits_decompressed, bits_compressed, value_count, table_data 
            } => {
                assert_eq!(compression_type, 0x00);
                assert_eq!(sub_type, 0x01);
                assert_eq!(bits_decompressed, 16);
                assert_eq!(bits_compressed, 8);
                assert_eq!(value_count, 10);
                assert_eq!(table_data, vec![0x00, 0x01, 0x02, 0x03]);
            },
            _ => panic!("Expected DecompressionTable"),
        }
    }

    #[test]
    fn test_parse_rom_dump() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0x1000u32.to_le_bytes()); // total_size
        bytes.extend_from_slice(&0x8000u32.to_le_bytes()); // start_address
        bytes.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]); // data
        
        let mut data = Bytes::from(bytes);
        let result = DataBlockContent::parse_from_bytes(0x80, 12, &mut data).unwrap();
        
        match result {
            DataBlockContent::ROMDump { chip_type, total_size, start_address, data } => {
                assert_eq!(chip_type, ROMDumpChipType::SegaPCM);
                assert_eq!(total_size, 0x1000);
                assert_eq!(start_address, 0x8000);
                assert_eq!(data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
            },
            _ => panic!("Expected ROMDump"),
        }
    }

    #[test]
    fn test_parse_ram_write_small() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0x1000u16.to_le_bytes()); // start_address
        bytes.extend_from_slice(&[0xCA, 0xFE, 0xBA, 0xBE]); // data
        
        let mut data = Bytes::from(bytes);
        let result = DataBlockContent::parse_from_bytes(0xC0, 6, &mut data).unwrap();
        
        match result {
            DataBlockContent::RAMWriteSmall { chip_type, start_address, data } => {
                assert_eq!(chip_type, RAMWriteChipType::RF5C68);
                assert_eq!(start_address, 0x1000);
                assert_eq!(data, vec![0xCA, 0xFE, 0xBA, 0xBE]);
            },
            _ => panic!("Expected RAMWriteSmall"),
        }
    }

    #[test]
    fn test_parse_ram_write_large() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0x20000u32.to_le_bytes()); // start_address
        bytes.extend_from_slice(&[0x12, 0x34, 0x56, 0x78]); // data
        
        let mut data = Bytes::from(bytes);
        let result = DataBlockContent::parse_from_bytes(0xE0, 8, &mut data).unwrap();
        
        match result {
            DataBlockContent::RAMWriteLarge { chip_type, start_address, data } => {
                assert_eq!(chip_type, RAMWriteChipType::SCSP);
                assert_eq!(start_address, 0x20000);
                assert_eq!(data, vec![0x12, 0x34, 0x56, 0x78]);
            },
            _ => panic!("Expected RAMWriteLarge"),
        }
    }

    #[test]
    fn test_decompress_uncompressed_stream() {
        let content = DataBlockContent::UncompressedStream {
            chip_type: StreamChipType::YM2612,
            data: vec![0x01, 0x02, 0x03, 0x04],
        };
        
        let result = content.decompress_data(None).unwrap();
        assert_eq!(result, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_decompress_non_stream_data_block() {
        let content = DataBlockContent::ROMDump {
            chip_type: ROMDumpChipType::SegaPCM,
            total_size: 0x1000,
            start_address: 0x8000,
            data: vec![0x01, 0x02, 0x03, 0x04],
        };
        
        let result = content.decompress_data(None);
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::InvalidDataFormat { field, details } => {
                assert_eq!(field, "data_block");
                assert!(details.contains("Cannot decompress non-stream data blocks"));
            },
            _ => panic!("Expected InvalidDataFormat error"),
        }
    }

    #[test]
    fn test_decompress_dpcm_without_table() {
        let content = DataBlockContent::CompressedStream {
            chip_type: StreamChipType::YM2612,
            compression: CompressionType::DPCM {
                bits_decompressed: 8,
                bits_compressed: 4,
                start_value: 128,
            },
            uncompressed_size: 100,
            data: vec![0x12, 0x34],
        };
        
        let result = content.decompress_data(None);
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::InvalidDataFormat { field, details } => {
                assert_eq!(field, "decompression_table");
                assert!(details.contains("DPCM decompression requires a decompression table"));
            },
            _ => panic!("Expected InvalidDataFormat error"),
        }
    }

    #[test]
    fn test_parse_zero_size_data_block() {
        let mut data = Bytes::from(vec![]);
        let result = DataBlockContent::parse_from_bytes(0x00, 0, &mut data).unwrap();
        
        match result {
            DataBlockContent::UncompressedStream { chip_type, data } => {
                assert_eq!(chip_type, StreamChipType::YM2612);
                assert!(data.is_empty());
            },
            _ => panic!("Expected UncompressedStream"),
        }
    }

    #[test]
    fn test_parse_edge_case_block_types() {
        // Test highest uncompressed stream type
        let mut data = Bytes::from(vec![0x01, 0x02]);
        let result = DataBlockContent::parse_from_bytes(0x3F, 2, &mut data).unwrap();
        match result {
            DataBlockContent::UncompressedStream { chip_type, .. } => {
                assert_eq!(chip_type, StreamChipType::Reserved(0x3F));
            },
            _ => panic!("Expected UncompressedStream"),
        }
        
        // Test lowest ROM dump type
        let mut data = Bytes::from(vec![0x00, 0x10, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]);
        let result = DataBlockContent::parse_from_bytes(0x80, 12, &mut data).unwrap();
        match result {
            DataBlockContent::ROMDump { chip_type, .. } => {
                assert_eq!(chip_type, ROMDumpChipType::SegaPCM);
            },
            _ => panic!("Expected ROMDump"),
        }
    }
}
