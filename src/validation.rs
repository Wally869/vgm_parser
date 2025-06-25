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

        Self::validate_offset(end_offset, file_size, field_name)
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
    use crate::HeaderData;

    #[test]
    fn test_version_validator() {
        let config = ValidationConfig::default();

        // Valid version (1.51 in decimal)
        assert!(VersionValidator::validate_version(151, &config).is_ok());

        // Too old version (0.50 in decimal)
        assert!(VersionValidator::validate_version(50, &config).is_err());

        // Too new version (2.00 in decimal)
        assert!(VersionValidator::validate_version(200, &config).is_err());
    }

    #[test]
    fn test_offset_validator() {
        // Valid offset
        assert!(OffsetValidator::validate_offset(100, 1000, "test").is_ok());

        // Invalid offset - beyond file
        assert!(OffsetValidator::validate_offset(1500, 1000, "test").is_err());

        // Valid range
        assert!(OffsetValidator::validate_range(100, 50, 1000, "test").is_ok());

        // Invalid range - beyond file
        assert!(OffsetValidator::validate_range(950, 100, 1000, "test").is_err());
    }

    #[test]
    fn test_chip_validator() {
        let mut header = HeaderData::default();

        // Valid chip clocks
        header.sn76489_clock = 3579545; // Common PSG clock
        header.ym2612_clock = 7670453; // Common YM2612 clock
        assert!(ChipValidator::validate_chip_clocks(&header).is_ok());

        // Invalid clock - too high
        header.sn76489_clock = 50_000_000; // Way too high
        assert!(ChipValidator::validate_chip_clocks(&header).is_err());
    }

    #[test]
    fn test_validation_config() {
        let config = ValidationConfig::default();
        assert!(config.min_vgm_version > 0);
        assert!(config.max_vgm_version > config.min_vgm_version);
        assert!(config.max_file_size > 1024);
    }
}
