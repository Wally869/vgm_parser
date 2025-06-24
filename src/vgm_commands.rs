#![allow(non_camel_case_types)]

use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use crate::errors::{VgmError, VgmResult};

const MAX_DATA_BLOCK_SIZE: u32 = 16 * 1024 * 1024; // 16MB limit

/// Compression types for compressed data blocks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompressionType {
    BitPacking {
        bits_decompressed: u8,
        bits_compressed: u8,
        sub_type: u8,  // 00=copy, 01=shift left, 02=use table
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
    YM2612,          // 0x00/0x40 - Yamaha YM2612 (SegaPCM streaming)
    RF5C68,          // 0x01/0x41 - Ricoh RF5C68 PCM
    RF5C164,         // 0x02/0x42 - Ricoh RF5C164 PCM
    PWM,             // 0x03/0x43 - Sega PWM
    OKIM6258,        // 0x04/0x44 - OKI MSM6258 ADPCM
    HuC6280,         // 0x05/0x45 - Hudson HuC6280 PCM
    SCSP,            // 0x06/0x46 - Yamaha SCSP PCM
    NESAPU,          // 0x07/0x47 - NES APU DPCM
    Mikey,           // 0x08/0x48 - Atari Lynx Mikey PCM
    Reserved(u8),    // 0x09-0x3F - Reserved for future use
}

/// Chip types for ROM/RAM dump blocks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ROMDumpChipType {
    SegaPCM,         // 0x80 - Sega PCM ROM data
    YM2608DeltaT,    // 0x81 - Yamaha YM2608 DELTA-T ROM
    YM2610ADPCM,     // 0x82 - Yamaha YM2610 ADPCM ROM
    YM2610DeltaT,    // 0x83 - Yamaha YM2610 DELTA-T ROM
    YMF278B,         // 0x84 - Yamaha YMF278B ROM
    YMF271,          // 0x85 - Yamaha YMF271 ROM
    YMZ280B,         // 0x86 - Yamaha YMZ280B ROM
    YMF278BRAM,      // 0x87 - Yamaha YMF278B RAM data
    Y8950DeltaT,     // 0x88 - Yamaha Y8950 DELTA-T ROM
    MultiPCM,        // 0x89 - Sega MultiPCM ROM
    UPD7759,         // 0x8A - NEC uPD7759 ROM
    OKIM6295,        // 0x8B - OKI MSM6295 ROM
    K054539,         // 0x8C - Konami K054539 ROM
    C140,            // 0x8D - Namco C140 ROM
    K053260,         // 0x8E - Konami K053260 ROM
    QSound,          // 0x8F - Capcom Q-Sound ROM
    ES5505_ES5506,   // 0x90 - Ensoniq ES5505/ES5506 ROM
    X1010,           // 0x91 - Seta X1-010 ROM
    C352,            // 0x92 - Namco C352 ROM
    GA20,            // 0x93 - Irem GA20 ROM
    Reserved(u8),    // 0x94-0xBF - Reserved for future use
}

/// Chip types for RAM write blocks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RAMWriteChipType {
    RF5C68,          // 0xC0/0xE0 - Ricoh RF5C68 RAM
    RF5C164,         // 0xC1 - Ricoh RF5C164 RAM
    NESAPU,          // 0xC2 - NES APU RAM
    SCSP,            // 0xE0 - Yamaha SCSP RAM (>64KB)
    ES5503,          // 0xE1 - Ensoniq ES5503 RAM (>64KB)
    Reserved(u8),    // Other values - Reserved for future use
}

