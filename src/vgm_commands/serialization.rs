//! VGM Command Serialization Module
//!
//! Handles converting Commands enum variants back to their byte array representations
//! according to the VGM format specification, including dual-chip support.

use super::commands::Commands;
use super::data_blocks::{CompressionType, DataBlockContent};
use crate::errors::{VgmError, VgmResult};

impl Commands {
    pub fn to_bytes(self) -> VgmResult<Vec<u8>> {
        let bytes = match self {
            Commands::AY8910StereoMask { value } => {
                vec![0x31, value]
            },
            Commands::GameGearPSGStereo { value, chip_index } => {
                match chip_index {
                    0 => vec![0x4f, value], // First chip
                    1 => vec![0x3f, value], // Second chip
                    _ => {
                        return Err(VgmError::InvalidDataFormat {
                            field: "chip_index".to_string(),
                            details: format!(
                                "Invalid chip_index {} for GameGearPSGStereo, must be 0 or 1",
                                chip_index
                            ),
                        })
                    },
                }
            },
            Commands::PSGWrite { value, chip_index } => {
                match chip_index {
                    0 => vec![0x50, value], // First chip
                    1 => vec![0x30, value], // Second chip
                    _ => {
                        return Err(VgmError::InvalidDataFormat {
                            field: "chip_index".to_string(),
                            details: format!(
                                "Invalid chip_index {} for PSGWrite, must be 0 or 1",
                                chip_index
                            ),
                        })
                    },
                }
            },
            Commands::YM2413Write {
                register,
                value,
                chip_index,
            } => {
                let opcode = if chip_index == 0 { 0x51 } else { 0xA1 };
                vec![opcode, register, value]
            },
            Commands::YM2612Port0Write {
                register,
                value,
                chip_index,
            } => {
                let opcode = if chip_index == 0 { 0x52 } else { 0xA2 };
                vec![opcode, register, value]
            },
            Commands::YM2612Port1Write {
                register,
                value,
                chip_index,
            } => {
                let opcode = if chip_index == 0 { 0x53 } else { 0xA3 };
                vec![opcode, register, value]
            },
            Commands::YM2151Write {
                register,
                value,
                chip_index,
            } => {
                let opcode = if chip_index == 0 { 0x54 } else { 0xA4 };
                vec![opcode, register, value]
            },
            Commands::YM2203Write {
                register,
                value,
                chip_index,
            } => {
                let opcode = if chip_index == 0 { 0x55 } else { 0xA5 };
                vec![opcode, register, value]
            },
            Commands::YM2608Port0Write {
                register,
                value,
                chip_index,
            } => {
                let opcode = if chip_index == 0 { 0x56 } else { 0xA6 };
                vec![opcode, register, value]
            },
            Commands::YM2608Port1Write {
                register,
                value,
                chip_index,
            } => {
                let opcode = if chip_index == 0 { 0x57 } else { 0xA7 };
                vec![opcode, register, value]
            },
            Commands::YM2610Port0Write {
                register,
                value,
                chip_index,
            } => {
                let opcode = if chip_index == 0 { 0x58 } else { 0xA8 };
                vec![opcode, register, value]
            },
            Commands::YM2610Port1Write {
                register,
                value,
                chip_index,
            } => {
                let opcode = if chip_index == 0 { 0x59 } else { 0xA9 };
                vec![opcode, register, value]
            },
            Commands::YM3812Write {
                register,
                value,
                chip_index,
            } => {
                let opcode = if chip_index == 0 { 0x5A } else { 0xAA };
                vec![opcode, register, value]
            },
            Commands::YM3526Write {
                register,
                value,
                chip_index,
            } => {
                let opcode = if chip_index == 0 { 0x5B } else { 0xAB };
                vec![opcode, register, value]
            },
            Commands::Y8950Write {
                register,
                value,
                chip_index,
            } => {
                let opcode = if chip_index == 0 { 0x5C } else { 0xAC };
                vec![opcode, register, value]
            },
            Commands::YMZ280BWrite {
                register,
                value,
                chip_index,
            } => {
                let opcode = if chip_index == 0 { 0x5D } else { 0xAD };
                vec![opcode, register, value]
            },
            Commands::YMF262Port0Write {
                register,
                value,
                chip_index,
            } => {
                let opcode = if chip_index == 0 { 0x5E } else { 0xAE };
                vec![opcode, register, value]
            },
            Commands::YMF262Port1Write {
                register,
                value,
                chip_index,
            } => {
                let opcode = if chip_index == 0 { 0x5F } else { 0xAF };
                vec![opcode, register, value]
            },
            Commands::WaitNSamples { n } => {
                let temp = n.to_le_bytes();
                vec![0x61, temp[0], temp[1]]
            },
            Commands::Wait735Samples => {
                vec![0x62]
            },
            Commands::Wait882Samples => {
                vec![0x63]
            },
            Commands::EndOfSoundData => {
                vec![0x66]
            },

            Commands::DataBlock { block_type, data } => {
                // The DataBlock command format: 0x67 0x66 tt ss ss ss ss (data)
                let mut out_data: Vec<u8> = vec![0x67, 0x66, block_type];

                // Calculate the size based on the data content
                let data_size = match &data {
                    DataBlockContent::UncompressedStream { data, .. } => data.len() as u32,
                    DataBlockContent::CompressedStream { data, .. } => data.len() as u32 + 9, // +9 for compression header
                    DataBlockContent::DecompressionTable { table_data, .. } => {
                        table_data.len() as u32 + 6
                    }, // +6 for header
                    DataBlockContent::ROMDump { data, .. } => data.len() as u32 + 8, // +8 for total_size and start_address
                    DataBlockContent::RAMWriteSmall { data, .. } => data.len() as u32 + 2, // +2 for start_address
                    DataBlockContent::RAMWriteLarge { data, .. } => data.len() as u32 + 4, // +4 for start_address
                    DataBlockContent::Unknown { data } => data.len() as u32,
                };

                out_data.extend(data_size.to_le_bytes());

                // Serialize the data content
                match data {
                    DataBlockContent::UncompressedStream { data, .. } => {
                        out_data.extend(data);
                    },
                    DataBlockContent::CompressedStream {
                        compression,
                        uncompressed_size,
                        data,
                        ..
                    } => {
                        // Write compression header
                        match compression {
                            CompressionType::BitPacking {
                                bits_decompressed,
                                bits_compressed,
                                sub_type,
                                add_value,
                            } => {
                                out_data.push(0x00); // Bit packing compression type
                                out_data.extend(uncompressed_size.to_le_bytes());
                                out_data.push(bits_decompressed);
                                out_data.push(bits_compressed);
                                out_data.push(sub_type);
                                out_data.extend(add_value.to_le_bytes());
                            },
                            CompressionType::DPCM {
                                bits_decompressed,
                                bits_compressed,
                                start_value,
                            } => {
                                out_data.push(0x01); // DPCM compression type
                                out_data.extend(uncompressed_size.to_le_bytes());
                                out_data.push(bits_decompressed);
                                out_data.push(bits_compressed);
                                out_data.push(0x00); // Reserved byte
                                out_data.extend(start_value.to_le_bytes());
                            },
                        }
                        out_data.extend(data);
                    },
                    DataBlockContent::DecompressionTable {
                        compression_type,
                        sub_type,
                        bits_decompressed,
                        bits_compressed,
                        value_count,
                        table_data,
                    } => {
                        out_data.push(compression_type);
                        out_data.push(sub_type);
                        out_data.push(bits_decompressed);
                        out_data.push(bits_compressed);
                        out_data.extend(value_count.to_le_bytes());
                        out_data.extend(table_data);
                    },
                    DataBlockContent::ROMDump {
                        total_size,
                        start_address,
                        data,
                        ..
                    } => {
                        out_data.extend(total_size.to_le_bytes());
                        out_data.extend(start_address.to_le_bytes());
                        out_data.extend(data);
                    },
                    DataBlockContent::RAMWriteSmall {
                        start_address,
                        data,
                        ..
                    } => {
                        out_data.extend(start_address.to_le_bytes());
                        out_data.extend(data);
                    },
                    DataBlockContent::RAMWriteLarge {
                        start_address,
                        data,
                        ..
                    } => {
                        out_data.extend(start_address.to_le_bytes());
                        out_data.extend(data);
                    },
                    DataBlockContent::Unknown { data } => {
                        out_data.extend(data);
                    },
                }

                out_data
            },
            Commands::PCMRAMWrite {
                chip_type: _,
                read_offset: _,
                write_offset: _,
                size: _,
                data: _,
            } => {
                return Err(VgmError::FeatureNotSupported {
                    feature: "PCM RAM Write command serialization".to_string(),
                    version: 0,     // Unknown version requirement
                    min_version: 0, // Would need to research the actual VGM version requirement
                });
            },

            Commands::WaitNSamplesPlus1 { n } => vec![0x70 + n],

            Commands::YM2612Port0Address2AWriteWait { n } => vec![0x80 + n],

            // DAC Stream Control Commands (0x90-0x95)
            Commands::DACStreamSetupControl {
                stream_id,
                chip_type,
                port,
                command,
                chip_index,
            } => {
                // Dual chip support: Set bit 7 of chip_type when chip_index == 1
                let adjusted_chip_type = if chip_index == 0 {
                    chip_type & 0x7F
                } else {
                    chip_type | 0x80
                };
                vec![0x90, stream_id, adjusted_chip_type, port, command]
            },
            Commands::DACStreamSetData {
                stream_id,
                data_bank_id,
                step_size,
                step_base,
            } => {
                vec![0x91, stream_id, data_bank_id, step_size, step_base]
            },
            Commands::DACStreamSetFrequency {
                stream_id,
                frequency,
            } => {
                let freq_bytes = frequency.to_le_bytes();
                vec![
                    0x92,
                    stream_id,
                    freq_bytes[0],
                    freq_bytes[1],
                    freq_bytes[2],
                    freq_bytes[3],
                ]
            },
            Commands::DACStreamStart {
                stream_id,
                data_start_offset,
                length_mode,
                data_length,
            } => {
                let offset_bytes = data_start_offset.to_le_bytes();
                let length_bytes = data_length.to_le_bytes();
                vec![
                    0x93,
                    stream_id,
                    offset_bytes[0],
                    offset_bytes[1],
                    offset_bytes[2],
                    offset_bytes[3],
                    length_mode,
                    length_bytes[0],
                    length_bytes[1],
                    length_bytes[2],
                    length_bytes[3],
                ]
            },
            Commands::DACStreamStop { stream_id } => {
                vec![0x94, stream_id]
            },
            Commands::DACStreamStartFast {
                stream_id,
                block_id,
                flags,
            } => {
                let block_bytes = block_id.to_le_bytes();
                vec![0x95, stream_id, block_bytes[0], block_bytes[1], flags]
            },

            Commands::AY8910Write {
                register,
                value,
                chip_index,
            } => {
                // Method #2: Use bit 7 of register for chip selection (0x00-7F = chip 1, 0x80-FF = chip 2)
                let adjusted_register = if chip_index == 0 {
                    register & 0x7F
                } else {
                    register | 0x80
                };
                vec![0xA0, adjusted_register, value]
            },
            Commands::RF5C68Write { register, value } => {
                vec![0xB0, register, value]
            },
            Commands::RF5C164Write { register, value } => {
                vec![0xB1, register, value]
            },
            Commands::PWMWrite { register, value } => {
                let temp = value.to_le_bytes();
                vec![0xB2, register, temp[0], temp[1]]
            },
            Commands::GameBoyDMGWrite {
                register,
                value,
                chip_index,
            } => {
                // Method #2: Use bit 7 of register for chip selection (0x00-7F = chip 1, 0x80-FF = chip 2)
                let adjusted_register = if chip_index == 0 {
                    register & 0x7F
                } else {
                    register | 0x80
                };
                vec![0xB3, adjusted_register, value]
            },
            Commands::NESAPUWrite {
                register,
                value,
                chip_index,
            } => {
                // Method #2: Use bit 7 of register for chip selection (0x00-7F = chip 1, 0x80-FF = chip 2)
                let adjusted_register = if chip_index == 0 {
                    register & 0x7F
                } else {
                    register | 0x80
                };
                vec![0xB4, adjusted_register, value]
            },
            Commands::MultiPCMWrite {
                register,
                value,
                chip_index,
            } => {
                // Method #2: Use bit 7 of register for chip selection (0x00-7F = chip 1, 0x80-FF = chip 2)
                let adjusted_register = if chip_index == 0 {
                    register & 0x7F
                } else {
                    register | 0x80
                };
                vec![0xB5, adjusted_register, value]
            },
            Commands::uPD7759Write {
                register,
                value,
                chip_index,
            } => {
                // Method #2: Use bit 7 of register for chip selection (0x00-7F = chip 1, 0x80-FF = chip 2)
                let adjusted_register = if chip_index == 0 {
                    register & 0x7F
                } else {
                    register | 0x80
                };
                vec![0xB6, adjusted_register, value]
            },
            Commands::OKIM6258Write {
                register,
                value,
                chip_index,
            } => {
                // Method #2: Use bit 7 of register for chip selection (0x00-7F = chip 1, 0x80-FF = chip 2)
                let adjusted_register = if chip_index == 0 {
                    register & 0x7F
                } else {
                    register | 0x80
                };
                vec![0xB7, adjusted_register, value]
            },
            Commands::OKIM6295Write {
                register,
                value,
                chip_index,
            } => {
                // Method #2: Use bit 7 of register for chip selection (0x00-7F = chip 1, 0x80-FF = chip 2)
                let adjusted_register = if chip_index == 0 {
                    register & 0x7F
                } else {
                    register | 0x80
                };
                vec![0xB8, adjusted_register, value]
            },
            Commands::HuC6280Write {
                register,
                value,
                chip_index,
            } => {
                // Method #2: Use bit 7 of register for chip selection (0x00-7F = chip 1, 0x80-FF = chip 2)
                let adjusted_register = if chip_index == 0 {
                    register & 0x7F
                } else {
                    register | 0x80
                };
                vec![0xB9, adjusted_register, value]
            },
            Commands::K053260Write {
                register,
                value,
                chip_index,
            } => {
                // Method #2: Use bit 7 of register for chip selection (0x00-7F = chip 1, 0x80-FF = chip 2)
                let adjusted_register = if chip_index == 0 {
                    register & 0x7F
                } else {
                    register | 0x80
                };
                vec![0xBA, adjusted_register, value]
            },
            Commands::PokeyWrite {
                register,
                value,
                chip_index,
            } => {
                // Method #2: Use bit 7 of register for chip selection (0x00-7F = chip 1, 0x80-FF = chip 2)
                let adjusted_register = if chip_index == 0 {
                    register & 0x7F
                } else {
                    register | 0x80
                };
                vec![0xBB, adjusted_register, value]
            },
            Commands::WonderSwanWrite {
                register,
                value,
                chip_index,
            } => {
                // Method #2: Use bit 7 of register for chip selection (0x00-7F = chip 1, 0x80-FF = chip 2)
                let adjusted_register = if chip_index == 0 {
                    register & 0x7F
                } else {
                    register | 0x80
                };
                vec![0xBC, adjusted_register, value]
            },
            Commands::SAA1099Write {
                register,
                value,
                chip_index,
            } => {
                // Method #2: Use bit 7 of register for chip selection (0x00-7F = chip 1, 0x80-FF = chip 2)
                let adjusted_register = if chip_index == 0 {
                    register & 0x7F
                } else {
                    register | 0x80
                };
                vec![0xBD, adjusted_register, value]
            },
            Commands::ES5506Write {
                register,
                value,
                chip_index,
            } => {
                // Method #2: Use bit 7 of register for chip selection (0x00-7F = chip 1, 0x80-FF = chip 2)
                let adjusted_register = if chip_index == 0 {
                    register & 0x7F
                } else {
                    register | 0x80
                };
                vec![0xBE, adjusted_register, value]
            },
            Commands::GA20Write {
                register,
                value,
                chip_index,
            } => {
                // Method #2: Use bit 7 of register for chip selection (0x00-7F = chip 1, 0x80-FF = chip 2)
                let adjusted_register = if chip_index == 0 {
                    register & 0x7F
                } else {
                    register | 0x80
                };
                vec![0xBF, adjusted_register, value]
            },
            Commands::SegaPCMWrite { offset, value } => {
                let temp = offset.to_le_bytes();
                vec![0xC0, temp[0], temp[1], value]
            },
            Commands::MultiPCMSetBank { channel, offset } => {
                let temp = offset.to_le_bytes();
                vec![0xC3, temp[0], temp[1], channel]
            },

            Commands::QSoundWrite { register, value } => {
                let temp = value.to_le_bytes();
                vec![0xC4, temp[1], temp[0], register]
            },
            Commands::SCSPWrite { offset, value } => {
                let temp = offset.to_le_bytes();
                vec![0xC5, temp[1], temp[0], value]
            },
            Commands::WonderSwanWrite16 { offset, value } => {
                let temp = offset.to_le_bytes();
                vec![0xC6, temp[1], temp[0], value]
            },
            Commands::VSUWrite { offset, value } => {
                let temp = offset.to_le_bytes();
                vec![0xC7, temp[1], temp[0], value]
            },
            Commands::X1010Write { offset, value } => {
                let temp = offset.to_le_bytes();
                vec![0xC8, temp[1], temp[0], value]
            },

            Commands::YMF278BWrite {
                port,
                register,
                value,
            } => {
                vec![0xD0, port, register, value]
            },

            Commands::YMF271Write {
                port,
                register,
                value,
            } => {
                vec![0xD1, port, register, value]
            },
            Commands::SCC1Write {
                port,
                register,
                value,
            } => {
                vec![0xD2, port, register, value]
            },
            Commands::K054539Write { register, value } => {
                let temp = register.to_le_bytes();
                vec![0xD3, temp[0], temp[1], value]
            },
            Commands::C140Write { register, value } => {
                let temp = register.to_le_bytes();
                vec![0xD4, temp[0], temp[1], value]
            },

            Commands::ES5503Write { register, value } => {
                let temp = register.to_le_bytes();
                vec![0xD5, temp[0], temp[1], value]
            },
            Commands::ES5506Write16 { register, value } => {
                let temp = value.to_le_bytes();
                vec![0xD6, register, temp[0], temp[1]]
            },
            Commands::SeekPCM { offset } => {
                let mut rslt = vec![0xE0];
                rslt.extend(offset.to_le_bytes());
                rslt
            },
            Commands::C352Write { register, value } => {
                let mut rslt = vec![0xE1];
                rslt.extend(register.to_le_bytes());
                rslt.extend(value.to_le_bytes());
                rslt
            },

            // offset write
            Commands::RF5C68WriteOffset { offset, value } => {
                let mut rslt = vec![0xC1];
                rslt.extend(offset.to_le_bytes());
                rslt.extend(value.to_le_bytes());
                rslt
            },
            Commands::RF5C164WriteOffset { offset, value } => {
                let mut rslt = vec![0xC1];
                rslt.extend(offset.to_le_bytes());
                rslt.extend(value.to_le_bytes());
                rslt
            },
        };

        Ok(bytes)
    }
}
