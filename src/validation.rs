use crate::errors::{VgmError, VgmResult};
use crate::{Commands, HeaderData, VgmMetadata};

/// Configuration for validation limits and rules
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Minimum supported VGM version (BCD format)
    pub min_vgm_version: u32,
    /// Maximum supported VGM version (BCD format)  
    pub max_vgm_version: u32,
    /// Maximum allowed file size (bytes)
    pub max_file_size: usize,
    /// Maximum allowed number of commands
    pub max_commands: usize,
    /// Maximum allowed data block size
    pub max_data_block_size: u32,
    /// Whether to perform strict validation (fail on warnings)
    pub strict_mode: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            min_vgm_version: 100,                   // Version 1.00 (decimal)
            max_vgm_version: 171,                   // Version 1.71 (decimal)
            max_file_size: 64 * 1024 * 1024,       // 64MB limit
            max_commands: 1_000_000,               // 1M commands limit
            max_data_block_size: 16 * 1024 * 1024, // 16MB data block limit
            strict_mode: false,
        }
    }
}

/// Validation context containing file information
#[derive(Debug)]
pub struct ValidationContext {
    pub file_size: usize,
    pub config: ValidationConfig,
}

/// Trait for validatable VGM components
pub trait VgmValidate {
    /// Validate the component with the given context
    fn validate(&self, context: &ValidationContext) -> VgmResult<()>;

    /// Perform lightweight validation checks
    fn quick_validate(&self) -> VgmResult<()> {
        let default_context = ValidationContext {
            file_size: usize::MAX,
            config: ValidationConfig::default(),
        };
        self.validate(&default_context)
    }
}

/// Version compatibility validator
pub struct VersionValidator;

impl VersionValidator {
    /// Check if VGM version is supported
    pub fn validate_version(version: u32, config: &ValidationConfig) -> VgmResult<()> {
        if version < config.min_vgm_version {
            return Err(VgmError::UnsupportedVgmVersion {
                version,
                supported_range: format!("{}+", Self::version_to_string(config.min_vgm_version)),
            });
        }

        if version > config.max_vgm_version {
            return Err(VgmError::UnsupportedVgmVersion {
                version,
                supported_range: format!(
                    "{}-{}",
                    Self::version_to_string(config.min_vgm_version),
                    Self::version_to_string(config.max_vgm_version)
                ),
            });
        }

        Ok(())
    }

    /// Convert decimal version to human-readable string
    fn version_to_string(version: u32) -> String {
        let major = version / 100;
        let minor = version % 100;
        format!("{}.{:02}", major, minor)
    }
}

/// Offset bounds validator
pub struct OffsetValidator;

impl OffsetValidator {
    /// Validate that an offset points within the file bounds
    pub fn validate_offset(offset: u32, file_size: usize, field_name: &str) -> VgmResult<()> {
        let offset_usize = usize::try_from(offset).map_err(|_| VgmError::InvalidOffset {
            field: field_name.to_string(),
            offset,
            file_size,
        })?;

        if offset_usize >= file_size {
            return Err(VgmError::InvalidOffset {
                field: field_name.to_string(),
                offset,
                file_size,
            });
        }

        Ok(())
    }

    /// Validate that a range (offset + size) is within file bounds
    pub fn validate_range(
        offset: u32,
        size: u32,
        file_size: usize,
        field_name: &str,
    ) -> VgmResult<()> {
        let end_offset = offset.checked_add(size).ok_or(VgmError::IntegerOverflow {
            operation: format!("{} range calculation", field_name),
            details: format!("offset {} + size {}", offset, size),
        })?;

        let end_offset_usize = usize::try_from(end_offset).map_err(|_| VgmError::InvalidOffset {
            field: field_name.to_string(),
            offset: end_offset,
            file_size,
        })?;

        // For ranges, end_offset can equal file_size (pointing after last byte)
        if end_offset_usize > file_size {
            return Err(VgmError::InvalidOffset {
                field: field_name.to_string(),
                offset: end_offset,
                file_size,
            });
        }

        Ok(())
    }
}

/// Chip configuration validator
pub struct ChipValidator;

impl ChipValidator {
    /// Validate chip clock configuration
    pub fn validate_chip_clocks(header: &HeaderData) -> VgmResult<()> {
        // Check for conflicting chip configurations
        if header.ym2612_clock > 0 && header.ym2203_clock > 0 {
            // Some chips are mutually exclusive in certain contexts
            // This is a simplified check - real validation would be more complex
        }

        // Validate reasonable clock ranges
        Self::validate_clock_range(header.sn76489_clock, "SN76489", 1_000_000, 8_000_000)?;
        Self::validate_clock_range(header.ym2612_clock, "YM2612", 6_000_000, 8_000_000)?;
        Self::validate_clock_range(header.ym2151_clock, "YM2151", 3_000_000, 4_000_000)?;

        Ok(())
    }

