#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::{
    collections::{HashMap, HashSet},
    fs,
};

use bytes::{Buf, Bytes, BytesMut};
//use flate2::{Decompress, bufread::GzDecoder};
use header::HeaderData;
use metadata::VgmMetadata;
use traits::{VgmParser, VgmWriter};
use vgm_commands::{parse_commands, write_commands, Commands};

mod errors;
mod utils;

mod systems;
mod vgm_commands;

mod header;
mod metadata;
mod traits;

mod custom_encoder;

mod tokenizing;

use serde::{Deserialize, Serialize};

use crate::custom_encoder::CustomEncode;

//const FILENAME: &'static str = "prologue.vgm"; //"./vgm_files/01 - Title Screen.vgz"; //prologue.vgm";
const FILENAME: &'static str = "contact.vgm";

#[derive(Debug, Serialize, Deserialize)]
pub struct VgmFile {
    pub header: HeaderData,
    pub commands: Vec<Commands>,
    pub metadata: VgmMetadata,
}

impl VgmFile {
    pub fn from_path(path: &str) -> Self {
        let file_data = fs::read(path).unwrap();
        let mut data = Bytes::from(file_data);
        let vgm_file = VgmFile::from_bytes(&mut data);

        return vgm_file;
    }

    pub fn has_data_block(&self) -> bool {
        for cmd in &self.commands {
            match cmd {
                Commands::DataBlock {
                    data_type: _,
                    data_size: _,
                    data: _,
                } => {
                    return true;
                }
                _ => (),
            }
        }

        return false;
    }

    pub fn has_pcm_write(&self) -> bool {
        for cmd in &self.commands {
            match cmd {
                Commands::PCMRAMWrite { offset: _, data: _ } => {
                    return true;
                }
                _ => (),
            }
        }

        return false;
    }
}

impl VgmParser for VgmFile {
    fn from_bytes(data: &mut Bytes) -> Self {
        let len_data = data.len();
        let header_data = HeaderData::from_bytes(data);
        let vgm_start_pos = header_data.vgm_data_offset as usize + 0x34;

        while len_data - data.len() < vgm_start_pos {
            data.get_u8();
        }

        return VgmFile {
            header: header_data, //HeaderData::from_bytes(data),
            commands: parse_commands(data),
            metadata: VgmMetadata::from_bytes(data),
        };
    }
}

impl VgmWriter for VgmFile {
    fn to_bytes(&self, buffer: &mut bytes::BytesMut) {
        self.header.to_bytes(buffer);
        write_commands(buffer, &self.commands);
        self.metadata.to_bytes(buffer);
    }
}

fn main() {
    let vgm_file = VgmFile::from_path(&format!("./vgm_files/{}", FILENAME));

    fs::write(
        format!("./generated/commands_{}.json", FILENAME),
        serde_json::to_string(&vgm_file.commands).unwrap(),
    )
    .unwrap();
    println!("parsed successfully");

    let mut out_buffer = BytesMut::new();
    vgm_file.to_bytes(&mut out_buffer);

    fs::write(format!("./generated/gen_{}.bin", FILENAME), out_buffer).unwrap();

    let mut instructions_set: HashSet<Commands> = HashSet::new();
    for cmd in &vgm_file.commands {
        instructions_set.insert(cmd.to_owned());
    }
    println!("how many tokens: {}", instructions_set.len());

    let mut register_tracker: HashMap<u8, u32> = HashMap::new();
    for cmd in &vgm_file.commands {
        match cmd {
            
            Commands::YM2608Port0Write { register, value: _ } => {
                if let Some(val) = register_tracker.get_mut(register) {
                    *val += 1;
                } else {
                    register_tracker.insert(*register, 1);
                }
            }
            
            Commands::YM2608Port1Write { register, value: _ } => {
                if let Some(val) = register_tracker.get_mut(register) {
                    *val += 1;
                } else {
                    register_tracker.insert(*register, 1);
                }
            }
            _ => (),
        }
    }

    fs::write(
        format!("./generated/registers_{}.json", FILENAME),
        serde_json::to_string(&register_tracker).unwrap(),
    )
    .unwrap();

    let mut custom_buffer = BytesMut::new();
    for cmd in vgm_file.commands {
        cmd.encode(&mut custom_buffer);
    }

    fs::write(
        format!("./generated/custom_{}.bin", FILENAME),
        custom_buffer,
    )
    .unwrap();
}
