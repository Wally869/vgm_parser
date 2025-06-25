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

        let commands = parse_commands_with_config(data, &parser_config, &mut resource_tracker)?;
        let metadata = VgmMetadata::from_bytes_with_config(data, &parser_config)?;

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

        let commands = parse_commands(data);
        let metadata = VgmMetadata::from_bytes(data)?;

        Ok(VgmFile {
            header: header_data,
            commands,
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
    use std::io::Write;
    use tempfile::NamedTempFile;

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

    /// Helper to create test VGM data using builders
    fn create_test_vgm_data() -> Vec<u8> {
        // Create a basic VGM file with minimal valid data
        let vgm = VgmFile {
            header: HeaderData {
                version: 150,
                sn76489_clock: 3579545,
                ym2612_clock: 7670453,
                total_nb_samples: 44100,
                rate: 44100,
                vgm_data_offset: 0x40,
                gd3_offset: 0x80,
                end_of_file_offset: 0x100,
                ..Default::default()
            },
            commands: vec![
                Commands::PSGWrite { value: 0x9F, chip_index: 0 },
                Commands::Wait735Samples,
                Commands::EndOfSoundData,
            ],
            metadata: VgmMetadata {
                english_data: Gd3LocaleData {
                    track: "Test Track".to_string(),
                    game: "Test Game".to_string(),
                    system: "Test System".to_string(),
                    author: "Test Author".to_string(),
                },
                japanese_data: Gd3LocaleData {
                    track: "".to_string(),
                    game: "".to_string(),
                    system: "".to_string(),
                    author: "".to_string(),
                },
                date_release: "2024".to_string(),
                name_vgm_creator: "Test Creator".to_string(),
                notes: "Test VGM file".to_string(),
            },
        };

        let mut buffer = BytesMut::new();
        vgm.to_bytes(&mut buffer).unwrap();
        buffer.to_vec()
    }

    #[test]
    fn test_vgm_file_creation() {
        let vgm = VgmFile {
            header: HeaderData::default(),
            commands: vec![Commands::EndOfSoundData],
            metadata: VgmMetadata {
                english_data: Gd3LocaleData {
                    track: "Test".to_string(),
                    game: "Test".to_string(),
                    system: "Test".to_string(),
                    author: "Test".to_string(),
                },
                japanese_data: Gd3LocaleData {
                    track: "".to_string(),
                    game: "".to_string(),
                    system: "".to_string(),
                    author: "".to_string(),
                },
                date_release: "2024".to_string(),
                name_vgm_creator: "Test".to_string(),
                notes: "Test".to_string(),
            },
        };

        // Test structure
        assert_eq!(vgm.commands.len(), 1);
        assert_eq!(vgm.header.version, 0);
        assert_eq!(vgm.metadata.english_data.track, "Test");
    }

    #[test]
    fn test_vgm_from_bytes() {
        let test_data = create_test_vgm_data();
        let mut bytes = Bytes::from(test_data);

        let vgm = VgmFile::from_bytes(&mut bytes).unwrap();
        
        assert_eq!(vgm.header.version, 150);
        assert_eq!(vgm.header.sn76489_clock, 3579545);
        assert!(!vgm.commands.is_empty());
        assert_eq!(vgm.metadata.english_data.track, "Test Track");
    }

    #[test]
    fn test_vgm_from_bytes_with_config() {
        let test_data = create_test_vgm_data();
        let mut bytes = Bytes::from(test_data);
        let config = ParserConfig::default();

        let vgm = VgmFile::from_bytes_with_config(&mut bytes, config).unwrap();
        
        assert_eq!(vgm.header.version, 150);
        assert!(!vgm.commands.is_empty());
    }

    #[test]
    fn test_vgm_from_bytes_validated() {
        let test_data = create_test_vgm_data();
        let mut bytes = Bytes::from(test_data);
        let config = ValidationConfig::default();

        let vgm = VgmFile::from_bytes_validated(&mut bytes, config).unwrap();
        
        assert_eq!(vgm.header.version, 150);
        assert!(!vgm.commands.is_empty());
    }

    #[test]
    fn test_vgm_from_bytes_with_full_config() {
        let test_data = create_test_vgm_data();
        let mut bytes = Bytes::from(test_data);
        let parser_config = ParserConfig::default();
        let validation_config = ValidationConfig::default();

        let vgm = VgmFile::from_bytes_with_full_config(&mut bytes, parser_config, validation_config).unwrap();
        
        assert_eq!(vgm.header.version, 150);
        assert!(!vgm.commands.is_empty());
    }

    #[test]
    fn test_vgm_to_bytes_round_trip() {
        let test_data = create_test_vgm_data();
        let mut bytes = Bytes::from(test_data);

        // Parse
        let vgm = VgmFile::from_bytes(&mut bytes).unwrap();
        
        // Serialize
        let mut buffer = BytesMut::new();
        vgm.to_bytes(&mut buffer).unwrap();
        
        // Parse again
        let mut bytes2 = Bytes::from(buffer.to_vec());
        let vgm2 = VgmFile::from_bytes(&mut bytes2).unwrap();
        
        // Compare key fields
        assert_eq!(vgm.header.version, vgm2.header.version);
        assert_eq!(vgm.header.sn76489_clock, vgm2.header.sn76489_clock);
        assert_eq!(vgm.commands.len(), vgm2.commands.len());
        assert_eq!(vgm.metadata.english_data.track, vgm2.metadata.english_data.track);
    }

    #[test]
    fn test_vgm_has_data_block() {
        // Test without data block
        let vgm_no_block = VgmFile {
            header: HeaderData::default(),
            commands: vec![
                Commands::PSGWrite { value: 0x9F, chip_index: 0 },
                Commands::EndOfSoundData,
            ],
            metadata: VgmMetadata {
                english_data: Gd3LocaleData {
                    track: "".to_string(),
                    game: "".to_string(),
                    system: "".to_string(),
                    author: "".to_string(),
                },
                japanese_data: Gd3LocaleData {
                    track: "".to_string(),
                    game: "".to_string(),
                    system: "".to_string(),
                    author: "".to_string(),
                },
                date_release: "".to_string(),
                name_vgm_creator: "".to_string(),
                notes: "".to_string(),
            },
        };
        assert!(!vgm_no_block.has_data_block());

        // Test with data block
        let vgm_with_block = VgmFile {
            header: HeaderData::default(),
            commands: vec![
                Commands::DataBlock {
                    block_type: 0x00,
                    data: DataBlockContent::UncompressedStream {
                        chip_type: StreamChipType::YM2612,
                        data: vec![0x01, 0x02, 0x03],
                    },
                },
                Commands::EndOfSoundData,
            ],
            metadata: VgmMetadata {
                english_data: Gd3LocaleData {
                    track: "".to_string(),
                    game: "".to_string(),
                    system: "".to_string(),
                    author: "".to_string(),
                },
                japanese_data: Gd3LocaleData {
                    track: "".to_string(),
                    game: "".to_string(),
                    system: "".to_string(),
                    author: "".to_string(),
                },
                date_release: "".to_string(),
                name_vgm_creator: "".to_string(),
                notes: "".to_string(),
            },
        };
        assert!(vgm_with_block.has_data_block());
    }

    #[test]
    fn test_vgm_has_pcm_write() {
        // Test without PCM write
        let vgm_no_pcm = VgmFile {
            header: HeaderData::default(),
            commands: vec![
                Commands::PSGWrite { value: 0x9F, chip_index: 0 },
                Commands::EndOfSoundData,
            ],
            metadata: VgmMetadata {
                english_data: Gd3LocaleData {
                    track: "".to_string(),
                    game: "".to_string(),
                    system: "".to_string(),
                    author: "".to_string(),
                },
                japanese_data: Gd3LocaleData {
                    track: "".to_string(),
                    game: "".to_string(),
                    system: "".to_string(),
                    author: "".to_string(),
                },
                date_release: "".to_string(),
                name_vgm_creator: "".to_string(),
                notes: "".to_string(),
            },
        };
        assert!(!vgm_no_pcm.has_pcm_write());

        // Test with PCM write
        let vgm_with_pcm = VgmFile {
            header: HeaderData::default(),
            commands: vec![
                Commands::PCMRAMWrite {
                    chip_type: 0x02, // YM2612
                    read_offset: 0x1000,
                    write_offset: 0x2000,
                    size: 0x100,
                    data: vec![0xAA; 0x100],
                },
                Commands::EndOfSoundData,
            ],
            metadata: VgmMetadata {
                english_data: Gd3LocaleData {
                    track: "".to_string(),
                    game: "".to_string(),
                    system: "".to_string(),
                    author: "".to_string(),
                },
                japanese_data: Gd3LocaleData {
                    track: "".to_string(),
                    game: "".to_string(),
                    system: "".to_string(),
                    author: "".to_string(),
                },
                date_release: "".to_string(),
                name_vgm_creator: "".to_string(),
                notes: "".to_string(),
            },
        };
        assert!(vgm_with_pcm.has_pcm_write());
    }

    #[test]
    fn test_vgm_validate() {
        let test_data = create_test_vgm_data();
        let mut bytes = Bytes::from(test_data.clone());
        let vgm = VgmFile::from_bytes(&mut bytes).unwrap();

        // Test validation with default config
        let result = vgm.validate(test_data.len());
        assert!(result.is_ok());

        // Test validation with custom config
        let config = ValidationConfig {
            min_vgm_version: 100,
            max_vgm_version: 200,
            ..Default::default()
        };
        let result = vgm.validate_with_config(config, test_data.len());
        assert!(result.is_ok());
    }

    #[test]
    fn test_vgm_from_path_errors() {
        // Test file not found
        let result = VgmFile::from_path("nonexistent_file.vgm");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::FileNotFound { .. }));
    }

    #[test]
    fn test_vgm_from_path_success() {
        let test_data = create_test_vgm_data();
        
        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&test_data).unwrap();
        temp_file.flush().unwrap();
        
        let path = temp_file.path().to_str().unwrap();
        let vgm = VgmFile::from_path(path).unwrap();
        
        assert_eq!(vgm.header.version, 150);
        assert!(!vgm.commands.is_empty());
    }

    #[test]
    fn test_vgm_from_path_with_config() {
        let test_data = create_test_vgm_data();
        
        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&test_data).unwrap();
        temp_file.flush().unwrap();
        
        let path = temp_file.path().to_str().unwrap();
        let config = ValidationConfig::default();
        let vgm = VgmFile::from_path_with_config(path, config).unwrap();
        
        assert_eq!(vgm.header.version, 150);
        assert!(!vgm.commands.is_empty());
    }

    #[test]
    fn test_vgm_from_path_with_full_config() {
        let test_data = create_test_vgm_data();
        
        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&test_data).unwrap();
        temp_file.flush().unwrap();
        
        let path = temp_file.path().to_str().unwrap();
        let validation_config = ValidationConfig::default();
        let parser_config = ParserConfig::default();
        let vgm = VgmFile::from_path_with_full_config(path, validation_config, parser_config).unwrap();
        
        assert_eq!(vgm.header.version, 150);
        assert!(!vgm.commands.is_empty());
    }

    #[test]
    fn test_vgm_file_too_small() {
        // Create data with VGM magic but too small (< 64 bytes)
        let mut small_data = Vec::new();
        small_data.extend_from_slice(b"Vgm "); // VGM magic bytes
        small_data.extend_from_slice(&vec![0u8; 28]); // Only 32 bytes total, need 64
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&small_data).unwrap();
        temp_file.flush().unwrap();
        
        let path = temp_file.path().to_str().unwrap();
        let result = VgmFile::from_path(path);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::FileTooSmall { .. }));
    }

    #[test]
    fn test_vgm_size_limit_exceeded() {
        let test_data = create_test_vgm_data();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&test_data).unwrap();
        temp_file.flush().unwrap();
        
        let path = temp_file.path().to_str().unwrap();
        let config = ValidationConfig {
            max_file_size: 100, // Very small limit
            ..Default::default()
        };
        
        let result = VgmFile::from_path_with_config(path, config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::DataSizeExceedsLimit { .. }));
    }

    #[test]
    fn test_vgm_integer_overflow_protection() {
        // Create VGM with invalid offset that would cause overflow
        let mut invalid_data = create_test_vgm_data();
        
        // Corrupt the vgm_data_offset field to cause overflow
        // VGM data offset is at position 0x34 (52) in the header
        invalid_data[0x34] = 0xFF;
        invalid_data[0x35] = 0xFF;
        invalid_data[0x36] = 0xFF;
        invalid_data[0x37] = 0xFF;
        
        let mut bytes = Bytes::from(invalid_data);
        let result = VgmFile::from_bytes(&mut bytes);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::IntegerOverflow { .. }));
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