    /// Validate that a chip clock is within reasonable bounds
    fn validate_clock_range(
        clock: u32,
        chip_name: &str,
        min_hz: u32,
        max_hz: u32,
    ) -> VgmResult<()> {
        if clock > 0 && (clock < min_hz || clock > max_hz) {
            return Err(VgmError::ValidationFailed {
                field: format!("{} clock", chip_name),
                reason: format!(
                    "Clock {} Hz outside valid range {}-{} Hz",
                    clock, min_hz, max_hz
                ),
            });
        }
        Ok(())
    }

    /// Validate chip volume configuration
    pub fn validate_chip_volumes(header: &HeaderData) -> VgmResult<()> {
        // Check volume modifier is within reasonable range
        if header.volume_modifier > 64 {
            return Err(VgmError::ValidationFailed {
                field: "volume_modifier".to_string(),
                reason: format!(
                    "Volume modifier {} exceeds maximum 64",
                    header.volume_modifier
                ),
            });
        }

        Ok(())
    }
}

/// Data consistency validator
pub struct ConsistencyValidator;

impl ConsistencyValidator {
    /// Validate that header offsets are consistent with file structure
    pub fn validate_header_consistency(header: &HeaderData, file_size: usize) -> VgmResult<()> {
        // Validate VGM data offset
        if header.vgm_data_offset > 0 {
            OffsetValidator::validate_offset(
                header.vgm_data_offset + 0x34,
                file_size,
                "vgm_data_offset",
            )?;
        }

        // Validate GD3 offset if present
        if header.gd3_offset > 0 {
            OffsetValidator::validate_offset(header.gd3_offset + 0x14, file_size, "gd3_offset")?;
        }

        // Validate loop offset if present
        if header.loop_offset > 0 {
            OffsetValidator::validate_offset(header.loop_offset + 0x1C, file_size, "loop_offset")?;
        }

        // Validate extra header offset if present
        if header.extra_header_offset > 0 {
            OffsetValidator::validate_offset(
                header.extra_header_offset + 0xBC,
                file_size,
                "extra_header_offset",
            )?;
        }

        Ok(())
    }

    /// Validate that commands are consistent with header configuration
    pub fn validate_commands_consistency(
        header: &HeaderData,
        commands: &[Commands],
    ) -> VgmResult<()> {
        let mut chip_usage = ChipUsageTracker::new();

        // Analyze command usage
        for command in commands {
            chip_usage.track_command(command);
        }

        // Check that used chips have clock configurations
        chip_usage.validate_against_header(header)?;

        Ok(())
    }
}

/// Helper struct to track chip usage in commands
#[derive(Debug, Default)]
struct ChipUsageTracker {
    sn76489_used: bool,
    ym2612_used: bool,
    ym2151_used: bool,
    ym2413_used: bool,
    ym2203_used: bool,
    ym2608_used: bool,
    ym2610_used: bool,
    ym3812_used: bool,
    ym3526_used: bool,
    y8950_used: bool,
}

impl ChipUsageTracker {
    fn new() -> Self {
        Self::default()
    }

    fn track_command(&mut self, command: &Commands) {
        match command {
            Commands::PSGWrite { .. } => self.sn76489_used = true,
            Commands::YM2612Port0Write { .. } | Commands::YM2612Port1Write { .. } => {
                self.ym2612_used = true
            },
            Commands::YM2151Write { .. } => self.ym2151_used = true,
            Commands::YM2413Write { .. } => self.ym2413_used = true,
            Commands::YM2203Write { .. } => self.ym2203_used = true,
            Commands::YM2608Port0Write { .. } | Commands::YM2608Port1Write { .. } => {
                self.ym2608_used = true
            },
            Commands::YM2610Port0Write { .. } | Commands::YM2610Port1Write { .. } => {
                self.ym2610_used = true
            },
            Commands::YM3812Write { .. } => self.ym3812_used = true,
            Commands::YM3526Write { .. } => self.ym3526_used = true,
            Commands::Y8950Write { .. } => self.y8950_used = true,
            _ => {}, // Other commands don't indicate specific chip usage
        }
    }

