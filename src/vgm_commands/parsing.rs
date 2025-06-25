//! VGM Command Parsing Module
//!
//! Contains the core parsing logic for converting raw VGM byte streams into Commands enum variants.
//! Includes three parsing implementations: standard, safe, and config-aware with resource tracking.

use super::commands::Commands;
use super::data_blocks::DataBlockContent;
use super::MAX_DATA_BLOCK_SIZE;
use crate::errors::{VgmError, VgmResult};
use crate::{ParserConfig, ResourceTracker};
use bytes::{Buf, BufMut, Bytes, BytesMut};

impl Commands {
    /// Standard parsing method for converting bytes to Commands
    pub fn from_bytes(bytes: &mut Bytes) -> VgmResult<Commands> {
        let command_val = bytes.get_u8();

        let command = match command_val {
            0x30 => {
                // PSG write command - second chip (dual chip support)
                Commands::PSGWrite {
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0x31 => {
                // AY8910 stereo mask command
                Commands::AY8910StereoMask {
                    value: bytes.get_u8(),
                }
            },
            0x3F => {
                // Game Gear PSG stereo command - second chip (dual chip support)
                Commands::GameGearPSGStereo {
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0x4F => {
                // Game Gear PSG stereo command - first chip (dual chip support)
                Commands::GameGearPSGStereo {
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x50 => {
                // PSG write command - first chip (dual chip support)
                Commands::PSGWrite {
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x51 => {
                // YM2413 write - first chip
                Commands::YM2413Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x52 => {
                // YM2612 port 0 write - first chip
                Commands::YM2612Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x53 => {
                // YM2612 port 1 write - first chip
                Commands::YM2612Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x54 => {
                // YM2151 write - first chip
                Commands::YM2151Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x55 => {
                // YM2203 write - first chip
                Commands::YM2203Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x56 => {
                // YM2608 port 0 write - first chip
                Commands::YM2608Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x57 => {
                // YM2608 port 1 write - first chip
                Commands::YM2608Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x58 => {
                // YM2610 port 0 write - first chip
                Commands::YM2610Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x59 => {
                // YM2610 port 1 write - first chip
                Commands::YM2610Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x5A => {
                // YM3812 write - first chip
                Commands::YM3812Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x5B => {
                // YM3526 write - first chip
                Commands::YM3526Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x5C => {
                // Y8950 write - first chip
                Commands::Y8950Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x5D => {
                // YMZ280B write - first chip
                Commands::YMZ280BWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x5E => {
                // YMF262 port 0 write - first chip
                Commands::YMF262Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x5F => {
                // YMF262 port 1 write - first chip
                Commands::YMF262Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 0,
                }
            },
            0x61 => {
                // Wait n samples
                let n = bytes.get_u16_le();
                Commands::WaitNSamples { n }
            },
            0x62 => Commands::Wait735Samples,
            0x63 => Commands::Wait882Samples,
            0x66 => Commands::EndOfSoundData,
            0x67 => {
                // Data block command: 0x67 0x66 tt ss ss ss ss (data)
                let compatibility_byte = bytes.get_u8();
                if compatibility_byte != 0x66 {
                    return Err(VgmError::InvalidCommandParameters {
                        opcode: 0x67,
                        position: 0,
                        reason: format!(
                            "Expected compatibility byte 0x66, found 0x{:02X}",
                            compatibility_byte
                        ),
                    });
                }

                let block_type = bytes.get_u8();
                let data_size = bytes.get_u32_le();

                if data_size > MAX_DATA_BLOCK_SIZE {
                    return Err(VgmError::InvalidDataFormat {
                        field: "data_block_size".to_string(),
                        details: format!(
                            "Data block size {} exceeds maximum {}",
                            data_size, MAX_DATA_BLOCK_SIZE
                        ),
                    });
                }

                if bytes.remaining() < data_size as usize {
                    return Err(VgmError::BufferUnderflow {
                        offset: 0,
                        needed: data_size as usize,
                        available: bytes.remaining(),
                    });
                }

                let data = DataBlockContent::parse_from_bytes(block_type, data_size, bytes)?;

                Commands::DataBlock { block_type, data }
            },
            0x68 => {
                // PCM RAM write command: 0x68 0x66 cc oo oo oo dd dd dd ss ss ss
                let compatibility_byte = bytes.get_u8();
                if compatibility_byte != 0x66 {
                    return Err(VgmError::InvalidCommandParameters {
                        opcode: 0x68,
                        position: 0,
                        reason: format!(
                            "Expected compatibility byte 0x66, found 0x{:02X}",
                            compatibility_byte
                        ),
                    });
                }

                let chip_type = bytes.get_u8();

                // Read 24-bit values (little-endian)
                let read_offset = bytes.get_u8() as u32
                    | ((bytes.get_u8() as u32) << 8)
                    | ((bytes.get_u8() as u32) << 16);

                let write_offset = bytes.get_u8() as u32
                    | ((bytes.get_u8() as u32) << 8)
                    | ((bytes.get_u8() as u32) << 16);

                let mut size = bytes.get_u8() as u32
                    | ((bytes.get_u8() as u32) << 8)
                    | ((bytes.get_u8() as u32) << 16);

                // Special case: size of 0 means 0x01000000 bytes
                if size == 0 {
                    size = 0x01000000;
                }

                let data: Vec<u8> = (0..size).map(|_| bytes.get_u8()).collect();

                Commands::PCMRAMWrite {
                    chip_type,
                    read_offset,
                    write_offset,
                    size,
                    data,
                }
            },
            0x70..=0x7F => {
                // Wait n+1 samples
                Commands::WaitNSamplesPlus1 {
                    n: command_val - 0x70,
                }
            },
            0x80..=0x8F => {
                // YM2612 port 0 address 2A write + wait n samples
                Commands::YM2612Port0Address2AWriteWait {
                    n: command_val - 0x80,
                }
            },
            0x90 => {
                // DAC Stream Setup Control
                let stream_id = bytes.get_u8();
                let chip_type = bytes.get_u8();
                let port = bytes.get_u8();
                let command = bytes.get_u8();

                // Decode dual chip support from chip_type bit 7
                let chip_index = if (chip_type & 0x80) != 0 { 1 } else { 0 };
                let chip_type = chip_type & 0x7F;

                Commands::DACStreamSetupControl {
                    stream_id,
                    chip_type,
                    port,
                    command,
                    chip_index,
                }
            },
            0x91 => {
                // DAC Stream Set Data
                Commands::DACStreamSetData {
                    stream_id: bytes.get_u8(),
                    data_bank_id: bytes.get_u8(),
                    step_size: bytes.get_u8(),
                    step_base: bytes.get_u8(),
                }
            },
            0x92 => {
                // DAC Stream Set Frequency
                Commands::DACStreamSetFrequency {
                    stream_id: bytes.get_u8(),
                    frequency: bytes.get_u32_le(),
                }
            },
            0x93 => {
                // DAC Stream Start
                let stream_id = bytes.get_u8();
                let data_start_offset = bytes.get_u32_le();
                let length_mode = bytes.get_u8();
                let data_length = bytes.get_u32_le();

                Commands::DACStreamStart {
                    stream_id,
                    data_start_offset,
                    length_mode,
                    data_length,
                }
            },
            0x94 => {
                // DAC Stream Stop
                Commands::DACStreamStop {
                    stream_id: bytes.get_u8(),
                }
            },
            0x95 => {
                // DAC Stream Start Fast
                Commands::DACStreamStartFast {
                    stream_id: bytes.get_u8(),
                    block_id: bytes.get_u16_le(),
                    flags: bytes.get_u8(),
                }
            },
            0xA0 => {
                // AY8910 write with dual chip support via register bit 7
                let register = bytes.get_u8();
                let chip_index = if (register & 0x80) != 0 { 1 } else { 0 };
                let register = register & 0x7F;

                Commands::AY8910Write {
                    register,
                    value: bytes.get_u8(),
                    chip_index,
                }
            },
            0xA1 => {
                // YM2413 write - second chip
                Commands::YM2413Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0xA2 => {
                // YM2612 port 0 write - second chip
                Commands::YM2612Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0xA3 => {
                // YM2612 port 1 write - second chip
                Commands::YM2612Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0xA4 => {
                // YM2151 write - second chip
                Commands::YM2151Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0xA5 => {
                // YM2203 write - second chip
                Commands::YM2203Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0xA6 => {
                // YM2608 port 0 write - second chip
                Commands::YM2608Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0xA7 => {
                // YM2608 port 1 write - second chip
                Commands::YM2608Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0xA8 => {
                // YM2610 port 0 write - second chip
                Commands::YM2610Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0xA9 => {
                // YM2610 port 1 write - second chip
                Commands::YM2610Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0xAA => {
                // YM3812 write - second chip
                Commands::YM3812Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0xAB => {
                // YM3526 write - second chip
                Commands::YM3526Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0xAC => {
                // Y8950 write - second chip
                Commands::Y8950Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0xAD => {
                // YMZ280B write - second chip
                Commands::YMZ280BWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0xAE => {
                // YMF262 port 0 write - second chip
                Commands::YMF262Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0xAF => {
                // YMF262 port 1 write - second chip
                Commands::YMF262Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                    chip_index: 1,
                }
            },
            0xB0 => Commands::RF5C68Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xB1 => Commands::RF5C164Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xB2 => Commands::PWMWrite {
                register: bytes.get_u8(),
                value: bytes.get_u16_le(),
            },
            0xB3 => {
                // GameBoy DMG write with dual chip support via register bit 7
                let register = bytes.get_u8();
                let chip_index = if (register & 0x80) != 0 { 1 } else { 0 };
                let register = register & 0x7F;

                Commands::GameBoyDMGWrite {
                    register,
                    value: bytes.get_u8(),
                    chip_index,
                }
            },
            0xB4 => {
                // NES APU write with dual chip support via register bit 7
                let register = bytes.get_u8();
                let chip_index = if (register & 0x80) != 0 { 1 } else { 0 };
                let register = register & 0x7F;

                Commands::NESAPUWrite {
                    register,
                    value: bytes.get_u8(),
                    chip_index,
                }
            },
            0xB5 => {
                // MultiPCM write with dual chip support via register bit 7
                let register = bytes.get_u8();
                let chip_index = if (register & 0x80) != 0 { 1 } else { 0 };
                let register = register & 0x7F;

                Commands::MultiPCMWrite {
                    register,
                    value: bytes.get_u8(),
                    chip_index,
                }
            },
            0xB6 => {
                // uPD7759 write with dual chip support via register bit 7
                let register = bytes.get_u8();
                let chip_index = if (register & 0x80) != 0 { 1 } else { 0 };
                let register = register & 0x7F;

                Commands::uPD7759Write {
                    register,
                    value: bytes.get_u8(),
                    chip_index,
                }
            },
            0xB7 => {
                // OKIM6258 write with dual chip support via register bit 7
                let register = bytes.get_u8();
                let chip_index = if (register & 0x80) != 0 { 1 } else { 0 };
                let register = register & 0x7F;

                Commands::OKIM6258Write {
                    register,
                    value: bytes.get_u8(),
                    chip_index,
                }
            },
            0xB8 => {
                // OKIM6295 write with dual chip support via register bit 7
                let register = bytes.get_u8();
                let chip_index = if (register & 0x80) != 0 { 1 } else { 0 };
                let register = register & 0x7F;

                Commands::OKIM6295Write {
                    register,
                    value: bytes.get_u8(),
                    chip_index,
                }
            },
            0xB9 => {
                // HuC6280 write with dual chip support via register bit 7
                let register = bytes.get_u8();
                let chip_index = if (register & 0x80) != 0 { 1 } else { 0 };
                let register = register & 0x7F;

                Commands::HuC6280Write {
                    register,
                    value: bytes.get_u8(),
                    chip_index,
                }
            },
            0xBA => {
                // K053260 write with dual chip support via register bit 7
                let register = bytes.get_u8();
                let chip_index = if (register & 0x80) != 0 { 1 } else { 0 };
                let register = register & 0x7F;

                Commands::K053260Write {
                    register,
                    value: bytes.get_u8(),
                    chip_index,
                }
            },
            0xBB => {
                // Pokey write with dual chip support via register bit 7
                let register = bytes.get_u8();
                let chip_index = if (register & 0x80) != 0 { 1 } else { 0 };
                let register = register & 0x7F;

                Commands::PokeyWrite {
                    register,
                    value: bytes.get_u8(),
                    chip_index,
                }
            },
            0xBC => {
                // WonderSwan write with dual chip support via register bit 7
                let register = bytes.get_u8();
                let chip_index = if (register & 0x80) != 0 { 1 } else { 0 };
                let register = register & 0x7F;

                Commands::WonderSwanWrite {
                    register,
                    value: bytes.get_u8(),
                    chip_index,
                }
            },
            0xBD => {
                // SAA1099 write with dual chip support via register bit 7
                let register = bytes.get_u8();
                let chip_index = if (register & 0x80) != 0 { 1 } else { 0 };
                let register = register & 0x7F;

                Commands::SAA1099Write {
                    register,
                    value: bytes.get_u8(),
                    chip_index,
                }
            },
            0xBE => {
                // ES5506 write with dual chip support via register bit 7
                let register = bytes.get_u8();
                let chip_index = if (register & 0x80) != 0 { 1 } else { 0 };
                let register = register & 0x7F;

                Commands::ES5506Write {
                    register,
                    value: bytes.get_u8(),
                    chip_index,
                }
            },
            0xBF => {
                // GA20 write with dual chip support via register bit 7
                let register = bytes.get_u8();
                let chip_index = if (register & 0x80) != 0 { 1 } else { 0 };
                let register = register & 0x7F;

                Commands::GA20Write {
                    register,
                    value: bytes.get_u8(),
                    chip_index,
                }
            },
            0xC0 => Commands::SegaPCMWrite {
                offset: bytes.get_u16_le(),
                value: bytes.get_u8(),
            },
            0xC1 => {
                // RF5C68 or RF5C164 offset write (need to determine which)
                Commands::RF5C68WriteOffset {
                    offset: bytes.get_u16_le(),
                    value: bytes.get_u8(),
                }
            },
            0xC3 => Commands::MultiPCMSetBank {
                channel: bytes.get_u8(),
                offset: bytes.get_u16_le(),
            },
            0xC4 => Commands::QSoundWrite {
                register: bytes.get_u8(),
                value: bytes.get_u16_le(),
            },
            0xC5 => {
                Commands::SCSPWrite {
                    offset: bytes.get_u16_le(), // Actually little-endian for SCSP
                    value: bytes.get_u8(),
                }
            },
            0xC6 => {
                Commands::WonderSwanWrite16 {
                    offset: bytes.get_u16_le(), // Actually little-endian
                    value: bytes.get_u8(),
                }
            },
            0xC7 => {
                Commands::VSUWrite {
                    offset: bytes.get_u16_le(), // Actually little-endian
                    value: bytes.get_u8(),
                }
            },
            0xC8 => {
                Commands::X1010Write {
                    offset: bytes.get_u16_le(), // Actually little-endian
                    value: bytes.get_u8(),
                }
            },
            0xD0 => Commands::YMF278BWrite {
                port: bytes.get_u8(),
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xD1 => Commands::YMF271Write {
                port: bytes.get_u8(),
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xD2 => Commands::SCC1Write {
                port: bytes.get_u8(),
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xD3 => Commands::K054539Write {
                register: bytes.get_u16_le(),
                value: bytes.get_u8(),
            },
            0xD4 => Commands::C140Write {
                register: bytes.get_u16_le(),
                value: bytes.get_u8(),
            },
            0xD5 => Commands::ES5503Write {
                register: bytes.get_u16_le(),
                value: bytes.get_u8(),
            },
            0xD6 => Commands::ES5506Write16 {
                register: bytes.get_u8(),
                value: bytes.get_u16_le(),
            },
            0xE0 => Commands::SeekPCM {
                offset: bytes.get_u32_le(),
            },
            0xE1 => Commands::C352Write {
                register: bytes.get_u16_le(),
                value: bytes.get_u16_le(),
            },
            _ => {
                return Err(VgmError::UnknownCommand {
                    opcode: command_val,
                    position: 0,
                });
            },
        };

        Ok(command)
    }

    /// Safe parsing method with identical logic to from_bytes
    pub fn from_bytes_safe(bytes: &mut Bytes) -> VgmResult<Commands> {
        // For now, this is identical to from_bytes
        // Could be enhanced with additional safety checks in the future
        Self::from_bytes(bytes)
    }

    /// Parse command with resource tracking and allocation limits
    pub fn from_bytes_with_config(
        bytes: &mut Bytes,
        config: &ParserConfig,
        tracker: &mut ResourceTracker,
    ) -> VgmResult<Commands> {
        let command_val = bytes.get_u8();

        let command = match command_val {
            // Use same parsing logic as from_bytes but with additional config checks
            0x67 => {
                // Enhanced data block parsing with config checks
                let compatibility_byte = bytes.get_u8();
                if compatibility_byte != 0x66 {
                    return Err(VgmError::InvalidCommandParameters {
                        opcode: 0x67,
                        position: 0,
                        reason: format!(
                            "Expected compatibility byte 0x66, found 0x{:02X}",
                            compatibility_byte
                        ),
                    });
                }

                let block_type = bytes.get_u8();
                let data_size = bytes.get_u32_le();

                // Check DataBlock size against config limits
                config.check_data_block_size(data_size)?;

                // Track DataBlock allocation
                tracker.track_data_block(config, data_size)?;

                // Security: Ensure sufficient data is available before allocation
                if bytes.remaining() < data_size as usize {
                    return Err(VgmError::BufferUnderflow {
                        offset: 0,
                        needed: data_size as usize,
                        available: bytes.remaining(),
                    });
                }

                let data = DataBlockContent::parse_from_bytes(block_type, data_size, bytes)?;

                Commands::DataBlock { block_type, data }
            },
            0x68 => {
                // Enhanced PCM RAM write with config checks: 0x68 0x66 cc oo oo oo dd dd dd ss ss ss
                let compatibility_byte = bytes.get_u8();
                if compatibility_byte != 0x66 {
                    return Err(VgmError::InvalidCommandParameters {
                        opcode: 0x68,
                        position: 0,
                        reason: format!(
                            "Expected compatibility byte 0x66, found 0x{:02X}",
                            compatibility_byte
                        ),
                    });
                }

                let chip_type = bytes.get_u8();

                // Read 24-bit values (little-endian)
                let read_offset = bytes.get_u8() as u32
                    | ((bytes.get_u8() as u32) << 8)
                    | ((bytes.get_u8() as u32) << 16);

                let write_offset = bytes.get_u8() as u32
                    | ((bytes.get_u8() as u32) << 8)
                    | ((bytes.get_u8() as u32) << 16);

                let mut size = bytes.get_u8() as u32
                    | ((bytes.get_u8() as u32) << 8)
                    | ((bytes.get_u8() as u32) << 16);

                // Special case: size of 0 means 0x01000000 bytes
                if size == 0 {
                    size = 0x01000000;
                }

                // Check PCM RAM write size against config limits
                config.check_data_block_size(size)?;

                // Track PCM RAM write allocation
                tracker.track_data_block(config, size)?;

                // Security: Ensure sufficient data is available
                if bytes.remaining() < size as usize {
                    return Err(VgmError::BufferUnderflow {
                        offset: 0,
                        needed: size as usize,
                        available: bytes.remaining(),
                    });
                }

                let data: Vec<u8> = (0..size).map(|_| bytes.get_u8()).collect();

                Commands::PCMRAMWrite {
                    chip_type,
                    read_offset,
                    write_offset,
                    size,
                    data,
                }
            },
            _ => {
                // For all other commands, use standard parsing logic
                // We need to create a new buffer with the command byte we already read
                let mut temp_bytes = BytesMut::new();
                temp_bytes.put_u8(command_val);
                temp_bytes.put(bytes.clone());
                let mut final_bytes = temp_bytes.freeze();

                return Self::from_bytes(&mut final_bytes);
            },
        };

        Ok(command)
    }
}
