//! VGM Commands Module
//!
//! This module contains all VGM command parsing, serialization, and data block processing.
//! Previously a single 4,500+ line file, now organized into logical submodules.

pub mod commands;
pub mod compression;
pub mod data_blocks;
pub mod parser;
pub mod parsing;
pub mod serialization;

#[cfg(test)]
mod tests;

// Re-export main public types for API compatibility
pub use commands::Commands;
pub use data_blocks::{
    CompressionType, DataBlockContent, RAMWriteChipType, ROMDumpChipType, StreamChipType,
};
pub use parser::{parse_commands, parse_commands_safe, parse_commands_with_config, write_commands};

// Re-export parsing configuration
pub use crate::parser_config::ParserConfig;

// Constants
pub const MAX_DATA_BLOCK_SIZE: u32 = 16 * 1024 * 1024; // 16MB limit