    fn validate_against_header(&self, header: &HeaderData) -> VgmResult<()> {
        // Check that used chips have clock configurations
        if self.sn76489_used && header.sn76489_clock == 0 {
            return Err(VgmError::InconsistentData {
                context: "Chip usage validation".to_string(),
                reason: "SN76489 commands found but no clock configured".to_string(),
            });
        }

        if self.ym2612_used && header.ym2612_clock == 0 {
            return Err(VgmError::InconsistentData {
                context: "Chip usage validation".to_string(),
                reason: "YM2612 commands found but no clock configured".to_string(),
            });
        }

        if self.ym2151_used && header.ym2151_clock == 0 {
            return Err(VgmError::InconsistentData {
                context: "Chip usage validation".to_string(),
                reason: "YM2151 commands found but no clock configured".to_string(),
            });
        }

        if self.ym2413_used && header.ym2413_clock == 0 {
            return Err(VgmError::InconsistentData {
                context: "Chip usage validation".to_string(),
                reason: "YM2413 commands found but no clock configured".to_string(),
            });
        }

        if self.ym2203_used && header.ym2203_clock == 0 {
            return Err(VgmError::InconsistentData {
                context: "Chip usage validation".to_string(),
                reason: "YM2203 commands found but no clock configured".to_string(),
            });
        }

        if self.ym2608_used && header.ym2608_clock == 0 {
            return Err(VgmError::InconsistentData {
                context: "Chip usage validation".to_string(),
                reason: "YM2608 commands found but no clock configured".to_string(),
            });
        }

        if self.ym2610_used && header.ym2610_b_clock == 0 {
            return Err(VgmError::InconsistentData {
                context: "Chip usage validation".to_string(),
                reason: "YM2610 commands found but no clock configured".to_string(),
            });
        }

        if self.ym3812_used && header.ym3812_clock == 0 {
            return Err(VgmError::InconsistentData {
                context: "Chip usage validation".to_string(),
                reason: "YM3812 commands found but no clock configured".to_string(),
            });
        }

        if self.ym3526_used && header.ym3526_clock == 0 {
            return Err(VgmError::InconsistentData {
                context: "Chip usage validation".to_string(),
                reason: "YM3526 commands found but no clock configured".to_string(),
            });
        }

        if self.y8950_used && header.y8950_clock == 0 {
            return Err(VgmError::InconsistentData {
                context: "Chip usage validation".to_string(),
                reason: "Y8950 commands found but no clock configured".to_string(),
            });
        }

        Ok(())
    }
}

/// Main validator that coordinates all validation checks
pub struct VgmValidator {
    config: ValidationConfig,
}

impl VgmValidator {
    /// Create a new validator with the given configuration
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Create a validator with default configuration
    pub fn default() -> Self {
        Self::new(ValidationConfig::default())
    }

    /// Perform comprehensive validation of a VGM file
    pub fn validate_vgm_file(
        &self,
        header: &HeaderData,
        commands: &[Commands],
        metadata: &VgmMetadata,
        file_size: usize,
    ) -> VgmResult<()> {
        let context = ValidationContext {
            file_size,
            config: self.config.clone(),
        };

        // Version compatibility validation
        VersionValidator::validate_version(header.version, &self.config)?;

        // Header validation
        header.validate(&context)?;

        // Commands validation
        if commands.len() > self.config.max_commands {
            return Err(VgmError::DataSizeExceedsLimit {
                field: "commands".to_string(),
                size: commands.len(),
                limit: self.config.max_commands,
            });
        }

        // Metadata validation
        metadata.validate(&context)?;

        // Cross-component consistency validation
        ConsistencyValidator::validate_header_consistency(header, file_size)?;
        ConsistencyValidator::validate_commands_consistency(header, commands)?;

        Ok(())
    }

    /// Perform quick validation suitable for streaming scenarios
    pub fn quick_validate_header(&self, header: &HeaderData) -> VgmResult<()> {
        // Fast version check
        VersionValidator::validate_version(header.version, &self.config)?;

        // Basic chip validation
        ChipValidator::validate_chip_clocks(header)?;
        ChipValidator::validate_chip_volumes(header)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HeaderData, Commands, VgmMetadata, Gd3LocaleData};

    #[test]
    fn test_validation_config_default() {
        let config = ValidationConfig::default();
        
        // Test all default values are reasonable
        assert_eq!(config.min_vgm_version, 100);
        assert_eq!(config.max_vgm_version, 171);
        assert_eq!(config.max_file_size, 64 * 1024 * 1024);
        assert_eq!(config.max_commands, 1_000_000);
        assert_eq!(config.max_data_block_size, 16 * 1024 * 1024);
        assert!(!config.strict_mode);
        
        // Verify logical relationships
        assert!(config.max_vgm_version > config.min_vgm_version);
        assert!(config.max_file_size > 1024);
        assert!(config.max_commands > 0);
        assert!(config.max_data_block_size > 0);
    }

    #[test]
    fn test_validation_config_custom() {
        let config = ValidationConfig {
            min_vgm_version: 150,
            max_vgm_version: 160,
            max_file_size: 1024 * 1024,
            max_commands: 10_000,
            max_data_block_size: 1024 * 1024,
            strict_mode: true,
        };
        
        assert_eq!(config.min_vgm_version, 150);
        assert_eq!(config.max_vgm_version, 160);
        assert!(config.strict_mode);
    }

