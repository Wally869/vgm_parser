//! VGM Test Data Generators and Builders
//!
//! This module provides builder pattern implementations for generating VGM test data.
//! Supports all VGM versions, edge cases, and invalid data for comprehensive testing.

use bytes::BytesMut;
use vgm_parser::{
    Commands, CompressionType, DataBlockContent, Gd3LocaleData, HeaderData, StreamChipType,
    VgmFile, VgmMetadata, VgmResult, VgmWriter,
};

/// Main builder for creating VGM test files with fluent API
#[derive(Debug)]
pub struct VgmBuilder {
    header: HeaderBuilder,
    commands: CommandsBuilder,
    metadata: MetadataBuilder,
}

impl Default for VgmBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl VgmBuilder {
    /// Create a new VGM builder with sensible defaults
    pub fn new() -> Self {
        Self {
            header: HeaderBuilder::default(),
            commands: CommandsBuilder::default(),
            metadata: MetadataBuilder::default(),
        }
    }

    /// Set VGM version (affects available features)
    pub fn version(mut self, version: u32) -> Self {
        self.header = self.header.version(version);
        self
    }

    /// Configure header settings
    pub fn header<F>(mut self, f: F) -> Self
    where
        F: FnOnce(HeaderBuilder) -> HeaderBuilder,
    {
        self.header = f(self.header);
        self
    }

    /// Configure commands
    pub fn commands<F>(mut self, f: F) -> Self
    where
        F: FnOnce(CommandsBuilder) -> CommandsBuilder,
    {
        self.commands = f(self.commands);
        self
    }

    /// Configure metadata
    pub fn metadata<F>(mut self, f: F) -> Self
    where
        F: FnOnce(MetadataBuilder) -> MetadataBuilder,
    {
        self.metadata = f(self.metadata);
        self
    }

    /// Build the VGM file with proper offset calculations
    pub fn build(self) -> VgmResult<VgmFile> {
        let mut header = self.header.build()?;
        let commands = self.commands.build()?;
        let metadata = self.metadata.build()?;

        // Calculate proper offsets by serializing components
        let mut temp_buffer = BytesMut::new();
        
        // Serialize header to get its size
        header.to_bytes(&mut temp_buffer)?;
        let header_size = temp_buffer.len();
        
        // Serialize commands to get their size  
        temp_buffer.clear();
        use vgm_parser::write_commands;
        write_commands(&mut temp_buffer, &commands)?;
        let commands_size = temp_buffer.len();
        
        // Serialize metadata to get its size
        temp_buffer.clear();
        metadata.to_bytes(&mut temp_buffer)?;
        let metadata_size = temp_buffer.len();
        
        // Calculate offsets
        // GD3 offset: position where GD3 starts = header size + commands size
        header.gd3_offset = (header_size + commands_size) as u32;
        
        // End of file offset: total file size - 4 (because it's relative to offset 0x04)
        // Total size = header + commands + metadata
        let total_size = header_size + commands_size + metadata_size;
        header.end_of_file_offset = (total_size - 4) as u32;

        Ok(VgmFile {
            header,
            commands,
            metadata,
        })
    }

    /// Build as bytes (useful for testing serialization)
    pub fn build_bytes(self) -> VgmResult<Vec<u8>> {
        let vgm_file = self.build()?;
        let mut buffer = BytesMut::new();
        vgm_file.to_bytes(&mut buffer)?;
        Ok(buffer.to_vec())
    }
}

/// Builder for VGM headers with version-specific defaults
#[derive(Debug)]
pub struct HeaderBuilder {
    data: HeaderData,
}

impl Default for HeaderBuilder {
    fn default() -> Self {
        Self {
            data: HeaderData {
                version: 150, // VGM 1.50 - most common
                sn76489_clock: 3579545, // Standard NTSC PSG clock
                ym2612_clock: 7670453,  // Standard NTSC YM2612 clock
                vgm_data_offset: 0x40,  // Standard offset for v1.50+
                total_nb_samples: 44100, // 1 second at 44.1kHz
                rate: 44100,
                ..Default::default()
            },
        }
    }
}