impl StreamChipType {
    pub fn from_block_type(block_type: u8) -> Self {
        match block_type & 0x3F {  // Remove compression bit
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
                            details: format!("Unknown compression type: 0x{:02X}", compression_type),
                        });
                    }
                };
                
                let remaining_size = data_size - 9; // 1 + 4 + 4 bytes consumed
                let data: Vec<u8> = (0..remaining_size as usize).map(|_| bytes.get_u8()).collect();
                
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
                let table_data: Vec<u8> = (0..table_size as usize).map(|_| bytes.get_u8()).collect();
                
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
                let data: Vec<u8> = (0..data_size_remaining as usize).map(|_| bytes.get_u8()).collect();
                
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
                let data: Vec<u8> = (0..data_size_remaining as usize).map(|_| bytes.get_u8()).collect();
                
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
                let data: Vec<u8> = (0..data_size_remaining as usize).map(|_| bytes.get_u8()).collect();
                
                Ok(DataBlockContent::RAMWriteLarge {
                    chip_type,
                    start_address,
                    data,
                })
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
pub enum Commands {
    AY8910StereoMask {
        value: u8,
    },
    GameGearPSGStereo {
        value: u8,
    },
    PSGWrite {
        value: u8,
    },
    YM2413Write {
        register: u8,
        value: u8,
    },
    YM2612Port0Write {
        register: u8,
        value: u8,
    },
    YM2612Port1Write {
        register: u8,
        value: u8,
    },
    YM2151Write {
        register: u8,
        value: u8,
    },
    YM2203Write {
        register: u8,
        value: u8,
    },
    YM2608Port0Write {
        register: u8,
        value: u8,
    },
    YM2608Port1Write {
        register: u8,
        value: u8,
    },
    YM2610Port0Write {
        register: u8,
        value: u8,
    },
    YM2610Port1Write {
        register: u8,
        value: u8,
    },
    YM3812Write {
        register: u8,
        value: u8,
    },
    YM3526Write {
        register: u8,
        value: u8,
    },
    Y8950Write {
        register: u8,
        value: u8,
    },
    YMZ280BWrite {
        register: u8,
        value: u8,
    },
    YMF262Port0Write {
        register: u8,
        value: u8,
    },
    YMF262Port1Write {
        register: u8,
        value: u8,
    },
    WaitNSamples {
        n: u16,
    },
    Wait735Samples,
    Wait882Samples,
    EndOfSoundData,
    DataBlock {
        block_type: u8,
        data: DataBlockContent,
    },
    PCMRAMWrite {
        chip_type: u8,
        read_offset: u32,      // 24-bit in VGM spec
        write_offset: u32,     // 24-bit in VGM spec
        size: u32,             // 24-bit in VGM spec
        data: Vec<u8>,
    },
    WaitNSamplesPlus1 {
        n: u8,
    },
    YM2612Port0Address2AWriteWait {
        n: u8,
    },
    // DAC Stream Control Commands (0x90-0x95)
    DACStreamSetupControl {
        stream_id: u8,
        chip_type: u8,
        port: u8,
        command: u8,
    },
    DACStreamSetData {
        stream_id: u8,
        data_bank_id: u8,
        step_size: u8,
        step_base: u8,
    },
    DACStreamSetFrequency {
        stream_id: u8,
        frequency: u32,
    },
    DACStreamStart {
        stream_id: u8,
        data_start_offset: u32,
        length_mode: u8,
        data_length: u32,
    },
    DACStreamStop {
        stream_id: u8,
    },
    DACStreamStartFast {
        stream_id: u8,
        block_id: u16,
        flags: u8,
    },
    AY8910Write {
        register: u8,
        value: u8,
    },
    RF5C68Write {
        register: u8,
        value: u8,
    },
    RF5C164Write {
        register: u8,
        value: u8,
    },
    PWMWrite {
        register: u8,
        value: u16,
    },
    GameBoyDMGWrite {
        register: u8,
        value: u8,
    },
    NESAPUWrite {
        register: u8,
        value: u8,
    },
    MultiPCMWrite {
        register: u8,
        value: u8,
    },
    uPD7759Write {
        register: u8,
        value: u8,
    },
    OKIM6258Write {
        register: u8,
        value: u8,
    },
    OKIM6295Write {
        register: u8,
        value: u8,
    },
    HuC6280Write {
        register: u8,
        value: u8,
    },
    K053260Write {
        register: u8,
        value: u8,
    },
    PokeyWrite {
        register: u8,
        value: u8,
    },
    WonderSwanWrite {
        register: u8,
        value: u8,
    },
    SAA1099Write {
        register: u8,
        value: u8,
    },
    ES5506Write {
        register: u8,
        value: u8,
    },
    GA20Write {
        register: u8,
        value: u8,
    },
    SegaPCMWrite {
        offset: u16,
        value: u8,
    },
    MultiPCMSetBank {
        channel: u8,
        offset: u16,
    },
    QSoundWrite {
        register: u8,
        value: u16,
    },
    SCSPWrite {
        offset: u16,
        value: u8,
    },
    WonderSwanWrite16 {
        offset: u16,
        value: u8,
    },
    VSUWrite {
        offset: u16,
        value: u8,
    },
    X1010Write {
        offset: u16,
        value: u8,
    },
    YMF278BWrite {
        port: u8,
        register: u8,
        value: u8,
    },
    YMF271Write {
        port: u8,
        register: u8,
        value: u8,
    },
    SCC1Write {
        port: u8,
        register: u8,
        value: u8,
    },
    K054539Write {
        register: u16,
        value: u8,
    },
    C140Write {
        register: u16,
        value: u8,
    },
    ES5503Write {
        register: u16,
        value: u8,
    },
    ES5506Write16 {
        register: u8,
        value: u16,
    },
    SeekPCM {
        offset: u32,
    },
    C352Write {
        register: u16,
        value: u16,
    },

    // offset write
    RF5C68WriteOffset {
        offset: u16,
        value: u8,
    },
    RF5C164WriteOffset {
        offset: u16,
        value: u8,
    },
}

pub fn parse_commands(data: &mut Bytes) -> Vec<Commands> {
    // Use default parser config for backward compatibility
    let config = crate::ParserConfig::default();
    let mut tracker = crate::ResourceTracker::new();
    
    match parse_commands_with_config(data, &config, &mut tracker) {
        Ok(commands) => commands,
        Err(e) => {
            println!("Warning: Command parsing failed with error: {}", e);
            vec![] // Return empty commands on error for backward compatibility
        }
    }
}

