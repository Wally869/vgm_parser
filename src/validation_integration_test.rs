#[cfg(test)]
mod integration_tests {
    use crate::validation::*;
    use crate::{Commands, Gd3LocaleData, HeaderData, VgmFile, VgmMetadata};
    use bytes::Bytes;

    #[test]
    fn test_validation_framework_integration() {
        // Test 1: Version validation (test directly)
        let config = ValidationConfig::default();

        // Should fail version validation - version too old
        assert!(VersionValidator::validate_version(0x00000050, &config).is_err());

        // Should pass version validation - valid version
        assert!(VersionValidator::validate_version(0x00000151, &config).is_ok());

        let context = ValidationContext {
            file_size: 1000,
            config: config.clone(),
        };

        // Test 2: Valid header should pass
        let mut valid_header = HeaderData::default();
        valid_header.version = 0x151; // Version 1.51
        valid_header.sn76489_clock = 3579545; // Valid PSG clock
        valid_header.rate = 44100; // Valid sample rate

        assert!(valid_header.validate(&context).is_ok());

        // Test 3: Invalid offset should fail
        let mut invalid_offset_header = HeaderData::default();
        invalid_offset_header.version = 0x151;
        invalid_offset_header.sn76489_clock = 3579545;
        invalid_offset_header.rate = 44100;
        invalid_offset_header.gd3_offset = 2000; // Beyond file size

        assert!(invalid_offset_header.validate(&context).is_err());

        // Test 4: Metadata validation
        let invalid_metadata = VgmMetadata {
            english_data: Gd3LocaleData {
                track: "A".repeat(2000), // Too long
                game: "Test Game".to_string(),
                system: "Test System".to_string(),
                author: "Test Author".to_string(),
            },
            japanese_data: Gd3LocaleData {
                track: "Test Track JP".to_string(),
                game: "Test Game JP".to_string(),
                system: "Test System JP".to_string(),
                author: "Test Author JP".to_string(),
            },
            date_release: "2023".to_string(),
            name_vgm_creator: "Test Creator".to_string(),
            notes: "Test Notes".to_string(),
        };

        // Should fail metadata validation
        assert!(invalid_metadata.validate(&context).is_err());

        // Test 5: Chip consistency validation
        let mut inconsistent_header = HeaderData::default();
        inconsistent_header.version = 0x151;
        inconsistent_header.sn76489_clock = 3579545;
        inconsistent_header.rate = 44100;
        inconsistent_header.ym2612_clock = 0; // No clock configured

        let commands_with_ym2612 = vec![Commands::YM2612Port0Write {
            register: 0x22,
            value: 0x00,
            chip_index: 0,
        }];

        // Should fail consistency validation
        assert!(ConsistencyValidator::validate_commands_consistency(
            &inconsistent_header,
            &commands_with_ym2612
        )
        .is_err());

        // Test 6: Valid consistency should pass
        inconsistent_header.ym2612_clock = 7670453; // Add clock configuration
        assert!(ConsistencyValidator::validate_commands_consistency(
            &inconsistent_header,
            &commands_with_ym2612
        )
        .is_ok());
    }

    #[test]
    fn test_validation_config_limits() {
        let strict_config = ValidationConfig {
            min_vgm_version: 0x00000150, // Version 1.50+
            max_vgm_version: 0x00000160, // Version 1.60 max
            max_file_size: 1024,         // 1KB limit
            max_commands: 100,           // 100 commands max
            max_data_block_size: 1024,   // 1KB data blocks max
            strict_mode: true,
        };

        // Test file size limit
        let large_file_data = vec![0u8; 2048]; // 2KB file
        let mut data = Bytes::from(large_file_data);

        // Should fail due to size limit (would fail earlier in practice)
        let result = VgmFile::from_bytes_validated(&mut data, strict_config.clone());
        // This will fail at parsing stage due to invalid data, but demonstrates the concept
        assert!(result.is_err());

        // Test version range
        assert!(VersionValidator::validate_version(0x140, &strict_config).is_err()); // Too old
        assert!(VersionValidator::validate_version(0x151, &strict_config).is_ok()); // Valid
        assert!(VersionValidator::validate_version(0x170, &strict_config).is_err());
        // Too new
    }

    #[test]
    fn test_chip_validator_edge_cases() {
        let mut header = HeaderData::default();

        // Test extreme clock values
        header.sn76489_clock = 1000; // Too low
        assert!(ChipValidator::validate_chip_clocks(&header).is_err());

        header.sn76489_clock = 100_000_000; // Too high
        assert!(ChipValidator::validate_chip_clocks(&header).is_err());

        header.sn76489_clock = 3579545; // Valid
        assert!(ChipValidator::validate_chip_clocks(&header).is_ok());

        // Test volume modifier limits
        header.volume_modifier = 100; // Too high
        assert!(ChipValidator::validate_chip_volumes(&header).is_err());

        header.volume_modifier = 32; // Valid
        assert!(ChipValidator::validate_chip_volumes(&header).is_ok());
    }

    #[test]
    fn test_offset_validator_edge_cases() {
        // Test offset beyond file size
        assert!(OffsetValidator::validate_offset(1000, 500, "test").is_err());

        // Test overflow in range validation
        assert!(OffsetValidator::validate_range(u32::MAX - 10, 20, 1000, "test").is_err());

        // Test valid small ranges
        assert!(OffsetValidator::validate_range(100, 50, 1000, "test").is_ok());

        // Test range that goes beyond file size
        assert!(OffsetValidator::validate_range(950, 100, 1000, "test").is_err());
    }

    #[test]
    fn test_comprehensive_vgm_validation() {
        // Create a complete VGM file structure for testing
        let header = HeaderData {
            version: 0x151,
            sn76489_clock: 3579545,
            ym2612_clock: 7670453,
            rate: 44100,
            total_nb_samples: 44100 * 60, // 1 minute at 44.1kHz
            gd3_offset: 0x100,
            vgm_data_offset: 0x40,
            ..HeaderData::default()
        };

        let metadata = VgmMetadata {
            english_data: Gd3LocaleData {
                track: "Test Track".to_string(),
                game: "Test Game".to_string(),
                system: "Genesis".to_string(),
                author: "Test Author".to_string(),
            },
            japanese_data: Gd3LocaleData {
                track: "テストトラック".to_string(),
                game: "テストゲーム".to_string(),
                system: "メガドライブ".to_string(),
                author: "テスト作者".to_string(),
            },
            date_release: "2023".to_string(),
            name_vgm_creator: "VGM Parser Test".to_string(),
            notes: "This is a test VGM file".to_string(),
        };

        let commands = vec![
            Commands::PSGWrite {
                value: 0x9F,
                chip_index: 0,
            },
            Commands::YM2612Port0Write {
                register: 0x22,
                value: 0x00,
                chip_index: 0,
            },
            Commands::Wait735Samples,
            Commands::EndOfSoundData,
        ];

        let validator = VgmValidator::default();
        let file_size = 1024;

        // Should pass comprehensive validation
        assert!(validator
            .validate_vgm_file(&header, &commands, &metadata, file_size)
            .is_ok());

        // Test with strict config
        let strict_config = ValidationConfig {
            strict_mode: true,
            ..ValidationConfig::default()
        };

        let strict_validator = VgmValidator::new(strict_config);
        assert!(strict_validator
            .validate_vgm_file(&header, &commands, &metadata, file_size)
            .is_ok());
    }
}