impl HeaderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set VGM version with appropriate defaults
    pub fn version(mut self, version: u32) -> Self {
        self.data.version = version;
        
        // Set version-specific defaults
        match version {
            100 => {
                // VGM 1.00 - basic PSG only
                self.data.vgm_data_offset = 0x40;
                self.data.ym2612_clock = 0; // No YM2612 in v1.00
            }
            101 => {
                // VGM 1.01 - added YM2612
                self.data.vgm_data_offset = 0x40;
            }
            110 => {
                // VGM 1.10 - added loop support
                self.data.vgm_data_offset = 0x40;
            }
            150 => {
                // VGM 1.50 - added many chips
                self.data.vgm_data_offset = 0x40;
            }
            151 => {
                // VGM 1.51 - added more chips
                self.data.vgm_data_offset = 0x40;
            }
            161 => {
                // VGM 1.61 - expanded header
                self.data.vgm_data_offset = 0x80;
            }
            170 => {
                // VGM 1.70 - dual chip support
                self.data.vgm_data_offset = 0x100;
            }
            171 => {
                // VGM 1.71 - latest
                self.data.vgm_data_offset = 0x100;
            }
            _ => {
                // Default to 1.50 behavior
                self.data.vgm_data_offset = 0x40;
            }
        }
        
        self
    }

    /// Enable PSG with clock speed
    pub fn psg_clock(mut self, clock: u32) -> Self {
        self.data.sn76489_clock = clock;
        self
    }

    /// Enable YM2612 with clock speed
    pub fn ym2612_clock(mut self, clock: u32) -> Self {
        self.data.ym2612_clock = clock;
        self
    }

    /// Enable YM2151 with clock speed
    pub fn ym2151_clock(mut self, clock: u32) -> Self {
        self.data.ym2151_clock = clock;
        self
    }

    /// Set total number of samples
    pub fn total_samples(mut self, samples: u32) -> Self {
        self.data.total_nb_samples = samples;
        self
    }

    /// Enable looping
    pub fn with_loop(mut self, loop_offset: u32, loop_samples: u32) -> Self {
        self.data.loop_offset = loop_offset;
        self.data.loop_nb_samples = loop_samples;
        self
    }

    /// Set sample rate
    pub fn sample_rate(mut self, rate: u32) -> Self {
        self.data.rate = rate;
        self
    }

    /// Set GD3 metadata offset
    pub fn gd3_offset(mut self, offset: u32) -> Self {
        self.data.gd3_offset = offset;
        self
    }

    /// Set raw field value (for testing edge cases)
    pub fn raw_field<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut HeaderData),
    {
        f(&mut self.data);
        self
    }

    pub fn build(self) -> VgmResult<HeaderData> {
        Ok(self.data)
    }
}

/// Builder for VGM command sequences
#[derive(Debug, Clone)]
pub struct CommandsBuilder {
    commands: Vec<Commands>,
}

impl Default for CommandsBuilder {
    fn default() -> Self {
        Self {
            commands: vec![Commands::EndOfSoundData], // Always end with this
        }
    }
}

impl CommandsBuilder {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Add a single command
    pub fn add_command(mut self, command: Commands) -> Self {
        // Insert before the end marker if it exists
        if self.commands.last() == Some(&Commands::EndOfSoundData) {
            let len = self.commands.len();
            self.commands.insert(len - 1, command);
        } else {
            self.commands.push(command);
        }
        self
    }

    /// Add multiple commands
    pub fn add_commands(mut self, commands: Vec<Commands>) -> Self {
        for command in commands {
            self = self.add_command(command);
        }
        self
    }

