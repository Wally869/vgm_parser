#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::{
    collections::{HashMap, HashSet},
    fs,
};

use bytes::BytesMut;

mod errors;
mod utils;
mod systems;
mod vgm_commands;
mod header;
mod metadata;
mod traits;
mod custom_encoder;
mod tokenizing;

use vgm_parser::{custom_encoder::CustomEncode, *};

//const FILENAME: &'static str = "prologue.vgm"; //"./vgm_files/01 - Title Screen.vgz"; //prologue.vgm";
const FILENAME: &'static str = "contact.vgm";

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
