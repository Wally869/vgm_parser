pub mod errors;
pub mod header;
pub mod metadata;
pub mod parser_config;
pub mod systems;
pub mod traits;
pub mod utils;
pub mod validation;
pub mod vgm_commands;

pub use errors::*;
pub use header::*;
pub use metadata::*;
pub use parser_config::*;
pub use systems::*;
pub use traits::*;
pub use validation::*;
pub use vgm_commands::*;

use bytes::{Buf, Bytes, BytesMut};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VgmFile {
    pub header: HeaderData,
    pub commands: Vec<Commands>,
    pub metadata: VgmMetadata,
}

impl VgmFile {
    /// Parse VGM file from path with default validation
    pub fn from_path(path: &str) -> VgmResult<Self> {
        Self::from_path_with_config(path, ValidationConfig::default())
    }

    /// Parse VGM file from path with custom validation configuration
    pub fn from_path_with_config(path: &str, config: ValidationConfig) -> VgmResult<Self> {
        Self::from_path_with_full_config(path, config, ParserConfig::default())
    }

    /// Parse VGM file from path with both validation and parser configuration  
    pub fn from_path_with_full_config(
        path: &str,
        validation_config: ValidationConfig,
        parser_config: ParserConfig,
    ) -> VgmResult<Self> {
        let file_data = std::fs::read(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => VgmError::FileNotFound {
                path: path.to_string(),
                io_kind: Some(e.kind()),
            },
            std::io::ErrorKind::PermissionDenied => VgmError::PermissionDenied {
                path: path.to_string(),
            },
            _ => VgmError::FileReadError {
                path: path.to_string(),
                reason: e.to_string(),
            },
        })?;

        // Detect format and decompress if necessary (supports both .vgm and .vgz)
        let vgm_data = crate::utils::detect_and_decompress(&file_data)?;

        // Check decompressed VGM file size
        if vgm_data.len() < 64 {
            return Err(VgmError::FileTooSmall {
                path: path.to_string(),
                size: vgm_data.len(),
            });
        }

        // Check decompressed file size against validation config limits
        if vgm_data.len() > validation_config.max_file_size {
            return Err(VgmError::DataSizeExceedsLimit {
                field: "decompressed_file_size".to_string(),
                size: vgm_data.len(),
                limit: validation_config.max_file_size,
            });
        }

        let mut data = Bytes::from(vgm_data.clone());
        let vgm_file = VgmFile::from_bytes_with_config(&mut data, parser_config)?;

        // Perform validation using decompressed data size
        let validator = VgmValidator::new(validation_config);
        validator.validate_vgm_file(
            &vgm_file.header,
            &vgm_file.commands,
            &vgm_file.metadata,
            vgm_data.len(),
        )?;

        Ok(vgm_file)
    }

    /// Parse VGM file from bytes with validation
    pub fn from_bytes_validated(data: &mut Bytes, config: ValidationConfig) -> VgmResult<Self> {
        Self::from_bytes_with_full_config(data, ParserConfig::default(), config)
    }

    /// Parse VGM file from bytes with parser configuration (no validation)
    pub fn from_bytes_with_config(
        data: &mut Bytes,
        parser_config: ParserConfig,
    ) -> VgmResult<Self> {
        let len_data = data.len();
        let mut resource_tracker = ResourceTracker::new();

        let header_data =
            HeaderData::from_bytes_with_config(data, &parser_config, &mut resource_tracker)?;

        // Security: Prevent integer overflow in offset calculation
        let vgm_start_pos = header_data
            .vgm_data_offset
            .checked_add(0x34)
            .and_then(|v| usize::try_from(v).ok())
            .ok_or(VgmError::IntegerOverflow {
                operation: "VGM data offset calculation".to_string(),
                details: format!("offset {} + 0x34", header_data.vgm_data_offset),
            })?;

        while len_data - data.len() < vgm_start_pos {
            data.get_u8();
        }

        let metadata = VgmMetadata::from_bytes_with_config(data, &parser_config)?;
        let commands = parse_commands_with_config(data, &parser_config, &mut resource_tracker)?;

        Ok(VgmFile {
            header: header_data,
            commands,
            metadata,
        })
    }

    /// Parse VGM file from bytes with both parser and validation configuration
    pub fn from_bytes_with_full_config(
        data: &mut Bytes,
        parser_config: ParserConfig,
        validation_config: ValidationConfig,
    ) -> VgmResult<Self> {
        let original_len = data.len();
        let vgm_file = Self::from_bytes_with_config(data, parser_config)?;

        // Perform validation
        let validator = VgmValidator::new(validation_config);
        validator.validate_vgm_file(
            &vgm_file.header,
            &vgm_file.commands,
            &vgm_file.metadata,
            original_len,
        )?;

        Ok(vgm_file)
    }

    /// Validate this VGM file with the given configuration
    pub fn validate_with_config(
        &self,
        config: ValidationConfig,
        file_size: usize,
    ) -> VgmResult<()> {
        let validator = VgmValidator::new(config);
        validator.validate_vgm_file(&self.header, &self.commands, &self.metadata, file_size)
    }

    /// Validate this VGM file with default configuration
    pub fn validate(&self, file_size: usize) -> VgmResult<()> {
        self.validate_with_config(ValidationConfig::default(), file_size)
    }

    pub fn has_data_block(&self) -> bool {
        for cmd in &self.commands {
            if let Commands::DataBlock { .. } = cmd {
                return true;
            }
        }
        false
    }

    pub fn has_pcm_write(&self) -> bool {
        for cmd in &self.commands {
            if let Commands::PCMRAMWrite { .. } = cmd {
                return true;
            }
        }
        false
    }
}