    /// Add PSG write command
    pub fn psg_write(self, value: u8, chip_index: u8) -> Self {
        self.add_command(Commands::PSGWrite { value, chip_index })
    }

    /// Add YM2612 port 0 write
    pub fn ym2612_port0_write(self, register: u8, value: u8, chip_index: u8) -> Self {
        self.add_command(Commands::YM2612Port0Write {
            register,
            value,
            chip_index,
        })
    }

    /// Add YM2612 port 1 write
    pub fn ym2612_port1_write(self, register: u8, value: u8, chip_index: u8) -> Self {
        self.add_command(Commands::YM2612Port1Write {
            register,
            value,
            chip_index,
        })
    }

    /// Add wait command
    pub fn wait_samples(self, n: u16) -> Self {
        self.add_command(Commands::WaitNSamples { n })
    }

    /// Add standard wait (735 samples = 1/60 second)
    pub fn wait_60hz(self) -> Self {
        self.add_command(Commands::Wait735Samples)
    }

    /// Add standard wait (882 samples = 1/50 second)
    pub fn wait_50hz(self) -> Self {
        self.add_command(Commands::Wait882Samples)
    }

    /// Add a data block
    pub fn data_block(self, block_type: u8, data: DataBlockContent) -> Self {
        self.add_command(Commands::DataBlock { block_type, data })
    }

    /// Ensure proper ending
    pub fn with_end(mut self) -> Self {
        if self.commands.last() != Some(&Commands::EndOfSoundData) {
            self.commands.push(Commands::EndOfSoundData);
        }
        self
    }

    /// Generate a simple PSG tone sequence
    pub fn simple_psg_sequence(self) -> Self {
        self.psg_write(0x8F, 0) // Channel 0 frequency latch + low bits (F = frequency value)
            .psg_write(0x90, 0) // Channel 0 volume latch + low volume
            .wait_60hz()
            .psg_write(0x9F, 0) // Channel 0 silence (volume = F)
            .with_end()
    }

    /// Generate a simple YM2612 sequence
    pub fn simple_ym2612_sequence(self) -> Self {
        self.ym2612_port0_write(0x22, 0x00, 0) // LFO off
            .ym2612_port0_write(0x27, 0x00, 0) // CH3 mode
            .ym2612_port0_write(0x28, 0xF0, 0) // Key on channel 1
            .wait_60hz()
            .ym2612_port0_write(0x28, 0x00, 0) // Key off channel 1
            .with_end()
    }

    pub fn build(self) -> VgmResult<Vec<Commands>> {
        Ok(self.commands)
    }
}

/// Builder for VGM metadata (GD3 tags)
#[derive(Debug, Clone)]
pub struct MetadataBuilder {
    english_track: String,
    english_game: String,
    english_system: String,
    english_author: String,
    japanese_track: String,
    japanese_game: String,
    japanese_system: String,
    japanese_author: String,
    date_release: String,
    name_vgm_creator: String,
    notes: String,
}

impl Default for MetadataBuilder {
    fn default() -> Self {
        Self {
            english_track: "Test Track".to_string(),
            english_game: "Test Game".to_string(),
            english_system: "Test System".to_string(),
            english_author: "Test Author".to_string(),
            japanese_track: String::new(),
            japanese_game: String::new(),
            japanese_system: String::new(),
            japanese_author: String::new(),
            date_release: "2024-01-01".to_string(),
            name_vgm_creator: "VGM Test Builder".to_string(),
            notes: "Generated by test builder".to_string(),
        }
    }
}