    #[test]
    fn test_validation_context() {
        let config = ValidationConfig::default();
        let context = ValidationContext {
            file_size: 1024,
            config: config.clone(),
        };
        
        assert_eq!(context.file_size, 1024);
        assert_eq!(context.config.min_vgm_version, config.min_vgm_version);
    }

    #[test]
    fn test_version_validator_valid_versions() {
        let config = ValidationConfig::default();

        // Test boundary valid versions
        assert!(VersionValidator::validate_version(100, &config).is_ok()); // Min version
        assert!(VersionValidator::validate_version(171, &config).is_ok()); // Max version
        assert!(VersionValidator::validate_version(151, &config).is_ok()); // Common version
        assert!(VersionValidator::validate_version(150, &config).is_ok()); // Another common version
    }

    #[test]
    fn test_version_validator_invalid_versions() {
        let config = ValidationConfig::default();

        // Too old versions
        assert!(VersionValidator::validate_version(50, &config).is_err());
        assert!(VersionValidator::validate_version(99, &config).is_err());
        
        // Too new versions  
        assert!(VersionValidator::validate_version(172, &config).is_err());
        assert!(VersionValidator::validate_version(200, &config).is_err());
        assert!(VersionValidator::validate_version(999, &config).is_err());
    }

    #[test]
    fn test_version_validator_error_messages() {
        let config = ValidationConfig::default();
        
        // Test error message for too old version
        let result = VersionValidator::validate_version(50, &config);
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::UnsupportedVgmVersion { version, supported_range } => {
                assert_eq!(version, 50);
                assert!(supported_range.contains("1.00+"));
            },
            _ => panic!("Expected UnsupportedVgmVersion error"),
        }
        