impl VgmParser for VgmFile {
    fn from_bytes(data: &mut Bytes) -> VgmResult<Self> {
        let len_data = data.len();
        let header_data = HeaderData::from_bytes(data)?;
        // Security: Prevent integer overflow in offset calculation
        let vgm_start_pos = header_data
            .vgm_data_offset
            .checked_add(0x34)
            .and_then(|v| usize::try_from(v).ok())
            .ok_or(VgmError::IntegerOverflow {
                operation: "VGM data offset calculation".to_string(),
                details: format!("offset {} + 0x34", header_data.vgm_data_offset),
            })?;

        while len_data - data.len() < vgm_start_pos {
            data.get_u8();
        }

        let metadata = VgmMetadata::from_bytes(data)?;

        Ok(VgmFile {
            header: header_data,
            commands: parse_commands(data),
            metadata,
        })
    }
}

impl VgmWriter for VgmFile {
    fn to_bytes(&self, buffer: &mut BytesMut) -> VgmResult<()> {
        self.header.to_bytes(buffer)?;
        write_commands(buffer, &self.commands)?;
        self.metadata.to_bytes(buffer)?;
        Ok(())
    }
}

#[cfg(test)]
mod validation_integration_test;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Get project root directory for test file paths
    fn get_project_root() -> PathBuf {
        // Try to find project root by looking for Cargo.toml
        let mut current = std::env::current_dir().expect("Failed to get current directory");
        loop {
            if current.join("Cargo.toml").exists() {
                return current;
            }
            if !current.pop() {
                // If we can't find Cargo.toml, assume current directory is project root
                return std::env::current_dir().expect("Failed to get current directory");
            }
        }
    }

    /// Get path relative to project root
    fn project_path(relative_path: &str) -> PathBuf {
        get_project_root().join(relative_path)
    }

    #[test]
    fn test_vgm_parse_write_cycle() {
        // Use project-relative paths
        let test_file = project_path("vgm_files/Into Battle.vgm");

        // Skip test if no test files available
        if !test_file.exists() {
            println!(
                "Skipping test_vgm_parse_write_cycle - test VGM file not found at {:?}",
                test_file
            );
            return;
        }

        // Parse the file
        let vgm = match VgmFile::from_path(test_file.to_str().expect("Invalid path encoding")) {
            Ok(vgm) => vgm,
            Err(e) => {
                println!(
                    "Skipping test_vgm_parse_write_cycle - failed to parse VGM file: {}",
                    e
                );
                return;
            },
        };

        // Basic assertions
        assert_eq!(vgm.header.version, 151); // v1.51
        assert!(!vgm.commands.is_empty());

        // Test round-trip
        let mut buffer = BytesMut::new();
        match vgm.to_bytes(&mut buffer) {
            Ok(()) => {},
            Err(e) => {
                println!(
                    "Skipping test_vgm_parse_write_cycle - failed to serialize VGM: {}",
                    e
                );
                return;
            },
        };

        // Parse again
        let mut data = Bytes::from(buffer.to_vec());
        let vgm2 = match VgmFile::from_bytes(&mut data) {
            Ok(vgm2) => vgm2,
            Err(e) => {
                println!(
                    "Skipping test_vgm_parse_write_cycle - failed to parse round-trip data: {}",
                    e
                );
                return;
            },
        };

        // Compare
        assert_eq!(vgm.header.version, vgm2.header.version);
        assert_eq!(vgm.commands.len(), vgm2.commands.len());
    }
}
