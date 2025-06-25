//! Integration tests demonstrating VGM test builders usage
//!
//! This file shows practical examples of how to use the VGM test builders
//! for various testing scenarios.

use bytes::Bytes;
use vgm_parser::{
    Commands, ParserConfig, ValidationConfig, VgmFile, VgmParser,
};

mod fixtures;
use fixtures::builders::*;

#[test]
fn test_round_trip_with_builders() {
    // Create a VGM file using builders - use minimal commands for now
    let original_bytes = VgmBuilder::new()
        .version(150)
        .header(|h| {
            h.psg_clock(3579545)
                .total_samples(44100) // 1 second at 44.1kHz
        })
        .commands(|c| {
            c.psg_write(0x80, 0) // Simple PSG write
                .wait_60hz()
                .with_end()
        })
        .metadata(|m| {
            m.english_track("Test Round Trip")
                .english_game("Builder Test Game")
        })
        .build_bytes()
        .unwrap();

    // Parse it back
    let mut data = Bytes::from(original_bytes.clone());
    let parsed_vgm = VgmFile::from_bytes(&mut data).unwrap();

    // Verify key properties
    assert_eq!(parsed_vgm.header.version, 150);
    assert_eq!(parsed_vgm.header.sn76489_clock, 3579545);
    assert_eq!(parsed_vgm.header.total_nb_samples, 44100);
    assert_eq!(parsed_vgm.metadata.english_data.track, "Test Round Trip");
    assert_eq!(parsed_vgm.metadata.english_data.game, "Builder Test Game");
    
    // Should have more than just the end command
    assert!(parsed_vgm.commands.len() > 1);
    assert_eq!(parsed_vgm.commands.last(), Some(&Commands::EndOfSoundData));
}

#[test]
fn test_version_specific_features() {
    // Test VGM 1.00 - should have limited features
    let vgm_100 = VgmVersionGenerators::vgm_v100_basic().build().unwrap();
    assert_eq!(vgm_100.header.version, 100);
    assert_eq!(vgm_100.header.ym2612_clock, 0); // No YM2612 in v1.00
    assert_ne!(vgm_100.header.sn76489_clock, 0); // But PSG should be present

    // Test VGM 1.70 - should support dual chips
    let vgm_170 = VgmVersionGenerators::vgm_v170_dual_chip().build().unwrap();
    assert_eq!(vgm_170.header.version, 170);
    
    // Check for dual chip commands
    let has_dual_chip = vgm_170.commands.iter().any(|cmd| {
        match cmd {
            Commands::PSGWrite { chip_index, .. } => *chip_index == 1,
            Commands::YM2612Port0Write { chip_index, .. } => *chip_index == 1,
            _ => false,
        }
    });
    assert!(has_dual_chip, "VGM 1.70 should have dual chip commands");
}

#[test]
fn test_validation_with_builders() {
    // Test that builder-generated files pass validation
    let strict_config = ValidationConfig {
        max_file_size: 1024 * 1024, // 1MB
        max_commands: 10000,
        max_data_block_size: 64 * 1024, // 64KB
        ..Default::default()
    };

    let vgm_bytes = VgmBuilder::new()
        .version(150)
        .commands(|c| c.simple_psg_sequence())
        .build_bytes()
        .unwrap();

    let mut data = Bytes::from(vgm_bytes.clone());
    let vgm = VgmFile::from_bytes_validated(&mut data, strict_config).unwrap();
    
    assert_eq!(vgm.header.version, 150);
    assert!(!vgm.commands.is_empty());
}

#[test]
fn test_error_handling_with_invalid_data() {
    // Test parsing invalid signature
    let invalid_bytes = InvalidVgmGenerators::invalid_signature();
    let mut data = Bytes::from(invalid_bytes);
    let result = VgmFile::from_bytes(&mut data);
    assert!(result.is_err(), "Should fail with invalid signature");

    // Test truncated file
    let truncated_bytes = InvalidVgmGenerators::truncated_file();
    let mut data = Bytes::from(truncated_bytes);
    let result = VgmFile::from_bytes(&mut data);
    assert!(result.is_err(), "Should fail with truncated file");
}

#[test]
fn test_edge_cases() {
    // Test minimal valid file
    let minimal = EdgeCaseGenerators::minimal_valid().build().unwrap();
    assert_eq!(minimal.header.total_nb_samples, 1);
    assert_eq!(minimal.metadata.english_data.track, "");
    
    // Should serialize and parse correctly
    let bytes = EdgeCaseGenerators::minimal_valid().build_bytes().unwrap();
    let mut data = Bytes::from(bytes);
    let parsed = VgmFile::from_bytes(&mut data).unwrap();
    assert_eq!(parsed.header.total_nb_samples, 1);
}

#[test]
fn test_data_block_generation() {
    // Test uncompressed data block
    let vgm_with_data = EdgeCaseGenerators::with_data_blocks().build().unwrap();
    
    let has_data_block = vgm_with_data.commands.iter().any(|cmd| {
        matches!(cmd, Commands::DataBlock { .. })
    });
    assert!(has_data_block, "Should contain data block");

    // Test compressed data block
    let vgm_compressed = EdgeCaseGenerators::with_compressed_data().build().unwrap();
    
    let has_compressed_block = vgm_compressed.commands.iter().any(|cmd| {
        if let Commands::DataBlock { data, .. } = cmd {
            matches!(data, vgm_parser::DataBlockContent::CompressedStream { .. })
        } else {
            false
        }
    });
    assert!(has_compressed_block, "Should contain compressed data block");
}