impl MetadataBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set English track name
    pub fn english_track<S: Into<String>>(mut self, track: S) -> Self {
        self.english_track = track.into();
        self
    }

    /// Set English game name
    pub fn english_game<S: Into<String>>(mut self, game: S) -> Self {
        self.english_game = game.into();
        self
    }

    /// Set English system name
    pub fn english_system<S: Into<String>>(mut self, system: S) -> Self {
        self.english_system = system.into();
        self
    }

    /// Set English author name
    pub fn english_author<S: Into<String>>(mut self, author: S) -> Self {
        self.english_author = author.into();
        self
    }

    /// Set Japanese track name
    pub fn japanese_track<S: Into<String>>(mut self, track: S) -> Self {
        self.japanese_track = track.into();
        self
    }

    /// Set Japanese game name
    pub fn japanese_game<S: Into<String>>(mut self, game: S) -> Self {
        self.japanese_game = game.into();
        self
    }

    /// Set Japanese system name
    pub fn japanese_system<S: Into<String>>(mut self, system: S) -> Self {
        self.japanese_system = system.into();
        self
    }

    /// Set Japanese author name
    pub fn japanese_author<S: Into<String>>(mut self, author: S) -> Self {
        self.japanese_author = author.into();
        self
    }

    /// Set release date
    pub fn release_date<S: Into<String>>(mut self, date: S) -> Self {
        self.date_release = date.into();
        self
    }

    /// Set VGM creator name
    pub fn creator_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name_vgm_creator = name.into();
        self
    }

    /// Set notes
    pub fn notes<S: Into<String>>(mut self, notes: S) -> Self {
        self.notes = notes.into();
        self
    }

    pub fn build(self) -> VgmResult<VgmMetadata> {
        Ok(VgmMetadata {
            english_data: Gd3LocaleData {
                track: self.english_track,
                game: self.english_game,
                system: self.english_system,
                author: self.english_author,
            },
            japanese_data: Gd3LocaleData {
                track: self.japanese_track,
                game: self.japanese_game,
                system: self.japanese_system,
                author: self.japanese_author,
            },
            date_release: self.date_release,
            name_vgm_creator: self.name_vgm_creator,
            notes: self.notes,
        })
    }
}

/// Version-specific VGM generators
pub struct VgmVersionGenerators;

impl VgmVersionGenerators {
    /// Generate VGM 1.00 file (basic PSG only)
    pub fn vgm_v100_basic() -> VgmBuilder {
        VgmBuilder::new()
            .version(100)
            .header(|h| h.psg_clock(3579545).ym2612_clock(0))
            .commands(|c| c.simple_psg_sequence())
            .metadata(|m| {
                m.english_track("VGM 1.00 Test")
                    .english_system("Sega Master System")
            })
    }

    /// Generate VGM 1.01 file (PSG + YM2612)
    pub fn vgm_v101_basic() -> VgmBuilder {
        VgmBuilder::new()
            .version(101)
            .header(|h| h.psg_clock(3579545).ym2612_clock(7670453))
            .commands(|c| c.simple_ym2612_sequence())
            .metadata(|m| {
                m.english_track("VGM 1.01 Test")
                    .english_system("Sega Genesis")
            })
    }

    /// Generate VGM 1.50 file (standard features)
    pub fn vgm_v150_standard() -> VgmBuilder {
        VgmBuilder::new()
            .version(150)
            .header(|h| {
                h.psg_clock(3579545)
                    .ym2612_clock(7670453)
                    .with_loop(0x100, 88200) // 2 second loop
            })
            .commands(|c| {
                c.simple_ym2612_sequence()
                    .wait_samples(1000)
                    .simple_psg_sequence()
            })
            .metadata(|m| {
                m.english_track("VGM 1.50 Test")
                    .english_system("Sega Genesis")
                    .notes("Standard VGM 1.50 with loop")
            })
    }

    /// Generate VGM 1.61 file (expanded header)
    pub fn vgm_v161_expanded() -> VgmBuilder {
        VgmBuilder::new()
            .version(161)
            .header(|h| {
                h.psg_clock(3579545)
                    .ym2612_clock(7670453)
                    .ym2151_clock(3579545)
            })
            .commands(|c| c.simple_ym2612_sequence())
            .metadata(|m| {
                m.english_track("VGM 1.61 Test")
                    .english_system("Multiple Systems")
            })
    }