/// Parse commands with resource tracking and limits
pub fn parse_commands_with_config(
    data: &mut Bytes, 
    config: &crate::ParserConfig, 
    tracker: &mut crate::ResourceTracker
) -> crate::VgmResult<Vec<Commands>> {
    let mut commands = Vec::new();
    let _remaining_at_start = data.len();
    
    loop {
        // Check command count limit before parsing each command
        tracker.track_command(config)?;
        
        match Commands::from_bytes_with_config(data, config, tracker) {
            Ok(curr_command) => {
                match curr_command {
                    Commands::EndOfSoundData => {
                        commands.push(curr_command);
                        break;
                    },
                    _ => commands.push(curr_command),
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(commands)
}

pub fn parse_commands_safe(data: &mut Bytes) -> Vec<Commands> {
    let mut commands = vec![];

    loop {
        let curr_command = Commands::from_bytes_safe(data);
        match curr_command {
            Ok(cmd) => match cmd {
                Commands::EndOfSoundData => {
                    commands.push(cmd);
                    break;
                },
                _ => commands.push(cmd),
            },
            Err(e) => {
                println!("Command parsing error: {}", e);
                break;
            },
        }
    }

    commands
}

pub fn write_commands(buffer: &mut BytesMut, commands: &Vec<Commands>) -> VgmResult<()> {
    for cmd in commands {
        let cmd_bytes = cmd.clone().to_bytes()?;
        buffer.put(&cmd_bytes[..]);
    }
    Ok(())
}

impl Commands {
    pub fn to_bytes(self) -> VgmResult<Vec<u8>> {
        let bytes = match self {
            Commands::AY8910StereoMask { value } => {
                vec![0x31, value]
            },
            Commands::GameGearPSGStereo { value } => {
                vec![0x4f, value]
            },
            Commands::PSGWrite { value } => {
                vec![0x50, value]
            },
            Commands::YM2413Write { register, value } => {
                vec![0x51, register, value]
            },
            Commands::YM2612Port0Write { register, value } => {
                vec![0x52, register, value]
            },
            Commands::YM2612Port1Write { register, value } => {
                vec![0x53, register, value]
            },
            Commands::YM2151Write { register, value } => {
                vec![0x54, register, value]
            },
            Commands::YM2203Write { register, value } => {
                vec![0x55, register, value]
            },
            Commands::YM2608Port0Write { register, value } => {
                vec![0x56, register, value]
            },
            Commands::YM2608Port1Write { register, value } => {
                vec![0x57, register, value]
            },
            Commands::YM2610Port0Write { register, value } => {
                vec![0x58, register, value]
            },
            Commands::YM2610Port1Write { register, value } => {
                vec![0x59, register, value]
            },
            Commands::YM3812Write { register, value } => {
                vec![0x5A, register, value]
            },
            Commands::YM3526Write { register, value } => {
                vec![0x5B, register, value]
            },
            Commands::Y8950Write { register, value } => {
                vec![0x5C, register, value]
            },
            Commands::YMZ280BWrite { register, value } => {
                vec![0x5D, register, value]
            },
            Commands::YMF262Port0Write { register, value } => {
                vec![0x5E, register, value]
            },
            Commands::YMF262Port1Write { register, value } => {
                vec![0x5F, register, value]
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

            Commands::DataBlock {
                block_type,
                data,
            } => {
                // The DataBlock command format: 0x67 0x66 tt ss ss ss ss (data)
                let mut out_data: Vec<u8> = vec![0x67, 0x66, block_type];
                
                // Calculate the size based on the data content
                let data_size = match &data {
                    DataBlockContent::UncompressedStream { data, .. } => data.len() as u32,
                    DataBlockContent::CompressedStream { data, .. } => data.len() as u32 + 9, // +9 for compression header
                    DataBlockContent::DecompressionTable { table_data, .. } => table_data.len() as u32 + 6, // +6 for header
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
                    DataBlockContent::CompressedStream { compression, uncompressed_size, data, .. } => {
                        // Write compression header
                        match compression {
                            CompressionType::BitPacking { bits_decompressed, bits_compressed, sub_type, add_value } => {
                                out_data.push(0x00); // Bit packing compression type
                                out_data.extend(uncompressed_size.to_le_bytes());
                                out_data.push(bits_decompressed);
                                out_data.push(bits_compressed);
                                out_data.push(sub_type);
                                out_data.extend(add_value.to_le_bytes());
                            },
                            CompressionType::DPCM { bits_decompressed, bits_compressed, start_value } => {
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
                    DataBlockContent::DecompressionTable { compression_type, sub_type, bits_decompressed, bits_compressed, value_count, table_data } => {
                        out_data.push(compression_type);
                        out_data.push(sub_type);
                        out_data.push(bits_decompressed);
                        out_data.push(bits_compressed);
                        out_data.extend(value_count.to_le_bytes());
                        out_data.extend(table_data);
                    },
                    DataBlockContent::ROMDump { total_size, start_address, data, .. } => {
                        out_data.extend(total_size.to_le_bytes());
                        out_data.extend(start_address.to_le_bytes());
                        out_data.extend(data);
                    },
                    DataBlockContent::RAMWriteSmall { start_address, data, .. } => {
                        out_data.extend(start_address.to_le_bytes());
                        out_data.extend(data);
                    },
                    DataBlockContent::RAMWriteLarge { start_address, data, .. } => {
                        out_data.extend(start_address.to_le_bytes());
                        out_data.extend(data);
                    },
                    DataBlockContent::Unknown { data } => {
                        out_data.extend(data);
                    },
                }
                
                out_data
            },
            Commands::PCMRAMWrite { chip_type: _, read_offset: _, write_offset: _, size: _, data: _ } => {
                return Err(VgmError::FeatureNotSupported {
                    feature: "PCM RAM Write command serialization".to_string(),
                    version: 0, // Unknown version requirement
                    min_version: 0, // Would need to research the actual VGM version requirement
                });
            },

            Commands::WaitNSamplesPlus1 { n } => vec![0x70 + n],

            Commands::YM2612Port0Address2AWriteWait { n } => vec![0x80 + n],

            // DAC Stream Control Commands (0x90-0x95)
            Commands::DACStreamSetupControl { .. } |
            Commands::DACStreamSetData { .. } |
            Commands::DACStreamSetFrequency { .. } |
            Commands::DACStreamStart { .. } |
            Commands::DACStreamStop { .. } |
            Commands::DACStreamStartFast { .. } => {
                return Err(VgmError::FeatureNotSupported {
                    feature: "DAC Stream Control Write command serialization".to_string(),
                    version: 0, // Unknown version requirement
                    min_version: 0, // Would need to research the actual VGM version requirement
                });
            },

            Commands::AY8910Write { register, value } => {
                vec![0xA0, register, value]
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
            Commands::GameBoyDMGWrite { register, value } => {
                vec![0xB3, register, value]
            },
            Commands::NESAPUWrite { register, value } => {
                vec![0xB4, register, value]
            },
            Commands::MultiPCMWrite { register, value } => {
                vec![0xB5, register, value]
            },
            Commands::uPD7759Write { register, value } => {
                vec![0xB6, register, value]
            },
            Commands::OKIM6258Write { register, value } => {
                vec![0xB7, register, value]
            },
            Commands::OKIM6295Write { register, value } => {
                vec![0xB8, register, value]
            },
            Commands::HuC6280Write { register, value } => {
                vec![0xB9, register, value]
            },
            Commands::K053260Write { register, value } => {
                vec![0xBA, register, value]
            },
            Commands::PokeyWrite { register, value } => {
                vec![0xBB, register, value]
            },
            Commands::WonderSwanWrite { register, value } => {
                vec![0xBC, register, value]
            },
            Commands::SAA1099Write { register, value } => {
                vec![0xBD, register, value]
            },
            Commands::ES5506Write { register, value } => {
                vec![0xBE, register, value]
            },
            Commands::GA20Write { register, value } => {
                vec![0xBF, register, value]
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
            }, // _ => panic!("Not implemented"),
        };
        
        Ok(bytes)
    }

    pub fn from_bytes(bytes: &mut Bytes) -> VgmResult<Commands> {
        let command_val = bytes.get_u8();
        

        let command = match command_val {
            0x31 => {
                // handle AY8910 stereo mask command
                // `bytes.get(1)` gives you the `dd` value
                // create and return a `Command` variant
                Commands::AY8910StereoMask {
                    value: bytes.get_u8(),
                }
            },
            0x4F => {
                // handle Game Gear PSG stereo command
                Commands::GameGearPSGStereo {
                    value: bytes.get_u8(),
                }
            },
            0x50 => {
                // handle PSG write command
                Commands::PSGWrite {
                    value: bytes.get_u8(),
                }
            },
            0x51 => {
                // handle YM2413 write command
                Commands::YM2413Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x52 => {
                // handle YM2612 port 0 write command
                Commands::YM2612Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x53 => {
                // handle YM2612 port 1 write command
                Commands::YM2612Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x54 => {
                // handle YM2151 write command
                Commands::YM2151Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x55 => {
                // handle YM2203 write command
                Commands::YM2203Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x56 => {
                // handle YM2608 port 0 write command
                Commands::YM2608Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x57 => {
                // handle YM2608 port 1 write command
                Commands::YM2608Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x58 => {
                // handle YM2610 port 0 write command
                Commands::YM2610Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x59 => {
                // handle YM2610 port 1 write command
                Commands::YM2610Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5A => {
                // handle YM3812 write command
                Commands::YM3812Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5B => {
                // handle YM3526 write command
                Commands::YM3526Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5C => {
                // handle Y8950 write command
                Commands::Y8950Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5D => {
                // handle YMZ280B write command
                Commands::YMZ280BWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5E => {
                // handle YMF262 port 0 write command
                Commands::YMF262Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5F => {
                // handle YMF262 port 1 write command
                Commands::YMF262Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x61 => {
                // handle wait command
                Commands::WaitNSamples {
                    n: bytes.get_u16_le(),
                }
            },
            0x62 => {
                // handle wait 735 samples command
                Commands::Wait735Samples
            },
            0x63 => {
                // handle wait 882 samples command
                Commands::Wait882Samples
            },
            0x66 => {
                // handle end of sound data command
                Commands::EndOfSoundData
            },
            0x67 => {
                // handle data block command: 0x67 0x66 tt ss ss ss ss (data)
                let compatibility_byte = bytes.get_u8();
                if compatibility_byte != 0x66 {
                    return Err(VgmError::InvalidCommandParameters {
                        opcode: 0x67,
                        position: 0, // TODO: track position
                        reason: format!("Expected compatibility byte 0x66, found 0x{:02X}", compatibility_byte),
                    });
                }
                
                let block_type = bytes.get_u8();
                let data_size = bytes.get_u32_le();
                
                // Security: Check data block size limit
                if data_size > MAX_DATA_BLOCK_SIZE {
                    return Err(VgmError::DataSizeExceedsLimit {
                        field: "DataBlock".to_string(),
                        size: data_size as usize,
                        limit: MAX_DATA_BLOCK_SIZE as usize,
                    });
                }
                
                // Security: Ensure sufficient data is available before allocation
                if bytes.remaining() < data_size as usize {
                    return Err(VgmError::BufferUnderflow {
                        offset: 0, // TODO: Track actual position
                        needed: data_size as usize,
                        available: bytes.remaining(),
                    });
                }
                
                // Parse the data block content based on its type
                let data = DataBlockContent::parse_from_bytes(block_type, data_size, bytes)?;
                
                Commands::DataBlock {
                    block_type,
                    data,
                }
            },
            0x68 => {
                // PCM RAM write command: 0x68 0x66 cc oo oo oo dd dd dd ss ss ss
                let compatibility_byte = bytes.get_u8();
                if compatibility_byte != 0x66 {
                    return Err(VgmError::InvalidCommandParameters {
                        opcode: 0x68,
                        position: 0, // TODO: track position
                        reason: format!("Expected compatibility byte 0x66, found 0x{:02X}", compatibility_byte),
                    });
                }
                
                let chip_type = bytes.get_u8();
                
                // Read 24-bit values (little-endian)
                let read_offset = bytes.get_u8() as u32 | 
                                ((bytes.get_u8() as u32) << 8) | 
                                ((bytes.get_u8() as u32) << 16);
                                
                let write_offset = bytes.get_u8() as u32 | 
                                 ((bytes.get_u8() as u32) << 8) | 
                                 ((bytes.get_u8() as u32) << 16);
                                 
                let mut size = bytes.get_u8() as u32 | 
                             ((bytes.get_u8() as u32) << 8) | 
                             ((bytes.get_u8() as u32) << 16);
                
                // Special case: size of 0 means 0x01000000 bytes
                if size == 0 {
                    size = 0x01000000;
                }
                
                // Read the data
                let data: Vec<u8> = (0..size as usize)
                    .map(|_| bytes.get_u8())
                    .collect();
                
                Commands::PCMRAMWrite {
                    chip_type,
                    read_offset,
                    write_offset,
                    size,
                    data,
                }
            },
            0x70..=0x7F => {
                // handle wait n+1 samples command
                Commands::WaitNSamplesPlus1 {
                    n: command_val - 0x70,
                }
            },
            0x80..=0x8F => {
                // handle YM2612 port 0 address 2A write command
                Commands::YM2612Port0Address2AWriteWait {
                    n: command_val - 0x80,
                }
            },
            0x90..=0x95 => {
                // DAC Stream Control Write commands - proper parsing implementation
                match command_val {
                    0x90 => {
                        // Setup Stream Control: ss tt pp cc (4 bytes)
                        let stream_id = bytes.get_u8();
                        let chip_type = bytes.get_u8();
                        let port = bytes.get_u8();
                        let command = bytes.get_u8();
                        Commands::DACStreamSetupControl { stream_id, chip_type, port, command }
                    },
                    0x91 => {
                        // Set Stream Data: ss dd ll bb (4 bytes)
                        let stream_id = bytes.get_u8();
                        let data_bank_id = bytes.get_u8();
                        let step_size = bytes.get_u8();
                        let step_base = bytes.get_u8();
                        Commands::DACStreamSetData { stream_id, data_bank_id, step_size, step_base }
                    },
                    0x92 => {
                        // Set Stream Frequency: ss ff ff ff ff (5 bytes)
                        let stream_id = bytes.get_u8();
                        let frequency = bytes.get_u32_le();
                        Commands::DACStreamSetFrequency { stream_id, frequency }
                    },
                    0x93 => {
                        // Start Stream: ss aa aa aa aa mm ll ll ll ll (10 bytes)
                        let stream_id = bytes.get_u8();
                        let data_start_offset = bytes.get_u32_le();
                        let length_mode = bytes.get_u8();
                        let data_length = bytes.get_u32_le();
                        Commands::DACStreamStart { stream_id, data_start_offset, length_mode, data_length }
                    },
                    0x94 => {
                        // Stop Stream: ss (1 byte)
                        let stream_id = bytes.get_u8();
                        Commands::DACStreamStop { stream_id }
                    },
                    0x95 => {
                        // Start Stream (fast call): ss bb bb ff (4 bytes)
                        let stream_id = bytes.get_u8();
                        let block_id = bytes.get_u16_le();
                        let flags = bytes.get_u8();
                        Commands::DACStreamStartFast { stream_id, block_id, flags }
                    },
                    _ => unreachable!(), // Range 0x90..=0x95 guarantees this won't happen
                }
            },
            0xA0 => {
                // handle AY8910 write command
                Commands::AY8910Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB0 => {
                // handle RF5C68 write command
                Commands::RF5C68Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB1 => {
                // handle RF5C164 write command
                Commands::RF5C164Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB2 => {
                // handle PWM write command
                // TODO: is not aadd but addd
                Commands::PWMWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u16_le(),
                }
            },
            0xB3 => {
                // handle GameBoy DMG write command
                Commands::GameBoyDMGWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB4 => {
                // handle NES APU write command
                Commands::NESAPUWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB5 => {
                // handle MultiPCM write command
                Commands::MultiPCMWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB6 => {
                // handle uPD7759 write command
                Commands::uPD7759Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB7 => Commands::HuC6280Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xB8 => Commands::K053260Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xB9 => Commands::PokeyWrite {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xBA => Commands::WonderSwanWrite {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xBB => Commands::SAA1099Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xBC => Commands::ES5506Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xBD => Commands::GA20Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xBE => Commands::ES5506Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xBF => Commands::GA20Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xC0 => Commands::SegaPCMWrite {
                offset: bytes.get_u16_le(),
                value: bytes.get_u8(),
            },
            0xC1 => Commands::RF5C68WriteOffset {
                offset: bytes.get_u16_le(),
                value: bytes.get_u8(),
            },
            0xC2 => Commands::RF5C164WriteOffset {
                offset: bytes.get_u16_le(),
                value: bytes.get_u8(),
            },
            0xC3 => Commands::MultiPCMSetBank {
                channel: bytes.get_u8(),
                offset: bytes.get_u16_le(),
            },
            0xC4 => {
                // TODO: weird stuff with the data
                let value = bytes.get_u16_le();
                Commands::QSoundWrite {
                    register: bytes.get_u8(),
                    value,
                }
            },
            0xC5 => {
                // TODO: weird stuff with the data
                //let value = bytes.get_u16_le();
                Commands::SCSPWrite {
                    offset: bytes.get_u16_le(),
                    value: bytes.get_u8(),
                }
            },
            0xC6 => {
                // TODO: check
                Commands::WonderSwanWrite16 {
                    offset: bytes.get_u16_le(),
                    value: bytes.get_u8(),
                }
            },
            0xC7 => {
                // TODO: check
                Commands::VSUWrite {
                    offset: bytes.get_u16_le(),
                    value: bytes.get_u8(),
                }
            },
            0xC8 => {
                // TODO: check
                Commands::X1010Write {
                    offset: bytes.get_u16_le(),
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
                    position: 0  // We'd need to track position properly in a real implementation
                });
            },
        };
        
        Ok(command)
    }

    pub fn from_bytes_safe(bytes: &mut Bytes) -> VgmResult<Commands> {
        let command_val = bytes.get_u8();
        let command = match command_val {
            0x31 => {
                // handle AY8910 stereo mask command
                // `bytes.get(1)` gives you the `dd` value
                // create and return a `Command` variant
                Commands::AY8910StereoMask {
                    value: bytes.get_u8(),
                }
            },
            0x4F => {
                // handle Game Gear PSG stereo command
                Commands::GameGearPSGStereo {
                    value: bytes.get_u8(),
                }
            },
            0x50 => {
                // handle PSG write command
                Commands::PSGWrite {
                    value: bytes.get_u8(),
                }
            },
            0x51 => {
                // handle YM2413 write command
                Commands::YM2413Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x52 => {
                // handle YM2612 port 0 write command
                Commands::YM2612Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x53 => {
                // handle YM2612 port 1 write command
                Commands::YM2612Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x54 => {
                // handle YM2151 write command
                Commands::YM2151Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x55 => {
                // handle YM2203 write command
                Commands::YM2203Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x56 => {
                // handle YM2608 port 0 write command
                Commands::YM2608Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x57 => {
                // handle YM2608 port 1 write command
                Commands::YM2608Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x58 => {
                // handle YM2610 port 0 write command
                Commands::YM2610Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x59 => {
                // handle YM2610 port 1 write command
                Commands::YM2610Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5A => {
                // handle YM3812 write command
                Commands::YM3812Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5B => {
                // handle YM3526 write command
                Commands::YM3526Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5C => {
                // handle Y8950 write command
                Commands::Y8950Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5D => {
                // handle YMZ280B write command
                Commands::YMZ280BWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5E => {
                // handle YMF262 port 0 write command
                Commands::YMF262Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5F => {
                // handle YMF262 port 1 write command
                Commands::YMF262Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x61 => {
                // handle wait command
                Commands::WaitNSamples {
                    n: bytes.get_u16_le(),
                }
            },
            0x62 => {
                // handle wait 735 samples command
                Commands::Wait735Samples
            },
            0x63 => {
                // handle wait 882 samples command
                Commands::Wait882Samples
            },
            0x66 => {
                // handle end of sound data command
                Commands::EndOfSoundData
            },
            0x67 => {
                // handle data block command: 0x67 0x66 tt ss ss ss ss (data)
                let compatibility_byte = bytes.get_u8();
                if compatibility_byte != 0x66 {
                    return Err(VgmError::InvalidCommandParameters {
                        opcode: 0x67,
                        position: 0, // TODO: track position
                        reason: format!("Expected compatibility byte 0x66, found 0x{:02X}", compatibility_byte),
                    });
                }
                
                let block_type = bytes.get_u8();
                let data_size = bytes.get_u32_le();
                
                // Security: Check data block size limit
                if data_size > MAX_DATA_BLOCK_SIZE {
                    return Err(VgmError::DataSizeExceedsLimit {
                        field: "DataBlock".to_string(),
                        size: data_size as usize,
                        limit: MAX_DATA_BLOCK_SIZE as usize,
                    });
                }
                
                // Security: Ensure sufficient data is available before allocation
                if bytes.remaining() < data_size as usize {
                    return Err(VgmError::BufferUnderflow {
                        offset: 0, // TODO: Track actual position
                        needed: data_size as usize,
                        available: bytes.remaining(),
                    });
                }
                
                // Parse the data block content based on its type
                let data = DataBlockContent::parse_from_bytes(block_type, data_size, bytes)?;
                
                Commands::DataBlock {
                    block_type,
                    data,
                }
            },
            0x68 => {
                // PCM RAM write command: 0x68 0x66 cc oo oo oo dd dd dd ss ss ss
                let compatibility_byte = bytes.get_u8();
                if compatibility_byte != 0x66 {
                    return Err(VgmError::InvalidCommandParameters {
                        opcode: 0x68,
                        position: 0, // TODO: track position
                        reason: format!("Expected compatibility byte 0x66, found 0x{:02X}", compatibility_byte),
                    });
                }
                
                let chip_type = bytes.get_u8();
                
                // Read 24-bit values (little-endian)
                let read_offset = bytes.get_u8() as u32 | 
                                ((bytes.get_u8() as u32) << 8) | 
                                ((bytes.get_u8() as u32) << 16);
                                
                let write_offset = bytes.get_u8() as u32 | 
                                 ((bytes.get_u8() as u32) << 8) | 
                                 ((bytes.get_u8() as u32) << 16);
                                 
                let mut size = bytes.get_u8() as u32 | 
                             ((bytes.get_u8() as u32) << 8) | 
                             ((bytes.get_u8() as u32) << 16);
                
                // Special case: size of 0 means 0x01000000 bytes
                if size == 0 {
                    size = 0x01000000;
                }
                
                // Read the data
                let data: Vec<u8> = (0..size as usize)
                    .map(|_| bytes.get_u8())
                    .collect();
                
                Commands::PCMRAMWrite {
                    chip_type,
                    read_offset,
                    write_offset,
                    size,
                    data,
                }
            },
            0x70..=0x7F => {
                // handle wait n+1 samples command
                Commands::WaitNSamplesPlus1 {
                    n: command_val - 0x70,
                }
            },
            0x80..=0x8F => {
                // handle YM2612 port 0 address 2A write command
                Commands::YM2612Port0Address2AWriteWait {
                    n: command_val - 0x80,
                }
            },
            0x90..=0x95 => {
                // DAC Stream Control Write commands - proper parsing implementation
                match command_val {
                    0x90 => {
                        // Setup Stream Control: ss tt pp cc (4 bytes)
                        let stream_id = bytes.get_u8();
                        let chip_type = bytes.get_u8();
                        let port = bytes.get_u8();
                        let command = bytes.get_u8();
                        Commands::DACStreamSetupControl { stream_id, chip_type, port, command }
                    },
                    0x91 => {
                        // Set Stream Data: ss dd ll bb (4 bytes)
                        let stream_id = bytes.get_u8();
                        let data_bank_id = bytes.get_u8();
                        let step_size = bytes.get_u8();
                        let step_base = bytes.get_u8();
                        Commands::DACStreamSetData { stream_id, data_bank_id, step_size, step_base }
                    },
                    0x92 => {
                        // Set Stream Frequency: ss ff ff ff ff (5 bytes)
                        let stream_id = bytes.get_u8();
                        let frequency = bytes.get_u32_le();
                        Commands::DACStreamSetFrequency { stream_id, frequency }
                    },
                    0x93 => {
                        // Start Stream: ss aa aa aa aa mm ll ll ll ll (10 bytes)
                        let stream_id = bytes.get_u8();
                        let data_start_offset = bytes.get_u32_le();
                        let length_mode = bytes.get_u8();
                        let data_length = bytes.get_u32_le();
                        Commands::DACStreamStart { stream_id, data_start_offset, length_mode, data_length }
                    },
                    0x94 => {
                        // Stop Stream: ss (1 byte)
                        let stream_id = bytes.get_u8();
                        Commands::DACStreamStop { stream_id }
                    },
                    0x95 => {
                        // Start Stream (fast call): ss bb bb ff (4 bytes)
                        let stream_id = bytes.get_u8();
                        let block_id = bytes.get_u16_le();
                        let flags = bytes.get_u8();
                        Commands::DACStreamStartFast { stream_id, block_id, flags }
                    },
                    _ => unreachable!(), // Range 0x90..=0x95 guarantees this won't happen
                }
            },
            0xA0 => {
                // handle AY8910 write command
                Commands::AY8910Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB0 => {
                // handle RF5C68 write command
                Commands::RF5C68Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB1 => {
                // handle RF5C164 write command
                Commands::RF5C164Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB2 => {
                // handle PWM write command
                // TODO: is not aadd but addd
                Commands::PWMWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u16_le(),
                }
            },
            0xB3 => {
                // handle GameBoy DMG write command
                Commands::GameBoyDMGWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB4 => {
                // handle NES APU write command
                Commands::NESAPUWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB5 => {
                // handle MultiPCM write command
                Commands::MultiPCMWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB6 => {
                // handle uPD7759 write command
                Commands::uPD7759Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB7 => Commands::HuC6280Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xB8 => Commands::K053260Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xB9 => Commands::PokeyWrite {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xBA => Commands::WonderSwanWrite {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xBB => Commands::SAA1099Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xBC => Commands::ES5506Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xBD => Commands::GA20Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xBE => Commands::ES5506Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xBF => Commands::GA20Write {
                register: bytes.get_u8(),
                value: bytes.get_u8(),
            },
            0xC0 => Commands::SegaPCMWrite {
                offset: bytes.get_u16_le(),
                value: bytes.get_u8(),
            },
            0xC1 => Commands::RF5C68WriteOffset {
                offset: bytes.get_u16_le(),
                value: bytes.get_u8(),
            },
            0xC2 => Commands::RF5C164WriteOffset {
                offset: bytes.get_u16_le(),
                value: bytes.get_u8(),
            },
            0xC3 => Commands::MultiPCMSetBank {
                channel: bytes.get_u8(),
                offset: bytes.get_u16_le(),
            },
            0xC4 => {
                // TODO: weird stuff with the data
                let value = bytes.get_u16_le();
                Commands::QSoundWrite {
                    register: bytes.get_u8(),
                    value,
                }
            },
            0xC5 => {
                // TODO: weird stuff with the data
                //let value = bytes.get_u16_le();
                Commands::SCSPWrite {
                    offset: bytes.get_u16_le(),
                    value: bytes.get_u8(),
                }
            },
            0xC6 => {
                // TODO: check
                Commands::WonderSwanWrite16 {
                    offset: bytes.get_u16_le(),
                    value: bytes.get_u8(),
                }
            },
            0xC7 => {
                // TODO: check
                Commands::VSUWrite {
                    offset: bytes.get_u16_le(),
                    value: bytes.get_u8(),
                }
            },
            0xC8 => {
                // TODO: check
                Commands::X1010Write {
                    offset: bytes.get_u16_le(),
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
                    position: 0  // TODO: Track actual position
                });
            },
        };

        Ok(command)
    }
    
    /// Parse command with resource tracking and allocation limits
    pub fn from_bytes_with_config(
        bytes: &mut Bytes,
        config: &crate::ParserConfig,
        tracker: &mut crate::ResourceTracker
    ) -> VgmResult<Commands> {
        let command_val = bytes.get_u8();
        
        let command = match command_val {
            0x31 => {
                Commands::AY8910StereoMask {
                    value: bytes.get_u8(),
                }
            },
            0x4F => {
                Commands::GameGearPSGStereo {
                    value: bytes.get_u8(),
                }
            },
            0x50 => {
                Commands::PSGWrite {
                    value: bytes.get_u8(),
                }
            },
            0x51 => {
                Commands::YM2413Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x52 => {
                Commands::YM2612Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x53 => {
                Commands::YM2612Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x54 => {
                Commands::YM2151Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x55 => {
                Commands::YM2203Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x56 => {
                Commands::YM2608Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x57 => {
                Commands::YM2608Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x58 => {
                Commands::YM2610Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x59 => {
                Commands::YM2610Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5A => {
                Commands::YM3812Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5B => {
                Commands::YM3526Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5C => {
                Commands::Y8950Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5D => {
                Commands::YMZ280BWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5E => {
                Commands::YMF262Port0Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x5F => {
                Commands::YMF262Port1Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0x61 => {
                Commands::WaitNSamples {
                    n: bytes.get_u16_le(),
                }
            },
            0x62 => Commands::Wait735Samples,
            0x63 => Commands::Wait882Samples,
            0x66 => Commands::EndOfSoundData,
            0x67 => {
                // handle data block command: 0x67 0x66 tt ss ss ss ss (data)
                let compatibility_byte = bytes.get_u8();
                if compatibility_byte != 0x66 {
                    return Err(VgmError::InvalidCommandParameters {
                        opcode: 0x67,
                        position: 0, // TODO: track position
                        reason: format!("Expected compatibility byte 0x66, found 0x{:02X}", compatibility_byte),
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
                        offset: 0, // TODO: Track actual position
                        needed: data_size as usize,
                        available: bytes.remaining(),
                    });
                }
                
                // Parse the data block content based on its type
                let data = DataBlockContent::parse_from_bytes(block_type, data_size, bytes)?;
                
                Commands::DataBlock {
                    block_type,
                    data,
                }
            },
            0x68 => {
                // PCM RAM write command: 0x68 0x66 cc oo oo oo dd dd dd ss ss ss
                let compatibility_byte = bytes.get_u8();
                if compatibility_byte != 0x66 {
                    return Err(VgmError::InvalidCommandParameters {
                        opcode: 0x68,
                        position: 0, // TODO: track position
                        reason: format!("Expected compatibility byte 0x66, found 0x{:02X}", compatibility_byte),
                    });
                }
                
                let chip_type = bytes.get_u8();
                
                // Read 24-bit values (little-endian)
                let read_offset = bytes.get_u8() as u32 | 
                                ((bytes.get_u8() as u32) << 8) | 
                                ((bytes.get_u8() as u32) << 16);
                                
                let write_offset = bytes.get_u8() as u32 | 
                                 ((bytes.get_u8() as u32) << 8) | 
                                 ((bytes.get_u8() as u32) << 16);
                                 
                let mut size = bytes.get_u8() as u32 | 
                             ((bytes.get_u8() as u32) << 8) | 
                             ((bytes.get_u8() as u32) << 16);
                
                // Special case: size of 0 means 0x01000000 bytes
                if size == 0 {
                    size = 0x01000000;
                }
                
                // Security: Check data size limits before allocation
                if size as usize > config.max_command_memory {
                    return Err(VgmError::DataSizeExceedsLimit {
                        field: "PCM RAM write data".to_string(),
                        size: size as usize,
                        limit: config.max_command_memory,
                    });
                }
                
                // Check buffer availability
                if bytes.remaining() < size as usize {
                    return Err(VgmError::BufferUnderflow {
                        offset: 0, // TODO: track position
                        needed: size as usize,
                        available: bytes.remaining(),
                    });
                }
                
                // Track memory allocation
                tracker.track_data_block(config, size)?;
                
                // Read the data
                let data: Vec<u8> = (0..size as usize)
                    .map(|_| bytes.get_u8())
                    .collect();
                
                Commands::PCMRAMWrite {
                    chip_type,
                    read_offset,
                    write_offset,
                    size,
                    data,
                }
            },
            0x70..=0x7F => {
                Commands::WaitNSamplesPlus1 {
                    n: command_val - 0x70,
                }
            },
            0x80..=0x8F => {
                Commands::YM2612Port0Address2AWriteWait {
                    n: command_val - 0x80,
                }
            },
            0x90 => {
                // Setup Stream Control: ss tt pp cc (4 bytes)
                let stream_id = bytes.get_u8();
                let chip_type = bytes.get_u8();
                let port = bytes.get_u8();
                let command = bytes.get_u8();
                Commands::DACStreamSetupControl { stream_id, chip_type, port, command }
            },
            0x91 => {
                // Set Stream Data: ss dd ll bb (4 bytes)
                let stream_id = bytes.get_u8();
                let data_bank_id = bytes.get_u8();
                let step_size = bytes.get_u8();
                let step_base = bytes.get_u8();
                Commands::DACStreamSetData { stream_id, data_bank_id, step_size, step_base }
            },
            0x92 => {
                // Set Stream Frequency: ss ff ff ff ff (5 bytes)
                let stream_id = bytes.get_u8();
                let frequency = bytes.get_u32_le();
                Commands::DACStreamSetFrequency { stream_id, frequency }
            },
            0x93 => {
                // Start Stream: ss aa aa aa aa mm ll ll ll ll (10 bytes)
                let stream_id = bytes.get_u8();
                let data_start_offset = bytes.get_u32_le();
                let length_mode = bytes.get_u8();
                let data_length = bytes.get_u32_le();
                Commands::DACStreamStart { stream_id, data_start_offset, length_mode, data_length }
            },
            0x94 => {
                // Stop Stream: ss (1 byte)
                let stream_id = bytes.get_u8();
                Commands::DACStreamStop { stream_id }
            },
            0x95 => {
                // Start Stream (fast call): ss bb bb ff (4 bytes)
                let stream_id = bytes.get_u8();
                let block_id = bytes.get_u16_le();
                let flags = bytes.get_u8();
                Commands::DACStreamStartFast { stream_id, block_id, flags }
            },
            0xA0 => {
                Commands::AY8910Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB0 => {
                Commands::RF5C68Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB1 => {
                Commands::RF5C164Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB2 => {
                Commands::PWMWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u16_le(),
                }
            },
            0xB3 => {
                Commands::GameBoyDMGWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB4 => {
                Commands::NESAPUWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB5 => {
                Commands::MultiPCMWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB6 => {
                Commands::uPD7759Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB7 => {
                Commands::OKIM6258Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB8 => {
                Commands::OKIM6295Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xB9 => {
                Commands::HuC6280Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xBA => {
                Commands::K053260Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xBB => {
                Commands::PokeyWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xBC => {
                Commands::WonderSwanWrite {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xBD => {
                Commands::SAA1099Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xBE => {
                Commands::ES5506Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xBF => {
                Commands::GA20Write {
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xC0 => {
                Commands::SegaPCMWrite {
                    offset: bytes.get_u16_le(),
                    value: bytes.get_u8(),
                }
            },
            0xC1 => {
                Commands::RF5C68WriteOffset {
                    offset: bytes.get_u16_le(),
                    value: bytes.get_u8(),
                }
            },
            0xC2 => {
                Commands::RF5C164WriteOffset {
                    offset: bytes.get_u16_le(),
                    value: bytes.get_u8(),
                }
            },
            0xD0 => {
                Commands::YMF278BWrite {
                    port: bytes.get_u8(),
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xD1 => {
                Commands::YMF271Write {
                    port: bytes.get_u8(),
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xD2 => {
                Commands::SCC1Write {
                    port: bytes.get_u8(),
                    register: bytes.get_u8(),
                    value: bytes.get_u8(),
                }
            },
            0xD3 => {
                Commands::K054539Write {
                    register: bytes.get_u16_le(),
                    value: bytes.get_u8(),
                }
            },
            0xD4 => {
                Commands::C140Write {
                    register: bytes.get_u16_le(),
                    value: bytes.get_u8(),
                }
            },
            0xD5 => {
                Commands::ES5503Write {
                    register: bytes.get_u16_le(),
                    value: bytes.get_u8(),
                }
            },
            0xD6 => {
                Commands::ES5506Write16 {
                    register: bytes.get_u8(),
                    value: bytes.get_u16_le(),
                }
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
                    position: 0  // TODO: Track actual position
                });
            },
        };

        Ok(command)
    }
}
