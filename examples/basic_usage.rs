use std::{
    collections::{HashMap, HashSet},
    fs,
};

use bytes::BytesMut;
use vgm_parser::*;

fn main() {
    // Set up file paths
    let input_file = "contact.vgm";
    let vgm_path = format!("./vgm_files/{}", input_file);

    // Check if the file exists
    if !std::path::Path::new(&vgm_path).exists() {
        println!(
            "VGM file '{}' not found. Please ensure it exists in ./vgm_files/",
            input_file
        );
        return;
    }

    println!("Parsing VGM file: {}", input_file);

    // Parse the VGM file
    let vgm_file = VgmFile::from_path(&vgm_path);

    println!("Parsed successfully!");
    println!("VGM version: {}", vgm_file.header.version);
    println!("Number of commands: {}", vgm_file.commands.len());

    // Create output directory if it doesn't exist
    fs::create_dir_all("./generated").unwrap_or_else(|e| {
        eprintln!("Warning: Could not create generated directory: {}", e);
    });

    // 1. Export commands to JSON
    if let Ok(json_data) = serde_json::to_string_pretty(&vgm_file.commands) {
        let json_path = format!("./generated/commands_{}.json", input_file);
        if let Err(e) = fs::write(&json_path, json_data) {
            eprintln!("Warning: Could not write JSON file: {}", e);
        } else {
            println!("Commands exported to: {}", json_path);
        }
    }

    // 2. Test round-trip binary serialization
    let mut out_buffer = BytesMut::new();
    vgm_file.to_bytes(&mut out_buffer);

    let binary_path = format!("./generated/gen_{}.bin", input_file);
    if let Err(e) = fs::write(&binary_path, &out_buffer) {
        eprintln!("Warning: Could not write binary file: {}", e);
    } else {
        println!("Binary data written to: {}", binary_path);
    }

    // 3. Analyze unique instruction set
    let mut instructions_set: HashSet<Commands> = HashSet::new();
    for cmd in &vgm_file.commands {
        instructions_set.insert(cmd.clone());
    }
    println!("Unique command types found: {}", instructions_set.len());

    // 4. Analyze YM2608 register usage (example chip analysis)
    let mut register_tracker: HashMap<u8, u32> = HashMap::new();
    let mut ym2608_commands = 0;

    for cmd in &vgm_file.commands {
        match cmd {
            Commands::YM2608Port0Write { register, value: _ } => {
                ym2608_commands += 1;
                *register_tracker.entry(*register).or_insert(0) += 1;
            },
            Commands::YM2608Port1Write { register, value: _ } => {
                ym2608_commands += 1;
                *register_tracker.entry(*register).or_insert(0) += 1;
            },
            _ => (),
        }
    }

    if ym2608_commands > 0 {
        println!("YM2608 commands found: {}", ym2608_commands);
        println!("Unique registers used: {}", register_tracker.len());

        // Export register usage to JSON
        if let Ok(register_json) = serde_json::to_string_pretty(&register_tracker) {
            let register_path = format!("./generated/registers_{}.json", input_file);
            if let Err(e) = fs::write(&register_path, register_json) {
                eprintln!("Warning: Could not write register analysis: {}", e);
            } else {
                println!("Register usage exported to: {}", register_path);
            }
        }
    } else {
        println!("No YM2608 commands found in this file");
    }

    // 5. Basic file analysis complete

    // 6. Basic metadata display
    println!("\nMetadata:");
    println!("English track: {}", vgm_file.metadata.english_data.track);
    println!("English game: {}", vgm_file.metadata.english_data.game);
    println!("English system: {}", vgm_file.metadata.english_data.system);
    println!("English author: {}", vgm_file.metadata.english_data.author);
    println!("Release date: {}", vgm_file.metadata.date_release);
    println!("VGM creator: {}", vgm_file.metadata.name_vgm_creator);

    println!("\nExample completed successfully!");
}