        // Test error message for too new version
        let result = VersionValidator::validate_version(200, &config);
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::UnsupportedVgmVersion { version, supported_range } => {
                assert_eq!(version, 200);
                assert!(supported_range.contains("1.00-1.71"));
            },
            _ => panic!("Expected UnsupportedVgmVersion error"),
        }
    }

    #[test]
    fn test_version_to_string() {
        assert_eq!(VersionValidator::version_to_string(100), "1.00");
        assert_eq!(VersionValidator::version_to_string(151), "1.51");
        assert_eq!(VersionValidator::version_to_string(171), "1.71");
        assert_eq!(VersionValidator::version_to_string(200), "2.00");
        assert_eq!(VersionValidator::version_to_string(999), "9.99");
    }

    #[test]
    fn test_offset_validator_valid_offsets() {
        // Valid offsets within bounds
        assert!(OffsetValidator::validate_offset(0, 1000, "test").is_ok());
        assert!(OffsetValidator::validate_offset(100, 1000, "test").is_ok());
        assert!(OffsetValidator::validate_offset(999, 1000, "test").is_ok());
    }

    #[test]
    fn test_offset_validator_invalid_offsets() {
        // Invalid offsets beyond file size
        assert!(OffsetValidator::validate_offset(1000, 1000, "test").is_err());
        assert!(OffsetValidator::validate_offset(1500, 1000, "test").is_err());
        assert!(OffsetValidator::validate_offset(u32::MAX, 1000, "test").is_err());
    }

    #[test]
    fn test_offset_validator_error_types() {
        let result = OffsetValidator::validate_offset(1500, 1000, "test_field");
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::InvalidOffset { field, offset, file_size } => {
                assert_eq!(field, "test_field");
                assert_eq!(offset, 1500);
                assert_eq!(file_size, 1000);
            },
            _ => panic!("Expected InvalidOffset error"),
        }
    }

    #[test]
    fn test_offset_validator_valid_ranges() {
        // Valid ranges within bounds
        assert!(OffsetValidator::validate_range(0, 100, 1000, "test").is_ok());
        assert!(OffsetValidator::validate_range(100, 50, 1000, "test").is_ok());
        assert!(OffsetValidator::validate_range(900, 100, 1000, "test").is_ok());
    }

    #[test]
    fn test_offset_validator_invalid_ranges() {
        // Invalid ranges beyond file bounds
        assert!(OffsetValidator::validate_range(950, 100, 1000, "test").is_err());
        assert!(OffsetValidator::validate_range(900, 200, 1000, "test").is_err());
    }

    #[test]
    fn test_offset_validator_range_overflow() {
        // Test integer overflow in range calculation
        let result = OffsetValidator::validate_range(u32::MAX - 10, 20, 1000, "test");
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::IntegerOverflow { operation, details } => {
                assert!(operation.contains("test range calculation"));
                assert!(details.contains("offset"));
                assert!(details.contains("size"));
            },
            _ => panic!("Expected IntegerOverflow error"),
        }
    }

    #[test]
    fn test_chip_validator_valid_clocks() {
        let mut header = HeaderData::default();

        // Test valid common clock frequencies
        header.sn76489_clock = 3579545; // Common PSG clock
        header.ym2612_clock = 7670453; // Common YM2612 clock
        header.ym2151_clock = 3579545; // Common YM2151 clock
        assert!(ChipValidator::validate_chip_clocks(&header).is_ok());

        // Test zero clocks (disabled chips)
        header.sn76489_clock = 0;
        header.ym2612_clock = 0;
        header.ym2151_clock = 0;
        assert!(ChipValidator::validate_chip_clocks(&header).is_ok());
    }

    #[test]
    fn test_chip_validator_invalid_clocks() {
        let mut header = HeaderData::default();

        // Test SN76489 clock out of range
        header.sn76489_clock = 500_000; // Too low
        assert!(ChipValidator::validate_chip_clocks(&header).is_err());
        
        header.sn76489_clock = 50_000_000; // Too high
        assert!(ChipValidator::validate_chip_clocks(&header).is_err());

        // Test YM2612 clock out of range
        header.sn76489_clock = 0; // Reset to valid
        header.ym2612_clock = 1_000_000; // Too low
        assert!(ChipValidator::validate_chip_clocks(&header).is_err());
        
        header.ym2612_clock = 20_000_000; // Too high
        assert!(ChipValidator::validate_chip_clocks(&header).is_err());

        // Test YM2151 clock out of range
        header.ym2612_clock = 0; // Reset to valid
        header.ym2151_clock = 1_000_000; // Too low
        assert!(ChipValidator::validate_chip_clocks(&header).is_err());
        
        header.ym2151_clock = 10_000_000; // Too high
        assert!(ChipValidator::validate_chip_clocks(&header).is_err());
    }

    #[test]
    fn test_chip_validator_clock_error_messages() {
        let mut header = HeaderData::default();
        header.sn76489_clock = 50_000_000; // Too high
        
        let result = ChipValidator::validate_chip_clocks(&header);
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::ValidationFailed { field, reason } => {
                assert_eq!(field, "SN76489 clock");
                assert!(reason.contains("50000000"));
                assert!(reason.contains("Hz"));
                assert!(reason.contains("outside valid range"));
            },
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_chip_validator_valid_volumes() {
        let mut header = HeaderData::default();
        
        // Valid volume modifier values
        header.volume_modifier = 0;
        assert!(ChipValidator::validate_chip_volumes(&header).is_ok());
        
        header.volume_modifier = 32;
        assert!(ChipValidator::validate_chip_volumes(&header).is_ok());
        
        header.volume_modifier = 64;
        assert!(ChipValidator::validate_chip_volumes(&header).is_ok());
    }

    #[test]
    fn test_chip_validator_invalid_volumes() {
        let mut header = HeaderData::default();
        
        // Invalid volume modifier values
        header.volume_modifier = 65;
        assert!(ChipValidator::validate_chip_volumes(&header).is_err());
        
        header.volume_modifier = 100;
        assert!(ChipValidator::validate_chip_volumes(&header).is_err());
        
        header.volume_modifier = u8::MAX;
        assert!(ChipValidator::validate_chip_volumes(&header).is_err());
    }

    #[test]
    fn test_chip_validator_volume_error_message() {
        let mut header = HeaderData::default();
        header.volume_modifier = 100;
        
        let result = ChipValidator::validate_chip_volumes(&header);
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::ValidationFailed { field, reason } => {
                assert_eq!(field, "volume_modifier");
                assert!(reason.contains("100"));
                assert!(reason.contains("exceeds maximum 64"));
            },
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_consistency_validator_valid_header() {
        let mut header = HeaderData::default();
        let file_size = 1000;
        
        // Valid offsets
        header.vgm_data_offset = 100;
        header.gd3_offset = 200;
        header.loop_offset = 150;
        
        assert!(ConsistencyValidator::validate_header_consistency(&header, file_size).is_ok());
    }

    #[test]
    fn test_consistency_validator_invalid_header_offsets() {
        let mut header = HeaderData::default();
        let file_size = 1000;
        
        // Invalid VGM data offset
        header.vgm_data_offset = 1000; // Would place data at 1000 + 0x34 = 1052 > file_size
        let result = ConsistencyValidator::validate_header_consistency(&header, file_size);
        assert!(result.is_err());
        
        // Reset and test GD3 offset
        header.vgm_data_offset = 0;
        header.gd3_offset = 1000; // Would place GD3 at 1000 + 0x14 = 1020 > file_size
        let result = ConsistencyValidator::validate_header_consistency(&header, file_size);
        assert!(result.is_err());
        
        // Reset and test loop offset
        header.gd3_offset = 0;
        header.loop_offset = 1000; // Would place loop at 1000 + 0x1C = 1028 > file_size
        let result = ConsistencyValidator::validate_header_consistency(&header, file_size);
        assert!(result.is_err());
    }

    #[test]
    fn test_chip_usage_tracker() {
        let mut tracker = ChipUsageTracker::new();
        
        // Initially no chips used
        assert!(!tracker.sn76489_used);
        assert!(!tracker.ym2612_used);
        assert!(!tracker.ym2151_used);
        assert!(!tracker.ym2413_used);
        
        // Track PSG command
        tracker.track_command(&Commands::PSGWrite { value: 0x9F, chip_index: 0 });
        assert!(tracker.sn76489_used);
        
        // Track YM2612 commands
        tracker.track_command(&Commands::YM2612Port0Write { register: 0x28, value: 0x00, chip_index: 0 });
        assert!(tracker.ym2612_used);
        
        tracker.track_command(&Commands::YM2612Port1Write { register: 0x28, value: 0x00, chip_index: 0 });
        assert!(tracker.ym2612_used);
        
        // Track other chip commands
        tracker.track_command(&Commands::YM2151Write { register: 0x08, value: 0x00, chip_index: 0 });
        assert!(tracker.ym2151_used);
        
        tracker.track_command(&Commands::YM2413Write { register: 0x10, value: 0x00, chip_index: 0 });
        assert!(tracker.ym2413_used);
        
        tracker.track_command(&Commands::YM2203Write { register: 0x07, value: 0x3F, chip_index: 0 });
        assert!(tracker.ym2203_used);
        
        tracker.track_command(&Commands::YM2608Port0Write { register: 0x07, value: 0x3F, chip_index: 0 });
        assert!(tracker.ym2608_used);
        
        tracker.track_command(&Commands::YM2610Port0Write { register: 0x07, value: 0x3F, chip_index: 0 });
        assert!(tracker.ym2610_used);
        
        tracker.track_command(&Commands::YM3812Write { register: 0x20, value: 0x00, chip_index: 0 });
        assert!(tracker.ym3812_used);
        
        tracker.track_command(&Commands::YM3526Write { register: 0x20, value: 0x00, chip_index: 0 });
        assert!(tracker.ym3526_used);
        
        tracker.track_command(&Commands::Y8950Write { register: 0x20, value: 0x00, chip_index: 0 });
        assert!(tracker.y8950_used);
    }

    #[test]
    fn test_chip_usage_tracker_non_chip_commands() {
        let mut tracker = ChipUsageTracker::new();
        
        // Commands that don't indicate specific chip usage
        tracker.track_command(&Commands::Wait735Samples);
        tracker.track_command(&Commands::Wait882Samples);
        tracker.track_command(&Commands::EndOfSoundData);
        tracker.track_command(&Commands::WaitNSamples { n: 100 });
        
        // Should not mark any chips as used
        assert!(!tracker.sn76489_used);
        assert!(!tracker.ym2612_used);
        assert!(!tracker.ym2151_used);
        assert!(!tracker.ym2413_used);
    }

    #[test]
    fn test_chip_usage_validation_success() {
        let mut tracker = ChipUsageTracker::new();
        let mut header = HeaderData::default();
        
        // Set up header with clocks
        header.sn76489_clock = 3579545;
        header.ym2612_clock = 7670453;
        header.ym2151_clock = 3579545;
        
        // Track usage
        tracker.track_command(&Commands::PSGWrite { value: 0x9F, chip_index: 0 });
        tracker.track_command(&Commands::YM2612Port0Write { register: 0x28, value: 0x00, chip_index: 0 });
        tracker.track_command(&Commands::YM2151Write { register: 0x08, value: 0x00, chip_index: 0 });
        
        // Should pass validation
        assert!(tracker.validate_against_header(&header).is_ok());
    }

    #[test]
    fn test_chip_usage_validation_failures() {
        let mut tracker = ChipUsageTracker::new();
        let header = HeaderData::default(); // All clocks are 0
        
        // Test SN76489 usage without clock
        tracker.track_command(&Commands::PSGWrite { value: 0x9F, chip_index: 0 });
        let result = tracker.validate_against_header(&header);
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::InconsistentData { context, reason } => {
                assert_eq!(context, "Chip usage validation");
                assert!(reason.contains("SN76489"));
                assert!(reason.contains("no clock configured"));
            },
            _ => panic!("Expected InconsistentData error"),
        }
        
        // Reset tracker and test YM2612
        let mut tracker = ChipUsageTracker::new();
        tracker.track_command(&Commands::YM2612Port0Write { register: 0x28, value: 0x00, chip_index: 0 });
        let result = tracker.validate_against_header(&header);
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::InconsistentData { context, reason } => {
                assert!(reason.contains("YM2612"));
            },
            _ => panic!("Expected InconsistentData error"),
        }
        
        // Test each chip type
        let chip_tests = [
            (Commands::YM2151Write { register: 0x08, value: 0x00, chip_index: 0 }, "YM2151"),
            (Commands::YM2413Write { register: 0x10, value: 0x00, chip_index: 0 }, "YM2413"),
            (Commands::YM2203Write { register: 0x07, value: 0x3F, chip_index: 0 }, "YM2203"),
            (Commands::YM2608Port0Write { register: 0x07, value: 0x3F, chip_index: 0 }, "YM2608"),
            (Commands::YM2610Port0Write { register: 0x07, value: 0x3F, chip_index: 0 }, "YM2610"),
            (Commands::YM3812Write { register: 0x20, value: 0x00, chip_index: 0 }, "YM3812"),
            (Commands::YM3526Write { register: 0x20, value: 0x00, chip_index: 0 }, "YM3526"),
            (Commands::Y8950Write { register: 0x20, value: 0x00, chip_index: 0 }, "Y8950"),
        ];
        
        for (command, chip_name) in &chip_tests {
            let mut tracker = ChipUsageTracker::new();
            tracker.track_command(command);
            let result = tracker.validate_against_header(&header);
            assert!(result.is_err(), "Expected error for {}", chip_name);
            match result.unwrap_err() {
                VgmError::InconsistentData { reason, .. } => {
                    assert!(reason.contains(chip_name), "Error should mention {}", chip_name);
                },
                _ => panic!("Expected InconsistentData error for {}", chip_name),
            }
        }
    }

    #[test]
    fn test_consistency_validator_commands() {
        let mut header = HeaderData::default();
        header.sn76489_clock = 3579545;
        header.ym2612_clock = 7670453;
        
        let commands = vec![
            Commands::PSGWrite { value: 0x9F, chip_index: 0 },
            Commands::YM2612Port0Write { register: 0x28, value: 0x00, chip_index: 0 },
            Commands::Wait735Samples,
            Commands::EndOfSoundData,
        ];
        
        // Should pass validation
        assert!(ConsistencyValidator::validate_commands_consistency(&header, &commands).is_ok());
        
        // Test with missing clock configuration
        header.sn76489_clock = 0; // Remove PSG clock
        let result = ConsistencyValidator::validate_commands_consistency(&header, &commands);
        assert!(result.is_err());
    }

    #[test]
    fn test_vgm_validator_new() {
        let config = ValidationConfig::default();
        let validator = VgmValidator::new(config.clone());
        assert_eq!(validator.config.min_vgm_version, config.min_vgm_version);
        
        let default_validator = VgmValidator::default();
        assert_eq!(default_validator.config.min_vgm_version, ValidationConfig::default().min_vgm_version);
    }

    #[test]
    fn test_vgm_validator_quick_validate_header() {
        let validator = VgmValidator::default();
        
        // Valid header
        let mut header = HeaderData::default();
        header.version = 151;
        header.sn76489_clock = 3579545;
        header.volume_modifier = 32;
        assert!(validator.quick_validate_header(&header).is_ok());
        
        // Invalid version
        header.version = 50; // Too old
        assert!(validator.quick_validate_header(&header).is_err());
        
        // Invalid volume
        header.version = 151; // Reset to valid
        header.volume_modifier = 100; // Too high
        assert!(validator.quick_validate_header(&header).is_err());
    }

    #[test]
    fn test_vgm_validator_full_validation() {
        let validator = VgmValidator::default();
        
        // Create valid test data
        let header = HeaderData {
            version: 151,
            sn76489_clock: 3579545,
            ym2612_clock: 7670453,
            vgm_data_offset: 0x40,
            gd3_offset: 0x100,
            volume_modifier: 32,
            ..Default::default()
        };
        
        let commands = vec![
            Commands::PSGWrite { value: 0x9F, chip_index: 0 },
            Commands::YM2612Port0Write { register: 0x28, value: 0x00, chip_index: 0 },
            Commands::Wait735Samples,
            Commands::EndOfSoundData,
        ];
        
        let metadata = VgmMetadata {
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
            notes: "Test Notes".to_string(),
        };
        
        let file_size = 1024;
        
        // Should pass validation
        assert!(validator.validate_vgm_file(&header, &commands, &metadata, file_size).is_ok());
    }

    #[test]
    fn test_vgm_validator_full_validation_failures() {
        let validator = VgmValidator::default();
        
        // Invalid version
        let mut header = HeaderData {
            version: 50, // Too old
            sn76489_clock: 3579545,
            ..Default::default()
        };
        
        let commands = vec![Commands::EndOfSoundData];
        let metadata = VgmMetadata {
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
        };
        
        let result = validator.validate_vgm_file(&header, &commands, &metadata, 1024);
        assert!(result.is_err());
        
        // Too many commands
        header.version = 151; // Fix version
        let many_commands = vec![Commands::Wait735Samples; 2_000_000]; // Exceeds default limit
        let result = validator.validate_vgm_file(&header, &many_commands, &metadata, 1024);
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::DataSizeExceedsLimit { field, size, limit } => {
                assert_eq!(field, "commands");
                assert_eq!(size, 2_000_000);
                assert_eq!(limit, 1_000_000);
            },
            _ => panic!("Expected DataSizeExceedsLimit error"),
        }
    }

    #[test]
    fn test_vgm_validate_trait_quick_validate() {
        // Test that quick_validate uses default context
        let header = HeaderData {
            version: 151,
            sn76489_clock: 3579545,
            volume_modifier: 32,
            ..Default::default()
        };
        
        // This should work since quick_validate creates a permissive context
        assert!(header.quick_validate().is_ok());
        
        // Test with invalid data
        let invalid_header = HeaderData {
            version: 50, // Too old
            ..Default::default()
        };
        
        assert!(invalid_header.quick_validate().is_err());
    }

    #[test]
    fn test_validation_config_edge_cases() {
        // Test with extreme values
        let config = ValidationConfig {
            min_vgm_version: 0,
            max_vgm_version: u32::MAX,
            max_file_size: usize::MAX,
            max_commands: usize::MAX,
            max_data_block_size: u32::MAX,
            strict_mode: true,
        };
        
        // Should still work with extreme values
        assert_eq!(config.min_vgm_version, 0);
        assert_eq!(config.max_vgm_version, u32::MAX);
    }

    #[test]
    fn test_offset_validator_edge_cases() {
        // Test with zero file size
        assert!(OffsetValidator::validate_offset(0, 0, "test").is_err());
        assert!(OffsetValidator::validate_offset(1, 0, "test").is_err());
        
        // Test with maximum values
        assert!(OffsetValidator::validate_offset(u32::MAX, usize::MAX, "test").is_ok());
        
        // Test range with zero size
        assert!(OffsetValidator::validate_range(100, 0, 1000, "test").is_ok());
    }

    #[test]
    fn test_chip_validator_all_chips_disabled() {
        let header = HeaderData::default(); // All clocks are 0
        
        // Should pass validation when no chips are configured
        assert!(ChipValidator::validate_chip_clocks(&header).is_ok());
        assert!(ChipValidator::validate_chip_volumes(&header).is_ok());
    }

    #[test]
    fn test_validation_comprehensive_integration() {
        // Test a comprehensive validation scenario with multiple validators
        let config = ValidationConfig {
            min_vgm_version: 150,
            max_vgm_version: 160,
            max_file_size: 2048,
            max_commands: 100,
            max_data_block_size: 1024,
            strict_mode: true,
        };
        
        let validator = VgmValidator::new(config);
        
        let header = HeaderData {
            version: 151,
            sn76489_clock: 3579545,
            vgm_data_offset: 100,
            gd3_offset: 200,
            volume_modifier: 16,
            ..Default::default()
        };
        
        let commands = vec![
            Commands::PSGWrite { value: 0x9F, chip_index: 0 },
            Commands::Wait735Samples,
            Commands::EndOfSoundData,
        ];
        
        let metadata = VgmMetadata {
            english_data: Gd3LocaleData {
                track: "Short".to_string(),
                game: "Test".to_string(),
                system: "Sys".to_string(),
                author: "Me".to_string(),
            },
            japanese_data: Gd3LocaleData {
                track: "".to_string(),
                game: "".to_string(),
                system: "".to_string(),
                author: "".to_string(),
            },
            date_release: "2024".to_string(),
            name_vgm_creator: "Test".to_string(),
            notes: "Notes".to_string(),
        };
        
        // Should pass all validations
        assert!(validator.validate_vgm_file(&header, &commands, &metadata, 1500).is_ok());
        
        // Test failure with too many commands
        let too_many_commands = vec![Commands::Wait735Samples; 150];
        let result = validator.validate_vgm_file(&header, &too_many_commands, &metadata, 1500);
        assert!(result.is_err());
    }

    #[test]
    fn test_debug_formatting() {
        let config = ValidationConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ValidationConfig"));
        
        let context = ValidationContext {
            file_size: 1024,
            config: config.clone(),
        };
        let debug_str = format!("{:?}", context);
        assert!(debug_str.contains("ValidationContext"));
        
        let tracker = ChipUsageTracker::new();
        let debug_str = format!("{:?}", tracker);
        assert!(debug_str.contains("ChipUsageTracker"));
    }
}