    /// Generate VGM 1.70 file (dual chip support)
    pub fn vgm_v170_dual_chip() -> VgmBuilder {
        VgmBuilder::new()
            .version(170)
            .header(|h| h.psg_clock(3579545).ym2612_clock(7670453))
            .commands(|c| {
                c.psg_write(0x80, 0) // Chip 0
                    .psg_write(0x80, 1) // Chip 1
                    .ym2612_port0_write(0x28, 0xF0, 0) // Chip 0
                    .ym2612_port0_write(0x28, 0xF0, 1) // Chip 1
                    .with_end()
            })
            .metadata(|m| {
                m.english_track("VGM 1.70 Dual Chip Test")
                    .english_system("Dual Genesis")
            })
    }

    /// Generate VGM 1.71 file (latest)
    pub fn vgm_v171_latest() -> VgmBuilder {
        VgmBuilder::new()
            .version(171)
            .header(|h| {
                h.psg_clock(3579545)
                    .ym2612_clock(7670453)
                    .sample_rate(44100)
            })
            .commands(|c| c.simple_ym2612_sequence())
            .metadata(|m| {
                m.english_track("VGM 1.71 Test")
                    .english_system("Modern System")
                    .japanese_track("VGM 1.71 テスト")
                    .japanese_game("モダンシステム")
            })
    }
}

/// Invalid file generators for error testing
pub struct InvalidVgmGenerators;

impl InvalidVgmGenerators {
    /// Generate file with invalid signature
    pub fn invalid_signature() -> Vec<u8> {
        let mut data = VgmBuilder::new().build_bytes().unwrap();
        // Corrupt the "Vgm " signature
        data[0] = b'X';
        data[1] = b'g';
        data[2] = b'm';
        data[3] = b' ';
        data
    }

    /// Generate file with version too high
    pub fn unsupported_version() -> VgmBuilder {
        VgmBuilder::new().version(999)
    }

    /// Generate file with negative offsets (will be cast to huge positive)
    pub fn invalid_offsets() -> VgmBuilder {
        VgmBuilder::new().header(|h| {
            h.raw_field(|data| {
                data.gd3_offset = u32::MAX; // Invalid huge offset
                data.loop_offset = u32::MAX;
            })
        })
    }

    /// Generate file with zero clocks (should be okay but edge case)
    pub fn zero_clocks() -> VgmBuilder {
        VgmBuilder::new().header(|h| h.psg_clock(0).ym2612_clock(0))
    }

    /// Generate file with huge sample counts
    pub fn huge_sample_count() -> VgmBuilder {
        VgmBuilder::new().header(|h| h.total_samples(u32::MAX))
    }

    /// Generate truncated file
    pub fn truncated_file() -> Vec<u8> {
        let full_data = VgmBuilder::new().build_bytes().unwrap();
        // Return only first 32 bytes (should be 64+ minimum)
        full_data[..32].to_vec()
    }
}

/// Edge case generators for boundary testing
pub struct EdgeCaseGenerators;

impl EdgeCaseGenerators {
    /// Generate minimal valid VGM file
    pub fn minimal_valid() -> VgmBuilder {
        VgmBuilder::new()
            .version(100)
            .header(|h| h.total_samples(1))
            .commands(|_c| CommandsBuilder::new().with_end())
            .metadata(|m| {
                m.english_track("")
                    .english_game("")
                    .english_system("")
                    .english_author("")
            })
    }

    /// Generate file with maximum string lengths
    pub fn max_strings() -> VgmBuilder {
        let long_string = "a".repeat(65535); // Max reasonable string length
        VgmBuilder::new().metadata(|m| {
            m.english_track(&long_string)
                .english_game(&long_string)
                .notes(&long_string)
        })
    }

