//! VGM Parser Interface Module
//!
//! High-level parsing functions for VGM command streams with error handling,
//! resource tracking, and backward compatibility.

use super::commands::Commands;
use crate::errors::VgmResult;
use crate::{ParserConfig, ResourceTracker};
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// Parse VGM commands with default configuration (backward compatibility)
pub fn parse_commands(data: &mut Bytes) -> Vec<Commands> {
    // Use default parser config for backward compatibility
    let config = ParserConfig::default();
    let mut tracker = ResourceTracker::new();

    match parse_commands_with_config(data, &config, &mut tracker) {
        Ok(commands) => commands,
        Err(e) => {
            eprintln!("Warning: Command parsing failed with error: {}", e);
            vec![] // Return empty commands on error for backward compatibility
        },
    }
}

/// Parse commands with resource tracking and limits
pub fn parse_commands_with_config(
    data: &mut Bytes,
    config: &ParserConfig,
    tracker: &mut ResourceTracker,
) -> VgmResult<Vec<Commands>> {
    let mut commands = Vec::new();
    let _remaining_at_start = data.len();

    loop {
        // Check if we have any data left
        if data.is_empty() {
            break;
        }
        
        // Check command count limit before parsing each command
        tracker.track_command(config)?;

        match Commands::from_bytes_with_config(data, config, tracker) {
            Ok(curr_command) => match curr_command {
                Commands::EndOfSoundData => {
                    commands.push(curr_command);
                    break;
                },
                _ => commands.push(curr_command),
            },
            Err(e) => {
                return Err(e);
            },
        }
    }

    Ok(commands)
}

/// Parse commands with error recovery (safe mode)
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

/// Write commands to byte buffer
pub fn write_commands(buffer: &mut BytesMut, commands: &Vec<Commands>) -> VgmResult<()> {
    for cmd in commands {
        let cmd_bytes = cmd.clone().to_bytes()?;
        buffer.put(&cmd_bytes[..]);
    }
    Ok(())
}
