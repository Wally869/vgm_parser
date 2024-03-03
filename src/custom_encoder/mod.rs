use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::vgm_commands::Commands;

pub trait CustomEncode {
    fn encode(&self, buffer: &mut BytesMut); // -> Vec<u8>;
}

pub trait CustomDecode {
    fn decode(data: &mut Bytes) -> Self;
}

impl CustomDecode for Commands {
    fn decode(data: &mut Bytes) -> Self {
        let instruction = data.get_u8();
        match instruction {
            0x01 => {
                // read port
                match data.get_u8() {
                    0x01 => {
                        return Commands::YM2608Port0Write {
                            register: data.get_u8(),
                            value: data.get_u8(),
                        };
                    }
                    0x02 => {
                        return Commands::YM2608Port1Write {
                            register: data.get_u8(),
                            value: data.get_u8(),
                        };
                    }
                    _ => panic!("never"),
                }
            }
            0x02 => {
                return Commands::WaitNSamples {
                    n: data.get_u16_le(),
                };
            }
            0x03 => {
                return Commands::EndOfSoundData;
            }
            _ => panic!("never"),
        }
    }
}

impl CustomEncode for Commands {
    fn encode(&self, buffer: &mut BytesMut) {
        match self {
            // match all waits to single type instruction?
            // with wait as 0x02
            Commands::Wait735Samples => {
                buffer.put_u8(0x02);
                buffer.put_u16_le(735);
            }
            Commands::Wait882Samples => {
                buffer.put_u8(0x02);
                buffer.put_u16_le(882);
            }
            Commands::WaitNSamples { n } => {
                buffer.put_u8(0x02);
                buffer.put_u16_le(*n);
            }
            Commands::WaitNSamplesPlus1 { n } => {
                buffer.put_u8(0x02);
                buffer.put_u16_le((n + 1) as u16);
            }
            Commands::YM2608Port0Write { register, value } => {
                buffer.put_u8(0x01);
                buffer.put_u8(0x01);
                buffer.put_u8(register.to_owned());
                buffer.put_u8(value.to_owned());
            }
            Commands::YM2608Port1Write { register, value } => {
                buffer.put_u8(0x01);
                buffer.put_u8(0x02);
                buffer.put_u8(register.to_owned());
                buffer.put_u8(value.to_owned());
            },
            Commands::EndOfSoundData => {
                buffer.put_u8(0x03);
            }
            _ => panic!("unsupported"),
        }
    }
}