    /// Generate file with many commands
    pub fn many_commands() -> VgmBuilder {
        let mut builder = VgmBuilder::new().commands(|_c| CommandsBuilder::new());
        
        // Add 10000 PSG writes
        for i in 0..10000 {
            let value = (i % 256) as u8;
            builder = builder.commands(|c| c.psg_write(value, 0));
        }
        
        builder.commands(|c| c.with_end())
    }

    /// Generate file with data blocks
    pub fn with_data_blocks() -> VgmBuilder {
        let test_data = vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC];
        let data_block = DataBlockContent::UncompressedStream {
            chip_type: StreamChipType::YM2612,
            data: test_data.clone(),
        };

        VgmBuilder::new().commands(|c| {
            c.data_block(0x00, data_block)
                .simple_ym2612_sequence()
                .with_end()
        })
    }

    /// Generate file with compressed data blocks
    pub fn with_compressed_data() -> VgmBuilder {
        let test_data = vec![0xAA; 1000]; // Repeated pattern, good for compression
        let data_block = DataBlockContent::CompressedStream {
            chip_type: StreamChipType::YM2612,
            compression: CompressionType::BitPacking {
                bits_decompressed: 8,
                bits_compressed: 4,
                sub_type: 0,
                add_value: 0xAA,
            },
            uncompressed_size: 1000,
            data: test_data,
        };

        VgmBuilder::new().commands(|c| {
            c.data_block(0x01, data_block)
                .simple_ym2612_sequence()
                .with_end()
        })
    }

    /// Generate file at size boundaries
    pub fn size_boundary_64kb() -> VgmBuilder {
        // Generate file close to 64KB boundary
        let mut builder = VgmBuilder::new();
        
        // Add enough commands to reach close to 64KB
        let target_commands = 16000; // Rough estimate
        for i in 0..target_commands {
            let value = (i % 256) as u8;
            builder = builder.commands(|c| c.psg_write(value, 0).wait_samples(1));
        }
        
        builder.commands(|c| c.with_end())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_vgm_builder() {
        let vgm = VgmBuilder::new()
            .version(150)
            .header(|h| h.psg_clock(3579545))
            .commands(|c| c.simple_psg_sequence())
            .build()
            .unwrap();

        assert_eq!(vgm.header.version, 150);
        assert_eq!(vgm.header.sn76489_clock, 3579545);
        assert!(!vgm.commands.is_empty());
        assert_eq!(vgm.commands.last(), Some(&Commands::EndOfSoundData));
    }

    #[test]
    fn test_version_generators() {
        let vgm_100 = VgmVersionGenerators::vgm_v100_basic().build().unwrap();
        assert_eq!(vgm_100.header.version, 100);
        assert_eq!(vgm_100.header.ym2612_clock, 0); // No YM2612 in v1.00

        let vgm_170 = VgmVersionGenerators::vgm_v170_dual_chip().build().unwrap();
        assert_eq!(vgm_170.header.version, 170);
    }

    #[test]
    fn test_edge_cases() {
        let minimal = EdgeCaseGenerators::minimal_valid().build().unwrap();
        assert_eq!(minimal.header.total_nb_samples, 1);

        let _many_cmds = EdgeCaseGenerators::many_commands().build().unwrap();
        // Should not crash with many commands
    }

    #[test]
    fn test_invalid_generators() {
        let invalid_sig = InvalidVgmGenerators::invalid_signature();
        assert_eq!(invalid_sig[0], b'X'); // Corrupted signature

        let truncated = InvalidVgmGenerators::truncated_file();
        assert_eq!(truncated.len(), 32); // Should be truncated
    }

    #[test]
    fn test_builder_serialization() {
        let bytes = VgmBuilder::new()
            .version(150)
            .commands(|c| c.simple_psg_sequence())
            .build_bytes()
            .unwrap();

        assert!(!bytes.is_empty());
        // Should start with VGM signature
        assert_eq!(&bytes[0..4], b"Vgm ");
    }
}