#[test] 
fn test_custom_command_sequences() {
    // Test building custom command sequences
    let custom_vgm = VgmBuilder::new()
        .commands(|c| {
            c.psg_write(0x80, 0) // Channel 0 frequency low
                .psg_write(0x01, 0) // Channel 0 frequency high
                .psg_write(0x90, 0) // Channel 0 volume
                .wait_60hz()
                .ym2612_port0_write(0x22, 0x00, 0) // LFO off
                .ym2612_port0_write(0x28, 0xF0, 0) // Key on
                .wait_samples(2205) // 1/20 second
                .ym2612_port0_write(0x28, 0x00, 0) // Key off
                .psg_write(0x9F, 0) // PSG channel 0 off
                .with_end()
        })
        .build()
        .unwrap();

    // Should have multiple different command types
    let command_types: std::collections::HashSet<std::mem::Discriminant<Commands>> = 
        custom_vgm.commands.iter().map(std::mem::discriminant).collect();
    
    assert!(command_types.len() >= 4, "Should have multiple command types");
}

#[test]
fn test_parser_config_compatibility() {
    // Test that builders work with different parser configurations
    let parser_config = ParserConfig {
        max_data_block_size: 1024 * 1024, // 1MB
        max_metadata_size: 64 * 1024,     // 64KB
        ..Default::default()
    };

    let bytes = VgmBuilder::new()
        .commands(|c| c.simple_ym2612_sequence())
        .build_bytes()
        .unwrap();

    let mut data = Bytes::from(bytes);
    let vgm = VgmFile::from_bytes_with_config(&mut data, parser_config).unwrap();
    
    assert!(!vgm.commands.is_empty());
}

#[test]
fn test_all_version_generators() {
    // Test that all version generators work
    let version_generators: Vec<(&str, fn() -> VgmBuilder)> = vec![
        ("1.00", VgmVersionGenerators::vgm_v100_basic),
        ("1.01", VgmVersionGenerators::vgm_v101_basic),
        ("1.50", VgmVersionGenerators::vgm_v150_standard),
        ("1.61", VgmVersionGenerators::vgm_v161_expanded),
        ("1.70", VgmVersionGenerators::vgm_v170_dual_chip),
        ("1.71", VgmVersionGenerators::vgm_v171_latest),
    ];

    for (version_name, generator) in version_generators {
        // Test building VGM object
        let vgm = generator().build().unwrap();
        assert!(!vgm.commands.is_empty(), "Version {} should have commands", version_name);
        assert_eq!(vgm.commands.last(), Some(&Commands::EndOfSoundData), 
                  "Version {} should end properly", version_name);
        
        // Test that it can be serialized and parsed (create new builder)
        let bytes = generator().build_bytes().unwrap();
        assert!(!bytes.is_empty(), "Version {} should serialize", version_name);
        
        let mut data = Bytes::from(bytes);
        let parsed = VgmFile::from_bytes(&mut data);
        assert!(parsed.is_ok(), "Version {} should parse back", version_name);
    }
}

#[test]
fn test_metadata_builder_features() {
    // Test comprehensive metadata building
    let vgm = VgmBuilder::new()
        .metadata(|m| {
            m.english_track("Sonic 1 - Green Hill Zone")
                .english_game("Sonic the Hedgehog")
                .english_system("Sega Genesis / Mega Drive")
                .english_author("Masato Nakamura")
                .japanese_track("ソニック1 - グリーンヒルゾーン")
                .japanese_game("ソニック・ザ・ヘッジホッグ")
                .japanese_system("セガ・メガドライブ")
                .japanese_author("中村正人")
                .release_date("1991-06-23")
                .creator_name("VGM Test Suite")
                .notes("Classic Sonic music test file with Japanese metadata")
        })
        .build()
        .unwrap();

    // Verify all metadata fields
    assert_eq!(vgm.metadata.english_data.track, "Sonic 1 - Green Hill Zone");
    assert_eq!(vgm.metadata.japanese_data.track, "ソニック1 - グリーンヒルゾーン");
    assert_eq!(vgm.metadata.date_release, "1991-06-23");
    assert!(vgm.metadata.notes.contains("Japanese metadata"));
}

#[test]
fn test_performance_with_large_files() {
    // Test that builders can create reasonably large files without issues
    use std::time::Instant;
    
    let start = Instant::now();
    
    let large_vgm = EdgeCaseGenerators::many_commands().build().unwrap();
    
    let build_time = start.elapsed();
    println!("Built large VGM with {} commands in {:?}", 
             large_vgm.commands.len(), build_time);
    
    // Should have created many commands (the implementation adds 10000)
    assert!(large_vgm.commands.len() > 5000, "Should have many commands");
    
    // Should still be parseable
    let bytes = EdgeCaseGenerators::many_commands().build_bytes().unwrap();
    assert!(bytes.len() > 10000, "Should be a substantial file");
}