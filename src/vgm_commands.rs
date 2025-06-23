#![allow(non_camel_case_types)]

use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};

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
        data_type: u8,
        data_size: u32,
        data: Vec<u8>,
    },
    PCMRAMWrite {
        offset: u32,
        data: Vec<u8>,
    },
    WaitNSamplesPlus1 {
        n: u8,
    },
    YM2612Port0Address2AWriteWait {
        n: u8,
    },
    DACStreamControlWrite {
        register: u8,
        value: u8,
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
    let mut commands = vec![];
    let _remaining_at_start = data.len();
    let mut counter = 0;
    loop {
        let curr_command = Commands::from_bytes(data);
        match curr_command {
            Commands::EndOfSoundData => {
                commands.push(curr_command);
                break;
            },
            _ => commands.push(curr_command),
        }
        //println!("curr pos: {:8X?}", remaining_at_start - data.len());
        //break;
        counter += 1;
    }
    println!("counter: {}", counter);

    commands
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
            Err(val) => {
                println!("Unknown command: {:2X?}", val);
                break;
            },
        }
    }

    commands
}

pub fn write_commands(buffer: &mut BytesMut, commands: &Vec<Commands>) {
    for cmd in commands {
        buffer.put(&cmd.clone().to_bytes()[..]);
    }
}

impl Commands {
    pub fn to_bytes(self) -> Vec<u8> {
        match self {
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
                data_type,
                data_size,
                data,
            } => {
                let mut out_data: Vec<u8> = vec![data_type];
                out_data.extend(data_size.to_le_bytes());
                out_data.extend(data);
                out_data
            },
            Commands::PCMRAMWrite { offset: _, data: _ } => {
                panic!("not implemented")
            },

            Commands::WaitNSamplesPlus1 { n } => vec![0x70 + n],

            Commands::YM2612Port0Address2AWriteWait { n } => vec![0x80 + n],

            Commands::DACStreamControlWrite {
                register: _,
                value: _,
            } => {
                panic!("not implemented")
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
        }
    }

    pub fn from_bytes(bytes: &mut Bytes) -> Commands {
        let command_val = bytes.get_u8();
        

        match command_val {
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
                // handle data block command
                // skip compatibility arg (0x66)
                bytes.get_u8();
                let data_type = bytes.get_u8();
                let data_size = bytes.get_u32_le();
                Commands::DataBlock {
                    data_type,
                    data_size,
                    data: (0..data_size as usize)
                        .map(|_| bytes.get_u8())
                        .collect(),
                }
            },
            0x68 => {
                // handle PCM RAM write command
                // TODO: not done
                Commands::PCMRAMWrite {
                    offset: 0,
                    data: vec![],
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
                // handle DAC Stream Control Write command
                // TODO: not done
                Commands::DACStreamControlWrite {
                    register: 0,
                    value: 0,
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
                println!("UNK instruction: {:02X?}", command_val);
                panic!("unk instruction")
            },
        }
    }

    pub fn from_bytes_safe(bytes: &mut Bytes) -> Result<Commands, u8> {
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
                // handle data block command
                // handle data block command
                // skip compatibility arg (0x66)
                bytes.get_u8();
                let data_type = bytes.get_u8();
                let data_size = bytes.get_u32_le();
                Commands::DataBlock {
                    data_type,
                    data_size,
                    data: (0..data_size as usize)
                        .map(|_| bytes.get_u8())
                        .collect(),
                }
            },
            0x68 => {
                // handle PCM RAM write command
                // TODO: not done
                Commands::PCMRAMWrite {
                    offset: 0,
                    data: vec![],
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
                // handle DAC Stream Control Write command
                // TODO: not done
                Commands::DACStreamControlWrite {
                    register: 0,
                    value: 0,
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
                println!("UNK instruction: {:02X?}", command_val);
                //panic!("unk instruction")
                return Err(command_val);
            },
        };

        Ok(command)
    }
}
