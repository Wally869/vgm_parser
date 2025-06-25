#[cfg(test)]
mod tests {
    use crate::vgm_commands::compression::{decompress_bit_packing, decompress_dpcm, BitReader};
    use crate::{Commands, CompressionType, DataBlockContent, StreamChipType, VgmError, ParserConfig, ResourceTracker};
    use bytes::{Buf, BufMut, Bytes, BytesMut};

    #[test]
    fn test_dual_chip_psg_support() {
        // Test PSG first chip (0x50)
        let mut data1 = Bytes::from(vec![0x50, 0xAB]);
        let cmd1 = Commands::from_bytes(&mut data1).unwrap();
        if let Commands::PSGWrite { value, chip_index } = cmd1 {
            assert_eq!(value, 0xAB);
            assert_eq!(chip_index, 0);
        } else {
            panic!("Expected PSGWrite command");
        }

        // Test PSG second chip (0x30)
        let mut data2 = Bytes::from(vec![0x30, 0xCD]);
        let cmd2 = Commands::from_bytes(&mut data2).unwrap();
        if let Commands::PSGWrite { value, chip_index } = cmd2 {
            assert_eq!(value, 0xCD);
            assert_eq!(chip_index, 1);
        } else {
            panic!("Expected PSGWrite command");
        }

        // Test Game Gear PSG stereo first chip (0x4F)
        let mut data3 = Bytes::from(vec![0x4F, 0xEF]);
        let cmd3 = Commands::from_bytes(&mut data3).unwrap();
        if let Commands::GameGearPSGStereo { value, chip_index } = cmd3 {
            assert_eq!(value, 0xEF);
            assert_eq!(chip_index, 0);
        } else {
            panic!("Expected GameGearPSGStereo command");
        }

        // Test Game Gear PSG stereo second chip (0x3F)
        let mut data4 = Bytes::from(vec![0x3F, 0x12]);
        let cmd4 = Commands::from_bytes(&mut data4).unwrap();
        if let Commands::GameGearPSGStereo { value, chip_index } = cmd4 {
            assert_eq!(value, 0x12);
            assert_eq!(chip_index, 1);
        } else {
            panic!("Expected GameGearPSGStereo command");
        }
    }

    #[test]
    fn test_dual_chip_psg_serialization() {
        // Test PSG first chip serialization
        let cmd1 = Commands::PSGWrite {
            value: 0xAB,
            chip_index: 0,
        };
        let bytes1 = cmd1.to_bytes().unwrap();
        assert_eq!(bytes1, vec![0x50, 0xAB]);

        // Test PSG second chip serialization
        let cmd2 = Commands::PSGWrite {
            value: 0xCD,
            chip_index: 1,
        };
        let bytes2 = cmd2.to_bytes().unwrap();
        assert_eq!(bytes2, vec![0x30, 0xCD]);

        // Test Game Gear PSG stereo first chip serialization
        let cmd3 = Commands::GameGearPSGStereo {
            value: 0xEF,
            chip_index: 0,
        };
        let bytes3 = cmd3.to_bytes().unwrap();
        assert_eq!(bytes3, vec![0x4F, 0xEF]);

        // Test Game Gear PSG stereo second chip serialization
        let cmd4 = Commands::GameGearPSGStereo {
            value: 0x12,
            chip_index: 1,
        };
        let bytes4 = cmd4.to_bytes().unwrap();
        assert_eq!(bytes4, vec![0x3F, 0x12]);
    }

    #[test]
    fn test_dual_chip_psg_invalid_chip_index() {
        // Test invalid chip_index for PSGWrite
        let cmd1 = Commands::PSGWrite {
            value: 0xAB,
            chip_index: 2,
        };
        let result1 = cmd1.to_bytes();
        assert!(result1.is_err());

        // Test invalid chip_index for GameGearPSGStereo
        let cmd2 = Commands::GameGearPSGStereo {
            value: 0xCD,
            chip_index: 255,
        };
        let result2 = cmd2.to_bytes();
        assert!(result2.is_err());
    }

    #[test]
    fn test_dual_chip_ym_parsing_first_chip() {
        // Test all YM family first chip commands (0x51-0x5F)
        let test_cases = vec![
            (0x51, "YM2413"),
            (0x52, "YM2612Port0"),
            (0x53, "YM2612Port1"),
            (0x54, "YM2151"),
            (0x55, "YM2203"),
            (0x56, "YM2608Port0"),
            (0x57, "YM2608Port1"),
            (0x58, "YM2610Port0"),
            (0x59, "YM2610Port1"),
            (0x5A, "YM3812"),
            (0x5B, "YM3526"),
            (0x5C, "Y8950"),
            (0x5D, "YMZ280B"),
            (0x5E, "YMF262Port0"),
            (0x5F, "YMF262Port1"),
        ];

        for (opcode, name) in test_cases {
            let mut bytes = Bytes::from(vec![opcode, 0x42, 0x73]); // register=0x42, value=0x73
            let result = Commands::from_bytes(&mut bytes);
            assert!(
                result.is_ok(),
                "Failed to parse {} first chip command",
                name
            );

            // Verify chip_index is 0 for first chip
            match result.unwrap() {
                Commands::YM2413Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 0);
                },
                Commands::YM2612Port0Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 0);
                },
                Commands::YM2612Port1Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 0);
                },
                Commands::YM2151Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 0);
                },
                Commands::YM2203Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 0);
                },
                Commands::YM2608Port0Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 0);
                },
                Commands::YM2608Port1Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 0);
                },
                Commands::YM2610Port0Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 0);
                },
                Commands::YM2610Port1Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 0);
                },
                Commands::YM3812Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 0);
                },
                Commands::YM3526Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 0);
                },
                Commands::Y8950Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 0);
                },
                Commands::YMZ280BWrite {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 0);
                },
                Commands::YMF262Port0Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 0);
                },
                Commands::YMF262Port1Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 0);
                },
                _ => panic!("Unexpected command type for opcode 0x{:02X}", opcode),
            }
        }
    }

    #[test]
    fn test_dual_chip_ym_parsing_second_chip() {
        // Test all YM family second chip commands (0xA1-0xAF)
        let test_cases = vec![
            (0xA1, "YM2413"),
            (0xA2, "YM2612Port0"),
            (0xA3, "YM2612Port1"),
            (0xA4, "YM2151"),
            (0xA5, "YM2203"),
            (0xA6, "YM2608Port0"),
            (0xA7, "YM2608Port1"),
            (0xA8, "YM2610Port0"),
            (0xA9, "YM2610Port1"),
            (0xAA, "YM3812"),
            (0xAB, "YM3526"),
            (0xAC, "Y8950"),
            (0xAD, "YMZ280B"),
            (0xAE, "YMF262Port0"),
            (0xAF, "YMF262Port1"),
        ];

        for (opcode, name) in test_cases {
            let mut bytes = Bytes::from(vec![opcode, 0x42, 0x73]); // register=0x42, value=0x73
            let result = Commands::from_bytes(&mut bytes);
            assert!(
                result.is_ok(),
                "Failed to parse {} second chip command",
                name
            );

            // Verify chip_index is 1 for second chip
            match result.unwrap() {
                Commands::YM2413Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 1);
                },
                Commands::YM2612Port0Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 1);
                },
                Commands::YM2612Port1Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 1);
                },
                Commands::YM2151Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 1);
                },
                Commands::YM2203Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 1);
                },
                Commands::YM2608Port0Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 1);
                },
                Commands::YM2608Port1Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 1);
                },
                Commands::YM2610Port0Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 1);
                },
                Commands::YM2610Port1Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 1);
                },
                Commands::YM3812Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 1);
                },
                Commands::YM3526Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 1);
                },
                Commands::Y8950Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 1);
                },
                Commands::YMZ280BWrite {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 1);
                },
                Commands::YMF262Port0Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 1);
                },
                Commands::YMF262Port1Write {
                    register,
                    value,
                    chip_index,
                } => {
                    assert_eq!(register, 0x42);
                    assert_eq!(value, 0x73);
                    assert_eq!(chip_index, 1);
                },
                _ => panic!("Unexpected command type for opcode 0x{:02X}", opcode),
            }
        }
    }

    #[test]
    fn test_dual_chip_ym_serialization() {
        // Test serialization of YM commands with chip_index
        let test_commands = vec![
            // First chip commands (should serialize to 0x5n)
            (
                Commands::YM2413Write {
                    register: 0x42,
                    value: 0x73,
                    chip_index: 0,
                },
                vec![0x51, 0x42, 0x73],
            ),
            (
                Commands::YM2612Port0Write {
                    register: 0x42,
                    value: 0x73,
                    chip_index: 0,
                },
                vec![0x52, 0x42, 0x73],
            ),
            (
                Commands::YM2151Write {
                    register: 0x42,
                    value: 0x73,
                    chip_index: 0,
                },
                vec![0x54, 0x42, 0x73],
            ),
            // Second chip commands (should serialize to 0xAn)
            (
                Commands::YM2413Write {
                    register: 0x42,
                    value: 0x73,
                    chip_index: 1,
                },
                vec![0xA1, 0x42, 0x73],
            ),
            (
                Commands::YM2612Port0Write {
                    register: 0x42,
                    value: 0x73,
                    chip_index: 1,
                },
                vec![0xA2, 0x42, 0x73],
            ),
            (
                Commands::YM2151Write {
                    register: 0x42,
                    value: 0x73,
                    chip_index: 1,
                },
                vec![0xA4, 0x42, 0x73],
            ),
        ];

        for (command, expected_bytes) in test_commands {
            let result = command.clone().to_bytes();
            assert!(result.is_ok(), "Failed to serialize command: {:?}", command);
            assert_eq!(
                result.unwrap(),
                expected_bytes,
                "Serialization mismatch for command: {:?}",
                command
            );
        }
    }

    #[test]
    fn test_dual_chip_ym_round_trip() {
        // Test round-trip parsing and serialization for all YM commands
        let test_data = vec![
            // First chip commands
            vec![0x51, 0x42, 0x73], // YM2413
            vec![0x52, 0x42, 0x73], // YM2612Port0
            vec![0x53, 0x42, 0x73], // YM2612Port1
            vec![0x54, 0x42, 0x73], // YM2151
            vec![0x55, 0x42, 0x73], // YM2203
            vec![0x56, 0x42, 0x73], // YM2608Port0
            vec![0x57, 0x42, 0x73], // YM2608Port1
            vec![0x58, 0x42, 0x73], // YM2610Port0
            vec![0x59, 0x42, 0x73], // YM2610Port1
            vec![0x5A, 0x42, 0x73], // YM3812
            vec![0x5B, 0x42, 0x73], // YM3526
            vec![0x5C, 0x42, 0x73], // Y8950
            vec![0x5D, 0x42, 0x73], // YMZ280B
            vec![0x5E, 0x42, 0x73], // YMF262Port0
            vec![0x5F, 0x42, 0x73], // YMF262Port1
            // Second chip commands
            vec![0xA1, 0x42, 0x73], // YM2413
            vec![0xA2, 0x42, 0x73], // YM2612Port0
            vec![0xA3, 0x42, 0x73], // YM2612Port1
            vec![0xA4, 0x42, 0x73], // YM2151
            vec![0xA5, 0x42, 0x73], // YM2203
            vec![0xA6, 0x42, 0x73], // YM2608Port0
            vec![0xA7, 0x42, 0x73], // YM2608Port1
            vec![0xA8, 0x42, 0x73], // YM2610Port0
            vec![0xA9, 0x42, 0x73], // YM2610Port1
            vec![0xAA, 0x42, 0x73], // YM3812
            vec![0xAB, 0x42, 0x73], // YM3526
            vec![0xAC, 0x42, 0x73], // Y8950
            vec![0xAD, 0x42, 0x73], // YMZ280B
            vec![0xAE, 0x42, 0x73], // YMF262Port0
            vec![0xAF, 0x42, 0x73], // YMF262Port1
        ];

        for original_bytes in test_data {
            let mut bytes = Bytes::from(original_bytes.clone());

            // Parse the command
            let parsed_command = Commands::from_bytes(&mut bytes);
            assert!(
                parsed_command.is_ok(),
                "Failed to parse bytes: {:?}",
                original_bytes
            );

            // Serialize the command back
            let serialized_bytes = parsed_command.unwrap().to_bytes();
            assert!(
                serialized_bytes.is_ok(),
                "Failed to serialize parsed command"
            );

            // Verify round-trip integrity
            assert_eq!(
                serialized_bytes.unwrap(),
                original_bytes,
                "Round-trip failed for bytes: {:?}",
                original_bytes
            );
        }
    }

    #[test]
    fn test_bit_packing_copy_mode() {
        // Test bit packing with copy mode (sub_type = 0x00)
        let compressed_data = vec![0b10101010, 0b11001100]; // 8-bit values
        let result = decompress_bit_packing(
            &compressed_data,
            8,    // bits_compressed
            16,   // bits_decompressed
            0x00, // sub_type: copy
            100,  // add_value
            4,    // uncompressed_size (2 16-bit values = 4 bytes)
            None,
        )
        .unwrap();

        // First value: 0b10101010 + 100 = 170 + 100 = 270 = 0x010E (little-endian: 0x0E, 0x01)
        // Second value: 0b11001100 + 100 = 204 + 100 = 304 = 0x0130 (little-endian: 0x30, 0x01)
        assert_eq!(result, vec![0x0E, 0x01, 0x30, 0x01]);
    }

    #[test]
    fn test_bit_packing_shift_mode() {
        // Test bit packing with shift left mode (sub_type = 0x01)
        // Using 8-bit compressed data that contains a 4-bit value
        let compressed_data = vec![0b11110000]; // MSB first: 0b1111 = 15
        let result = decompress_bit_packing(
            &compressed_data,
            4,    // bits_compressed
            8,    // bits_decompressed
            0x01, // sub_type: shift left
            10,   // add_value
            1,    // uncompressed_size (1 byte)
            None,
        )
        .unwrap();

        // BitReader reads MSB first, so from 0b11110000 with 4 bits we get 0b1111 = 15
        // Value: 15 << (8-4) = 15 << 4 = 0b11110000 = 240
        // Final: 240 + 10 = 250
        assert_eq!(result, vec![250]);
    }

    #[test]
    fn test_bit_packing_table_mode() {
        // Test bit packing with table lookup mode (sub_type = 0x02)
        let compressed_data = vec![0x00, 0x01]; // Two 8-bit indices
        let table = vec![
            0x34, 0x12, // Entry 0: 0x1234 (little-endian)
            0x78, 0x56, // Entry 1: 0x5678 (little-endian)
        ];

        let result = decompress_bit_packing(
            &compressed_data,
            8,    // bits_compressed
            16,   // bits_decompressed
            0x02, // sub_type: use table
            0,    // add_value (ignored for table mode)
            4,    // uncompressed_size (2 16-bit values = 4 bytes)
            Some(&table),
        )
        .unwrap();

        assert_eq!(result, vec![0x34, 0x12, 0x78, 0x56]);
    }

    #[test]
    fn test_dpcm_decompression() {
        // Test DPCM decompression
        let compressed_data = vec![0b00011011]; // Contains indices 0, 1, 2, 3 (2 bits each)
        let table = vec![
            0x00, 0x00, // Delta 0: 0
            0x01, 0x00, // Delta 1: 1
            0xFF, 0xFF, // Delta 2: -1 (two's complement)
            0x02, 0x00, // Delta 3: 2
        ];

        let result = decompress_dpcm(
            &compressed_data,
            2,   // bits_compressed
            16,  // bits_decompressed
            100, // start_value
            8,   // uncompressed_size (4 16-bit values = 8 bytes)
            &table,
        )
        .unwrap();

        // Starting value: 100
        // After delta 0 (+0): 100 = 0x0064 (little-endian: 0x64, 0x00)
        // After delta 1 (+1): 101 = 0x0065 (little-endian: 0x65, 0x00)
        // After delta 2 (-1): 100 = 0x0064 (little-endian: 0x64, 0x00)
        // After delta 3 (+2): 102 = 0x0066 (little-endian: 0x66, 0x00)
        assert_eq!(result, vec![0x64, 0x00, 0x65, 0x00, 0x64, 0x00, 0x66, 0x00]);
    }

    #[test]
    fn test_data_block_parsing_compressed() {
        let mut bytes = BytesMut::new();

        // Compressed stream block type 0x40 (YM2612)
        let block_type = 0x40;
        let data_size = 16; // 10 bytes header + 6 bytes data

        // Compression header
        bytes.put_u8(0x00); // Bit packing compression
        bytes.put_u32_le(100); // Uncompressed size
        bytes.put_u8(8); // bits_decompressed
        bytes.put_u8(4); // bits_compressed
        bytes.put_u8(0x00); // sub_type: copy
        bytes.put_u16_le(10); // add_value

        // Compressed data
        bytes.extend_from_slice(&[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);

        let mut bytes = bytes.freeze();
        let content =
            DataBlockContent::parse_from_bytes(block_type, data_size, &mut bytes).unwrap();

        match content {
            DataBlockContent::CompressedStream {
                chip_type,
                compression,
                uncompressed_size,
                data,
            } => {
                assert_eq!(chip_type, StreamChipType::YM2612);
                assert_eq!(uncompressed_size, 100);
                assert_eq!(data.len(), 6);

                match compression {
                    CompressionType::BitPacking {
                        bits_decompressed,
                        bits_compressed,
                        sub_type,
                        add_value,
                    } => {
                        assert_eq!(bits_decompressed, 8);
                        assert_eq!(bits_compressed, 4);
                        assert_eq!(sub_type, 0x00);
                        assert_eq!(add_value, 10);
                    },
                    _ => panic!("Expected BitPacking compression"),
                }
            },
            _ => panic!("Expected CompressedStream"),
        }
    }

    #[test]
    fn test_data_block_decompression() {
        // Create a compressed data block
        let content = DataBlockContent::CompressedStream {
            chip_type: StreamChipType::YM2612,
            compression: CompressionType::BitPacking {
                bits_decompressed: 8,
                bits_compressed: 8,
                sub_type: 0x00,
                add_value: 0,
            },
            uncompressed_size: 4,
            data: vec![0x10, 0x20, 0x30, 0x40],
        };

        let decompressed = content.decompress_data(None).unwrap();
        assert_eq!(decompressed, vec![0x10, 0x20, 0x30, 0x40]);
    }

    #[test]
    fn test_bit_reader() {
        let data = vec![0b10110100, 0b11001010];
        let mut reader = BitReader::new(&data);

        // Read 3 bits: should get 0b101
        assert_eq!(reader.read_bits(3).unwrap(), 0b101);

        // Read 5 bits: should get 0b10100
        assert_eq!(reader.read_bits(5).unwrap(), 0b10100);

        // Read 4 bits: should get 0b1100
        assert_eq!(reader.read_bits(4).unwrap(), 0b1100);

        // Read 4 bits: should get 0b1010
        assert_eq!(reader.read_bits(4).unwrap(), 0b1010);
    }

    #[test]
    fn test_decompression_table_block() {
        let mut bytes = BytesMut::new();

        // Decompression table block type 0x7F
        let block_type = 0x7F;
        let data_size = 10; // 6 bytes header + 4 bytes table data

        bytes.put_u8(0x00); // compression_type
        bytes.put_u8(0x00); // sub_type
        bytes.put_u8(16); // bits_decompressed
        bytes.put_u8(8); // bits_compressed
        bytes.put_u16_le(2); // value_count

        // Table data (2 16-bit values)
        bytes.extend_from_slice(&[0x34, 0x12, 0x78, 0x56]);

        let mut bytes = bytes.freeze();
        let content =
            DataBlockContent::parse_from_bytes(block_type, data_size, &mut bytes).unwrap();

        match content {
            DataBlockContent::DecompressionTable {
                compression_type,
                sub_type,
                bits_decompressed,
                bits_compressed,
                value_count,
                table_data,
            } => {
                assert_eq!(compression_type, 0x00);
                assert_eq!(sub_type, 0x00);
                assert_eq!(bits_decompressed, 16);
                assert_eq!(bits_compressed, 8);
                assert_eq!(value_count, 2);
                assert_eq!(table_data, vec![0x34, 0x12, 0x78, 0x56]);
            },
            _ => panic!("Expected DecompressionTable"),
        }
    }

    #[test]
    fn test_dual_chip_method2_parsing_first_chip() {
        // Test Method #2 dual chip support - first chip (bit 7 = 0)
        let mut test_data = BytesMut::new();

        // AY8910Write first chip: register 0x07, value 0x38
        test_data.put_u8(0xA0);
        test_data.put_u8(0x07); // Register 0x07, bit 7 = 0 -> first chip
        test_data.put_u8(0x38);

        // GameBoyDMGWrite first chip: register 0x40, value 0x80
        test_data.put_u8(0xB3);
        test_data.put_u8(0x40); // Register 0x40, bit 7 = 0 -> first chip
        test_data.put_u8(0x80);

        // NESAPUWrite first chip: register 0x15, value 0x0F
        test_data.put_u8(0xB4);
        test_data.put_u8(0x15); // Register 0x15, bit 7 = 0 -> first chip
        test_data.put_u8(0x0F);

        let mut bytes = test_data.freeze();

        // Parse first command
        let cmd1 = Commands::from_bytes(&mut bytes).unwrap();
        assert_eq!(
            cmd1,
            Commands::AY8910Write {
                register: 0x07,
                value: 0x38,
                chip_index: 0
            }
        );

        // Parse second command
        let cmd2 = Commands::from_bytes(&mut bytes).unwrap();
        assert_eq!(
            cmd2,
            Commands::GameBoyDMGWrite {
                register: 0x40,
                value: 0x80,
                chip_index: 0
            }
        );

        // Parse third command
        let cmd3 = Commands::from_bytes(&mut bytes).unwrap();
        assert_eq!(
            cmd3,
            Commands::NESAPUWrite {
                register: 0x15,
                value: 0x0F,
                chip_index: 0
            }
        );
    }

    #[test]
    fn test_dual_chip_method2_parsing_second_chip() {
        // Test Method #2 dual chip support - second chip (bit 7 = 1)
        let mut test_data = BytesMut::new();

        // AY8910Write second chip: register 0x07 with bit 7 set (0x87), value 0x38
        test_data.put_u8(0xA0);
        test_data.put_u8(0x87); // Register 0x07 | 0x80 -> second chip
        test_data.put_u8(0x38);

        // GameBoyDMGWrite second chip: register 0x40 with bit 7 set (0xC0), value 0x80
        test_data.put_u8(0xB3);
        test_data.put_u8(0xC0); // Register 0x40 | 0x80 -> second chip
        test_data.put_u8(0x80);

        // MultiPCMWrite second chip: register 0x10 with bit 7 set (0x90), value 0xFF
        test_data.put_u8(0xB5);
        test_data.put_u8(0x90); // Register 0x10 | 0x80 -> second chip
        test_data.put_u8(0xFF);

        let mut bytes = test_data.freeze();

        // Parse first command
        let cmd1 = Commands::from_bytes(&mut bytes).unwrap();
        assert_eq!(
            cmd1,
            Commands::AY8910Write {
                register: 0x07,
                value: 0x38,
                chip_index: 1
            }
        );

        // Parse second command
        let cmd2 = Commands::from_bytes(&mut bytes).unwrap();
        assert_eq!(
            cmd2,
            Commands::GameBoyDMGWrite {
                register: 0x40,
                value: 0x80,
                chip_index: 1
            }
        );

        // Parse third command
        let cmd3 = Commands::from_bytes(&mut bytes).unwrap();
        assert_eq!(
            cmd3,
            Commands::MultiPCMWrite {
                register: 0x10,
                value: 0xFF,
                chip_index: 1
            }
        );
    }

    #[test]
    fn test_dual_chip_method2_serialization() {
        // Test Method #2 dual chip serialization for both chips

        // First chip commands
        let ay8910_chip1 = Commands::AY8910Write {
            register: 0x0E,
            value: 0x3F,
            chip_index: 0,
        };
        let gameboy_chip1 = Commands::GameBoyDMGWrite {
            register: 0x26,
            value: 0x8F,
            chip_index: 0,
        };

        // Second chip commands
        let ay8910_chip2 = Commands::AY8910Write {
            register: 0x0E,
            value: 0x3F,
            chip_index: 1,
        };
        let pokey_chip2 = Commands::PokeyWrite {
            register: 0x08,
            value: 0xA0,
            chip_index: 1,
        };

        // Serialize and check results
        let bytes1 = ay8910_chip1.clone().to_bytes().unwrap();
        assert_eq!(bytes1, vec![0xA0, 0x0E, 0x3F]); // Register 0x0E (bit 7 = 0)

        let bytes2 = gameboy_chip1.clone().to_bytes().unwrap();
        assert_eq!(bytes2, vec![0xB3, 0x26, 0x8F]); // Register 0x26 (bit 7 = 0)

        let bytes3 = ay8910_chip2.clone().to_bytes().unwrap();
        assert_eq!(bytes3, vec![0xA0, 0x8E, 0x3F]); // Register 0x0E | 0x80 = 0x8E

        let bytes4 = pokey_chip2.clone().to_bytes().unwrap();
        assert_eq!(bytes4, vec![0xBB, 0x88, 0xA0]); // Register 0x08 | 0x80 = 0x88
    }

    #[test]
    fn test_dual_chip_method2_round_trip() {
        // Test Method #2 dual chip round-trip (parse then serialize)
        let test_commands = vec![
            Commands::AY8910Write {
                register: 0x07,
                value: 0x38,
                chip_index: 0,
            },
            Commands::AY8910Write {
                register: 0x07,
                value: 0x38,
                chip_index: 1,
            },
            Commands::GameBoyDMGWrite {
                register: 0x40,
                value: 0x80,
                chip_index: 0,
            },
            Commands::GameBoyDMGWrite {
                register: 0x40,
                value: 0x80,
                chip_index: 1,
            },
            Commands::NESAPUWrite {
                register: 0x15,
                value: 0x0F,
                chip_index: 0,
            },
            Commands::NESAPUWrite {
                register: 0x15,
                value: 0x0F,
                chip_index: 1,
            },
            Commands::HuC6280Write {
                register: 0x02,
                value: 0x44,
                chip_index: 0,
            },
            Commands::HuC6280Write {
                register: 0x02,
                value: 0x44,
                chip_index: 1,
            },
            Commands::PokeyWrite {
                register: 0x08,
                value: 0xA0,
                chip_index: 0,
            },
            Commands::PokeyWrite {
                register: 0x08,
                value: 0xA0,
                chip_index: 1,
            },
        ];

        for cmd in test_commands {
            // Serialize command to bytes
            let serialized = cmd.clone().to_bytes().unwrap();

            // Parse bytes back to command
            let mut bytes = Bytes::from(serialized);
            let parsed = Commands::from_bytes(&mut bytes).unwrap();

            // Should be identical
            assert_eq!(cmd, parsed);
        }
    }

    #[test]
    fn test_dac_stream_dual_chip_parsing() {
        // Test DAC Stream dual chip support - chip_type bit 7 determines chip
        let mut test_data = BytesMut::new();

        // DACStreamSetupControl first chip: stream_id 0x01, chip_type 0x02 (bit 7 = 0), port 0x00, command 0x01
        test_data.put_u8(0x90);
        test_data.put_u8(0x01); // stream_id
        test_data.put_u8(0x02); // chip_type, bit 7 = 0 -> first chip
        test_data.put_u8(0x00); // port
        test_data.put_u8(0x01); // command

        // DACStreamSetupControl second chip: stream_id 0x02, chip_type 0x02 with bit 7 set (0x82), port 0x01, command 0x02
        test_data.put_u8(0x90);
        test_data.put_u8(0x02); // stream_id
        test_data.put_u8(0x82); // chip_type 0x02 | 0x80 -> second chip
        test_data.put_u8(0x01); // port
        test_data.put_u8(0x02); // command

        let mut bytes = test_data.freeze();

        // Parse first command
        let cmd1 = Commands::from_bytes(&mut bytes).unwrap();
        assert_eq!(
            cmd1,
            Commands::DACStreamSetupControl {
                stream_id: 0x01,
                chip_type: 0x02,
                port: 0x00,
                command: 0x01,
                chip_index: 0
            }
        );

        // Parse second command
        let cmd2 = Commands::from_bytes(&mut bytes).unwrap();
        assert_eq!(
            cmd2,
            Commands::DACStreamSetupControl {
                stream_id: 0x02,
                chip_type: 0x02,
                port: 0x01,
                command: 0x02,
                chip_index: 1
            }
        );
    }

    #[test]
    fn test_dac_stream_dual_chip_serialization() {
        // Test DAC Stream dual chip serialization

        // First chip command
        let dac_chip1 = Commands::DACStreamSetupControl {
            stream_id: 0x01,
            chip_type: 0x05,
            port: 0x00,
            command: 0x01,
            chip_index: 0,
        };

        // Second chip command
        let dac_chip2 = Commands::DACStreamSetupControl {
            stream_id: 0x02,
            chip_type: 0x05,
            port: 0x01,
            command: 0x02,
            chip_index: 1,
        };

        // Serialize and check results
        let bytes1 = dac_chip1.clone().to_bytes().unwrap();
        assert_eq!(bytes1, vec![0x90, 0x01, 0x05, 0x00, 0x01]); // chip_type 0x05 (bit 7 = 0)

        let bytes2 = dac_chip2.clone().to_bytes().unwrap();
        assert_eq!(bytes2, vec![0x90, 0x02, 0x85, 0x01, 0x02]); // chip_type 0x05 | 0x80 = 0x85
    }

    #[test]
    fn test_dac_stream_dual_chip_round_trip() {
        // Test DAC Stream dual chip round-trip (parse then serialize)
        let test_commands = vec![
            Commands::DACStreamSetupControl {
                stream_id: 0x01,
                chip_type: 0x03,
                port: 0x00,
                command: 0x01,
                chip_index: 0,
            },
            Commands::DACStreamSetupControl {
                stream_id: 0x01,
                chip_type: 0x03,
                port: 0x00,
                command: 0x01,
                chip_index: 1,
            },
        ];

        for cmd in test_commands {
            // Serialize command to bytes
            let serialized = cmd.clone().to_bytes().unwrap();

            // Parse bytes back to command
            let mut bytes = Bytes::from(serialized);
            let parsed = Commands::from_bytes(&mut bytes).unwrap();

            // Should be identical
            assert_eq!(cmd, parsed);
        }
    }

    #[test]
    fn test_dac_stream_all_commands_serialization() {
        // Test all DAC Stream commands serialization to ensure they all work
        let test_commands = vec![
            Commands::DACStreamSetupControl {
                stream_id: 0x01,
                chip_type: 0x02,
                port: 0x00,
                command: 0x01,
                chip_index: 0,
            },
            Commands::DACStreamSetData {
                stream_id: 0x01,
                data_bank_id: 0x00,
                step_size: 0x01,
                step_base: 0x00,
            },
            Commands::DACStreamSetFrequency {
                stream_id: 0x01,
                frequency: 44100,
            },
            Commands::DACStreamStart {
                stream_id: 0x01,
                data_start_offset: 0x1000,
                length_mode: 0x00,
                data_length: 0x2000,
            },
            Commands::DACStreamStop { stream_id: 0x01 },
            Commands::DACStreamStartFast {
                stream_id: 0x01,
                block_id: 0x0001,
                flags: 0x00,
            },
        ];

        for cmd in test_commands {
            // All commands should serialize without error
            let result = cmd.clone().to_bytes();
            assert!(
                result.is_ok(),
                "Failed to serialize DAC Stream command: {:?}",
                cmd
            );
        }
    }

    #[test]
    fn test_comprehensive_dual_chip_integration() {
        // Test all dual chip methods working together in a single VGM stream
        let mut test_data = BytesMut::new();

        // Method #1: PSG dual chip (0x50 -> 0x30)
        test_data.put_u8(0x50); // PSG first chip
        test_data.put_u8(0x9F);
        test_data.put_u8(0x30); // PSG second chip
        test_data.put_u8(0x9F);

        // Method #1: YM dual chip (0x51 -> 0xA1)
        test_data.put_u8(0x51); // YM2413 first chip
        test_data.put_u8(0x30);
        test_data.put_u8(0x14);
        test_data.put_u8(0xA1); // YM2413 second chip
        test_data.put_u8(0x30);
        test_data.put_u8(0x14);

        // Method #2: Bit 7 checking (AY8910)
        test_data.put_u8(0xA0); // AY8910
        test_data.put_u8(0x07); // Register 0x07, first chip
        test_data.put_u8(0x38);
        test_data.put_u8(0xA0); // AY8910
        test_data.put_u8(0x87); // Register 0x07 | 0x80, second chip
        test_data.put_u8(0x38);

        // DAC Stream dual chip
        test_data.put_u8(0x90); // DAC Stream Setup
        test_data.put_u8(0x01);
        test_data.put_u8(0x02); // chip_type 0x02, first chip
        test_data.put_u8(0x00);
        test_data.put_u8(0x01);
        test_data.put_u8(0x90); // DAC Stream Setup
        test_data.put_u8(0x02);
        test_data.put_u8(0x82); // chip_type 0x02 | 0x80, second chip
        test_data.put_u8(0x01);
        test_data.put_u8(0x02);

        // End of sound data
        test_data.put_u8(0x66);

        let mut bytes = test_data.freeze();

        // Parse all commands
        let mut commands = Vec::new();
        while bytes.remaining() > 0 {
            let cmd = Commands::from_bytes(&mut bytes).unwrap();
            if matches!(cmd, Commands::EndOfSoundData) {
                commands.push(cmd);
                break;
            }
            commands.push(cmd);
        }

        // Verify all commands were parsed correctly
        assert_eq!(commands.len(), 9); // 8 chip commands + 1 end command

        // Verify specific commands
        assert_eq!(
            commands[0],
            Commands::PSGWrite {
                value: 0x9F,
                chip_index: 0
            }
        );
        assert_eq!(
            commands[1],
            Commands::PSGWrite {
                value: 0x9F,
                chip_index: 1
            }
        );
        assert_eq!(
            commands[2],
            Commands::YM2413Write {
                register: 0x30,
                value: 0x14,
                chip_index: 0
            }
        );
        assert_eq!(
            commands[3],
            Commands::YM2413Write {
                register: 0x30,
                value: 0x14,
                chip_index: 1
            }
        );
        assert_eq!(
            commands[4],
            Commands::AY8910Write {
                register: 0x07,
                value: 0x38,
                chip_index: 0
            }
        );
        assert_eq!(
            commands[5],
            Commands::AY8910Write {
                register: 0x07,
                value: 0x38,
                chip_index: 1
            }
        );
        assert_eq!(
            commands[6],
            Commands::DACStreamSetupControl {
                stream_id: 0x01,
                chip_type: 0x02,
                port: 0x00,
                command: 0x01,
                chip_index: 0
            }
        );
        assert_eq!(
            commands[7],
            Commands::DACStreamSetupControl {
                stream_id: 0x02,
                chip_type: 0x02,
                port: 0x01,
                command: 0x02,
                chip_index: 1
            }
        );
        assert_eq!(commands[8], Commands::EndOfSoundData);
    }

    #[test]
    fn test_dual_chip_serialization_round_trip_all_methods() {
        // Test complete round-trip serialization for all dual chip methods
        let test_commands = vec![
            // Method #1: PSG
            Commands::PSGWrite {
                value: 0x9F,
                chip_index: 0,
            },
            Commands::PSGWrite {
                value: 0x9F,
                chip_index: 1,
            },
            Commands::GameGearPSGStereo {
                value: 0xFF,
                chip_index: 0,
            },
            Commands::GameGearPSGStereo {
                value: 0xFF,
                chip_index: 1,
            },
            // Method #1: YM family (test a few key ones)
            Commands::YM2413Write {
                register: 0x30,
                value: 0x14,
                chip_index: 0,
            },
            Commands::YM2413Write {
                register: 0x30,
                value: 0x14,
                chip_index: 1,
            },
            Commands::YM2612Port0Write {
                register: 0x22,
                value: 0x00,
                chip_index: 0,
            },
            Commands::YM2612Port0Write {
                register: 0x22,
                value: 0x00,
                chip_index: 1,
            },
            // Method #2: Bit 7 checking (test several)
            Commands::AY8910Write {
                register: 0x07,
                value: 0x38,
                chip_index: 0,
            },
            Commands::AY8910Write {
                register: 0x07,
                value: 0x38,
                chip_index: 1,
            },
            Commands::GameBoyDMGWrite {
                register: 0x26,
                value: 0x8F,
                chip_index: 0,
            },
            Commands::GameBoyDMGWrite {
                register: 0x26,
                value: 0x8F,
                chip_index: 1,
            },
            Commands::NESAPUWrite {
                register: 0x15,
                value: 0x0F,
                chip_index: 0,
            },
            Commands::NESAPUWrite {
                register: 0x15,
                value: 0x0F,
                chip_index: 1,
            },
            Commands::HuC6280Write {
                register: 0x02,
                value: 0x44,
                chip_index: 0,
            },
            Commands::HuC6280Write {
                register: 0x02,
                value: 0x44,
                chip_index: 1,
            },
            // DAC Stream dual chip
            Commands::DACStreamSetupControl {
                stream_id: 0x01,
                chip_type: 0x02,
                port: 0x00,
                command: 0x01,
                chip_index: 0,
            },
            Commands::DACStreamSetupControl {
                stream_id: 0x01,
                chip_type: 0x02,
                port: 0x00,
                command: 0x01,
                chip_index: 1,
            },
        ];

        for cmd in test_commands {
            // Serialize command to bytes
            let serialized = cmd.clone().to_bytes().unwrap();

            // Parse bytes back to command
            let mut bytes = Bytes::from(serialized);
            let parsed = Commands::from_bytes(&mut bytes).unwrap();

            // Should be identical
            assert_eq!(cmd, parsed, "Round-trip failed for command: {:?}", cmd);
        }
    }

    #[test]
    fn test_dual_chip_backward_compatibility() {
        // Test that existing single-chip commands still work (backward compatibility)
        let single_chip_commands = vec![
            // Test that chip_index: 0 produces the same output as original commands
            Commands::PSGWrite {
                value: 0x9F,
                chip_index: 0,
            },
            Commands::YM2413Write {
                register: 0x30,
                value: 0x14,
                chip_index: 0,
            },
            Commands::YM2612Port0Write {
                register: 0x22,
                value: 0x00,
                chip_index: 0,
            },
            Commands::AY8910Write {
                register: 0x07,
                value: 0x38,
                chip_index: 0,
            },
            Commands::GameBoyDMGWrite {
                register: 0x26,
                value: 0x8F,
                chip_index: 0,
            },
            Commands::DACStreamSetupControl {
                stream_id: 0x01,
                chip_type: 0x02,
                port: 0x00,
                command: 0x01,
                chip_index: 0,
            },
        ];

        for cmd in single_chip_commands {
            // All commands with chip_index: 0 should serialize without error
            let result = cmd.clone().to_bytes();
            assert!(result.is_ok(), "Single chip command failed: {:?}", cmd);

            // Verify the opcodes are correct for first chip
            let bytes = result.unwrap();
            match cmd {
                Commands::PSGWrite { .. } => assert_eq!(bytes[0], 0x50),
                Commands::YM2413Write { .. } => assert_eq!(bytes[0], 0x51),
                Commands::YM2612Port0Write { .. } => assert_eq!(bytes[0], 0x52),
                Commands::AY8910Write { .. } => {
                    assert_eq!(bytes[0], 0xA0);
                    assert_eq!(bytes[1] & 0x80, 0); // Bit 7 should be 0
                },
                Commands::GameBoyDMGWrite { .. } => {
                    assert_eq!(bytes[0], 0xB3);
                    assert_eq!(bytes[1] & 0x80, 0); // Bit 7 should be 0
                },
                Commands::DACStreamSetupControl { .. } => {
                    assert_eq!(bytes[0], 0x90);
                    assert_eq!(bytes[2] & 0x80, 0); // chip_type bit 7 should be 0
                },
                _ => {},
            }
        }
    }

    #[test]
    fn test_dual_chip_mixed_parsing_methods() {
        // Test that we can mix parsing methods (from_bytes, from_bytes_safe, from_bytes_with_config)
        let mut test_data = BytesMut::new();

        // Add a dual chip command
        test_data.put_u8(0xA1); // YM2413 second chip
        test_data.put_u8(0x30);
        test_data.put_u8(0x14);

        let bytes_copy1 = test_data.clone().freeze();
        let bytes_copy2 = test_data.clone().freeze();
        let bytes_copy3 = test_data.clone().freeze();

        // Test from_bytes
        let mut bytes1 = bytes_copy1;
        let cmd1 = Commands::from_bytes(&mut bytes1).unwrap();

        // Test from_bytes_safe
        let mut bytes2 = bytes_copy2;
        let cmd2 = Commands::from_bytes_safe(&mut bytes2).unwrap();

        // Test from_bytes_with_config
        let mut bytes3 = bytes_copy3;
        let config = crate::ParserConfig::default();
        let mut tracker = crate::ResourceTracker::new();
        let cmd3 = Commands::from_bytes_with_config(&mut bytes3, &config, &mut tracker).unwrap();

        // All should produce the same result
        let expected = Commands::YM2413Write {
            register: 0x30,
            value: 0x14,
            chip_index: 1,
        };
        assert_eq!(cmd1, expected);
        assert_eq!(cmd2, expected);
        assert_eq!(cmd3, expected);
    }

    // Property-based tests using proptest
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_psg_commands_round_trip(
            value in 0u8..=255u8,
            chip_index in 0u8..=1u8
        ) {
            // Test PSG command round-trip parsing
            let original_cmd = Commands::PSGWrite { value, chip_index };
            
            // Serialize to bytes
            let bytes_result = original_cmd.clone().to_bytes();
            prop_assert!(bytes_result.is_ok());
            
            let bytes = bytes_result.unwrap();
            let mut data = Bytes::from(bytes);
            
            // Parse back from bytes
            let parsed_result = Commands::from_bytes(&mut data);
            prop_assert!(parsed_result.is_ok());
            
            let parsed_cmd = parsed_result.unwrap();
            prop_assert_eq!(original_cmd, parsed_cmd);
        }

        #[test]
        fn test_ym2612_commands_round_trip(
            register in 0u8..=255u8,
            value in 0u8..=255u8,
            chip_index in 0u8..=1u8
        ) {
            // Test YM2612 Port 0 command round-trip parsing
            let original_cmd = Commands::YM2612Port0Write { register, value, chip_index };
            
            // Serialize to bytes
            let bytes_result = original_cmd.clone().to_bytes();
            prop_assert!(bytes_result.is_ok());
            
            let bytes = bytes_result.unwrap();
            let mut data = Bytes::from(bytes);
            
            // Parse back from bytes
            let parsed_result = Commands::from_bytes(&mut data);
            prop_assert!(parsed_result.is_ok());
            
            let parsed_cmd = parsed_result.unwrap();
            prop_assert_eq!(original_cmd, parsed_cmd);
        }

        #[test]
        fn test_wait_commands_round_trip(
            n in 1u16..=65535u16
        ) {
            // Test wait commands round-trip parsing
            let original_cmd = Commands::WaitNSamples { n };
            
            // Serialize to bytes
            let bytes_result = original_cmd.clone().to_bytes();
            prop_assert!(bytes_result.is_ok());
            
            let bytes = bytes_result.unwrap();
            let mut data = Bytes::from(bytes);
            
            // Parse back from bytes
            let parsed_result = Commands::from_bytes(&mut data);
            prop_assert!(parsed_result.is_ok());
            
            let parsed_cmd = parsed_result.unwrap();
            prop_assert_eq!(original_cmd, parsed_cmd);
        }

        #[test]
        fn test_dac_stream_commands_round_trip(
            stream_id in 0u8..=255u8,
            chip_type in 0u8..=127u8,
            port in 0u8..=255u8,
            command in 0u8..=255u8,
            chip_index in 0u8..=1u8
        ) {
            // Test DAC Stream Setup Control command round-trip parsing
            let original_cmd = Commands::DACStreamSetupControl {
                stream_id, chip_type, port, command, chip_index
            };
            
            // Serialize to bytes
            let bytes_result = original_cmd.clone().to_bytes();
            prop_assert!(bytes_result.is_ok());
            
            let bytes = bytes_result.unwrap();
            let mut data = Bytes::from(bytes);
            
            // Parse back from bytes
            let parsed_result = Commands::from_bytes(&mut data);
            prop_assert!(parsed_result.is_ok());
            
            let parsed_cmd = parsed_result.unwrap();
            prop_assert_eq!(original_cmd, parsed_cmd);
        }

        #[test]
        fn test_dual_chip_method2_commands(
            register in 0u8..=127u8,
            value in 0u8..=255u8,
            chip_index in 0u8..=1u8
        ) {
            // Test Method #2 dual chip support (bit 7 of register)
            let commands_to_test = vec![
                Commands::AY8910Write { register, value, chip_index },
                Commands::GameBoyDMGWrite { register, value, chip_index },
                Commands::NESAPUWrite { register, value, chip_index },
                Commands::MultiPCMWrite { register, value, chip_index },
            ];
            
            for original_cmd in commands_to_test {
                // Serialize to bytes
                let bytes_result = original_cmd.clone().to_bytes();
                prop_assert!(bytes_result.is_ok());
                
                let bytes = bytes_result.unwrap();
                let mut data = Bytes::from(bytes);
                
                // Parse back from bytes
                let parsed_result = Commands::from_bytes(&mut data);
                prop_assert!(parsed_result.is_ok());
                
                let parsed_cmd = parsed_result.unwrap();
                prop_assert_eq!(original_cmd, parsed_cmd);
            }
        }

        #[test]
        fn test_wait_n_samples_plus_1_commands(
            n in 0u8..=15u8
        ) {
            // Test WaitNSamplesPlus1 commands (0x70-0x7F range)
            let original_cmd = Commands::WaitNSamplesPlus1 { n };
            
            // Serialize to bytes
            let bytes_result = original_cmd.clone().to_bytes();
            prop_assert!(bytes_result.is_ok());
            
            let bytes = bytes_result.unwrap();
            let mut data = Bytes::from(bytes);
            
            // Parse back from bytes
            let parsed_result = Commands::from_bytes(&mut data);
            prop_assert!(parsed_result.is_ok());
            
            let parsed_cmd = parsed_result.unwrap();
            prop_assert_eq!(original_cmd, parsed_cmd);
        }

        #[test]
        fn test_ym2612_address_2a_wait_commands(
            n in 0u8..=15u8
        ) {
            // Test YM2612Port0Address2AWriteWait commands (0x80-0x8F range)
            let original_cmd = Commands::YM2612Port0Address2AWriteWait { n };
            
            // Serialize to bytes
            let bytes_result = original_cmd.clone().to_bytes();
            prop_assert!(bytes_result.is_ok());
            
            let bytes = bytes_result.unwrap();
            let mut data = Bytes::from(bytes);
            
            // Parse back from bytes
            let parsed_result = Commands::from_bytes(&mut data);
            prop_assert!(parsed_result.is_ok());
            
            let parsed_cmd = parsed_result.unwrap();
            prop_assert_eq!(original_cmd, parsed_cmd);
        }

        #[test]
        fn test_ymaha_fm_chip_commands_round_trip(
            register in 0u8..=255u8,
            value in 0u8..=255u8,
            chip_index in 0u8..=1u8
        ) {
            // Test various Yamaha FM chip commands
            let commands_to_test = vec![
                Commands::YM2413Write { register, value, chip_index },
                Commands::YM2151Write { register, value, chip_index },
                Commands::YM2203Write { register, value, chip_index },
                Commands::YM2608Port0Write { register, value, chip_index },
                Commands::YM2608Port1Write { register, value, chip_index },
                Commands::YM2610Port0Write { register, value, chip_index },
                Commands::YM2610Port1Write { register, value, chip_index },
                Commands::YM3812Write { register, value, chip_index },
                Commands::YM3526Write { register, value, chip_index },
                Commands::Y8950Write { register, value, chip_index },
                Commands::YMZ280BWrite { register, value, chip_index },
                Commands::YMF262Port0Write { register, value, chip_index },
                Commands::YMF262Port1Write { register, value, chip_index },
            ];
            
            for original_cmd in commands_to_test {
                // Serialize to bytes
                let bytes_result = original_cmd.clone().to_bytes();
                prop_assert!(bytes_result.is_ok());
                
                let bytes = bytes_result.unwrap();
                let mut data = Bytes::from(bytes);
                
                // Parse back from bytes
                let parsed_result = Commands::from_bytes(&mut data);
                prop_assert!(parsed_result.is_ok());
                
                let parsed_cmd = parsed_result.unwrap();
                prop_assert_eq!(original_cmd, parsed_cmd);
            }
        }

        #[test]
        fn test_frequency_and_offset_commands(
            frequency in 1u32..=0xFFFFFFu32,
            offset in 0u16..=65535u16,
            value in 0u8..=255u8
        ) {
            // Test commands with frequency and offset parameters
            let stream_id = 0u8;
            
            let commands_to_test = vec![
                Commands::DACStreamSetFrequency { stream_id, frequency },
                Commands::SegaPCMWrite { offset, value },
                Commands::SCSPWrite { offset, value },
                Commands::VSUWrite { offset, value },
                Commands::X1010Write { offset, value },
            ];
            
            for original_cmd in commands_to_test {
                // Serialize to bytes
                let bytes_result = original_cmd.clone().to_bytes();
                prop_assert!(bytes_result.is_ok());
                
                let bytes = bytes_result.unwrap();
                let mut data = Bytes::from(bytes);
                
                // Parse back from bytes
                let parsed_result = Commands::from_bytes(&mut data);
                prop_assert!(parsed_result.is_ok());
                
                let parsed_cmd = parsed_result.unwrap();
                prop_assert_eq!(original_cmd, parsed_cmd);
            }
        }

        #[test]
        fn test_special_commands_invariants(
            ay8910_value in 0u8..=255u8
        ) {
            // Test special command invariants
            
            // Fixed wait commands
            let fixed_commands = vec![
                Commands::Wait735Samples,
                Commands::Wait882Samples,
                Commands::EndOfSoundData,
            ];
            
            for original_cmd in fixed_commands {
                let bytes_result = original_cmd.clone().to_bytes();
                prop_assert!(bytes_result.is_ok());
                
                let bytes = bytes_result.unwrap();
                let mut data = Bytes::from(bytes);
                
                let parsed_result = Commands::from_bytes(&mut data);
                prop_assert!(parsed_result.is_ok());
                
                let parsed_cmd = parsed_result.unwrap();
                prop_assert_eq!(original_cmd, parsed_cmd);
            }
            
            // AY8910 stereo mask command
            let ay_cmd = Commands::AY8910StereoMask { value: ay8910_value };
            let bytes_result = ay_cmd.clone().to_bytes();
            prop_assert!(bytes_result.is_ok());
            
            let bytes = bytes_result.unwrap();
            let mut data = Bytes::from(bytes);
            
            let parsed_result = Commands::from_bytes(&mut data);
            prop_assert!(parsed_result.is_ok());
            
            let parsed_cmd = parsed_result.unwrap();
            prop_assert_eq!(ay_cmd, parsed_cmd);
        }
    }

    // ========== SERIALIZATION TESTS ==========
    // Tests to improve coverage of serialization.rs

    #[test]
    fn test_datablock_uncompressed_stream_serialization() {
        let cmd = Commands::DataBlock {
            block_type: 0x00,
            data: DataBlockContent::UncompressedStream {
                chip_type: StreamChipType::YM2612,
                data: vec![0x01, 0x02, 0x03, 0x04],
            },
        };
        
        let result = cmd.to_bytes().unwrap();
        let expected = vec![
            0x67, 0x66, 0x00,  // DataBlock header
            0x04, 0x00, 0x00, 0x00,  // data size (4 bytes)
            0x01, 0x02, 0x03, 0x04,  // data
        ];
        assert_eq!(result, expected);
    }

    #[test] 
    fn test_datablock_compressed_stream_bitpacking_serialization() {
        let cmd = Commands::DataBlock {
            block_type: 0x40,
            data: DataBlockContent::CompressedStream {
                chip_type: StreamChipType::RF5C68,
                compression: CompressionType::BitPacking {
                    bits_decompressed: 16,
                    bits_compressed: 8,
                    sub_type: 1,
                    add_value: 100,
                },
                uncompressed_size: 1000,
                data: vec![0xAA, 0xBB],
            },
        };
        
        let result = cmd.to_bytes().unwrap();
        assert_eq!(result[0..3], [0x67, 0x66, 0x40]); // DataBlock header
        assert_eq!(result[3..7], [0x0B, 0x00, 0x00, 0x00]); // size: 2 data + 9 header = 11
        assert_eq!(result[7], 0x00); // BitPacking compression type
        assert_eq!(result[8..12], [0xE8, 0x03, 0x00, 0x00]); // uncompressed_size: 1000
        assert_eq!(result[12], 16); // bits_decompressed
        assert_eq!(result[13], 8);  // bits_compressed
        assert_eq!(result[14], 1);  // sub_type
        assert_eq!(result[15..17], [0x64, 0x00]); // add_value: 100
        assert_eq!(result[17..19], [0xAA, 0xBB]); // data
    }

    #[test]
    fn test_datablock_compressed_stream_dpcm_serialization() {
        let cmd = Commands::DataBlock {
            block_type: 0x41,
            data: DataBlockContent::CompressedStream {
                chip_type: StreamChipType::RF5C164,
                compression: CompressionType::DPCM {
                    bits_decompressed: 8,
                    bits_compressed: 4,
                    start_value: 256,
                },
                uncompressed_size: 2000,
                data: vec![0x11, 0x22, 0x33],
            },
        };
        
        let result = cmd.to_bytes().unwrap();
        assert_eq!(result[0..3], [0x67, 0x66, 0x41]); // DataBlock header
        assert_eq!(result[3..7], [0x0C, 0x00, 0x00, 0x00]); // size: 3 data + 9 header = 12
        assert_eq!(result[7], 0x01); // DPCM compression type
        assert_eq!(result[8..12], [0xD0, 0x07, 0x00, 0x00]); // uncompressed_size: 2000
        assert_eq!(result[12], 8); // bits_decompressed
        assert_eq!(result[13], 4); // bits_compressed
        assert_eq!(result[14], 0x00); // reserved byte
        assert_eq!(result[15..17], [0x00, 0x01]); // start_value: 256
        assert_eq!(result[17..20], [0x11, 0x22, 0x33]); // data
    }

    #[test]
    fn test_datablock_decompression_table_serialization() {
        let cmd = Commands::DataBlock {
            block_type: 0x7F,
            data: DataBlockContent::DecompressionTable {
                compression_type: 0x00,
                sub_type: 0x01,
                bits_decompressed: 16,
                bits_compressed: 8,
                value_count: 256,
                table_data: vec![0x01, 0x02, 0x03, 0x04],
            },
        };
        
        let result = cmd.to_bytes().unwrap();
        assert_eq!(result[0..3], [0x67, 0x66, 0x7F]); // DataBlock header
        assert_eq!(result[3..7], [0x0A, 0x00, 0x00, 0x00]); // size: 4 data + 6 header = 10
        assert_eq!(result[7], 0x00); // compression_type
        assert_eq!(result[8], 0x01); // sub_type
        assert_eq!(result[9], 16); // bits_decompressed
        assert_eq!(result[10], 8); // bits_compressed
        assert_eq!(result[11..13], [0x00, 0x01]); // value_count: 256
        assert_eq!(result[13..17], [0x01, 0x02, 0x03, 0x04]); // table_data
    }

    #[test]
    fn test_datablock_rom_dump_serialization() {
        let cmd = Commands::DataBlock {
            block_type: 0x80,
            data: DataBlockContent::ROMDump {
                chip_type: crate::vgm_commands::data_blocks::ROMDumpChipType::SegaPCM,
                total_size: 0x10000,
                start_address: 0x8000,
                data: vec![0xDE, 0xAD, 0xBE, 0xEF],
            },
        };
        
        let result = cmd.to_bytes().unwrap();
        assert_eq!(result[0..3], [0x67, 0x66, 0x80]); // DataBlock header
        assert_eq!(result[3..7], [0x0C, 0x00, 0x00, 0x00]); // size: 4 data + 8 header = 12
        assert_eq!(result[7..11], [0x00, 0x00, 0x01, 0x00]); // total_size: 0x10000
        assert_eq!(result[11..15], [0x00, 0x80, 0x00, 0x00]); // start_address: 0x8000
        assert_eq!(result[15..19], [0xDE, 0xAD, 0xBE, 0xEF]); // data
    }

    #[test]
    fn test_datablock_ram_write_small_serialization() {
        let cmd = Commands::DataBlock {
            block_type: 0xC0,
            data: DataBlockContent::RAMWriteSmall {
                chip_type: crate::vgm_commands::data_blocks::RAMWriteChipType::RF5C68,
                start_address: 0x1000,
                data: vec![0xCA, 0xFE],
            },
        };
        
        let result = cmd.to_bytes().unwrap();
        assert_eq!(result[0..3], [0x67, 0x66, 0xC0]); // DataBlock header
        assert_eq!(result[3..7], [0x04, 0x00, 0x00, 0x00]); // size: 2 data + 2 header = 4
        assert_eq!(result[7..9], [0x00, 0x10]); // start_address: 0x1000
        assert_eq!(result[9..11], [0xCA, 0xFE]); // data
    }

    #[test]
    fn test_datablock_ram_write_large_serialization() {
        let cmd = Commands::DataBlock {
            block_type: 0xE0,
            data: DataBlockContent::RAMWriteLarge {
                chip_type: crate::vgm_commands::data_blocks::RAMWriteChipType::SCSP,
                start_address: 0x20000,
                data: vec![0x12, 0x34, 0x56],
            },
        };
        
        let result = cmd.to_bytes().unwrap();
        assert_eq!(result[0..3], [0x67, 0x66, 0xE0]); // DataBlock header
        assert_eq!(result[3..7], [0x07, 0x00, 0x00, 0x00]); // size: 3 data + 4 header = 7
        assert_eq!(result[7..11], [0x00, 0x00, 0x02, 0x00]); // start_address: 0x20000
        assert_eq!(result[11..14], [0x12, 0x34, 0x56]); // data
    }

    #[test]
    fn test_datablock_unknown_serialization() {
        let cmd = Commands::DataBlock {
            block_type: 0xFF,
            data: DataBlockContent::Unknown {
                data: vec![0x01, 0x02, 0x03],
            },
        };
        
        let result = cmd.to_bytes().unwrap();
        assert_eq!(result[0..3], [0x67, 0x66, 0xFF]); // DataBlock header
        assert_eq!(result[3..7], [0x03, 0x00, 0x00, 0x00]); // size: 3
        assert_eq!(result[7..10], [0x01, 0x02, 0x03]); // data
    }

    #[test]
    fn test_pcm_ram_write_unsupported_error() {
        let cmd = Commands::PCMRAMWrite {
            chip_type: 0x02,
            read_offset: 0x1000,
            write_offset: 0x2000,
            size: 0x100,
            data: vec![0xAA; 0x100],
        };
        
        let result = cmd.to_bytes();
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::errors::VgmError::FeatureNotSupported { feature, .. } => {
                assert!(feature.contains("PCM RAM Write command serialization"));
            },
            _ => panic!("Expected FeatureNotSupported error"),
        }
    }

    #[test]
    fn test_chip_command_serializations() {
        // Test RF5C68Write
        let cmd = Commands::RF5C68Write { register: 0x10, value: 0x20 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB0, 0x10, 0x20]);

        // Test RF5C164Write  
        let cmd = Commands::RF5C164Write { register: 0x11, value: 0x21 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB1, 0x11, 0x21]);

        // Test PWMWrite
        let cmd = Commands::PWMWrite { register: 0x12, value: 0x3344 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB2, 0x12, 0x44, 0x33]);

        // Test GameBoyDMGWrite - chip 0
        let cmd = Commands::GameBoyDMGWrite { register: 0x10, value: 0x20, chip_index: 0 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB3, 0x10, 0x20]);

        // Test GameBoyDMGWrite - chip 1 (sets bit 7)
        let cmd = Commands::GameBoyDMGWrite { register: 0x10, value: 0x20, chip_index: 1 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB3, 0x90, 0x20]);

        // Test NESAPUWrite - chip 0
        let cmd = Commands::NESAPUWrite { register: 0x15, value: 0x25, chip_index: 0 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB4, 0x15, 0x25]);

        // Test NESAPUWrite - chip 1 (sets bit 7)
        let cmd = Commands::NESAPUWrite { register: 0x15, value: 0x25, chip_index: 1 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB4, 0x95, 0x25]);

        // Test MultiPCMWrite - chip 0
        let cmd = Commands::MultiPCMWrite { register: 0x16, value: 0x26, chip_index: 0 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB5, 0x16, 0x26]);

        // Test MultiPCMWrite - chip 1 (sets bit 7)
        let cmd = Commands::MultiPCMWrite { register: 0x16, value: 0x26, chip_index: 1 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB5, 0x96, 0x26]);

        // Test uPD7759Write - chip 0
        let cmd = Commands::uPD7759Write { register: 0x17, value: 0x27, chip_index: 0 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB6, 0x17, 0x27]);

        // Test uPD7759Write - chip 1 (sets bit 7)
        let cmd = Commands::uPD7759Write { register: 0x17, value: 0x27, chip_index: 1 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB6, 0x97, 0x27]);

        // Test OKIM6258Write - chip 0
        let cmd = Commands::OKIM6258Write { register: 0x18, value: 0x28, chip_index: 0 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB7, 0x18, 0x28]);

        // Test OKIM6258Write - chip 1 (sets bit 7)
        let cmd = Commands::OKIM6258Write { register: 0x18, value: 0x28, chip_index: 1 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB7, 0x98, 0x28]);

        // Test OKIM6295Write - chip 0
        let cmd = Commands::OKIM6295Write { register: 0x19, value: 0x29, chip_index: 0 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB8, 0x19, 0x29]);

        // Test OKIM6295Write - chip 1 (sets bit 7)
        let cmd = Commands::OKIM6295Write { register: 0x19, value: 0x29, chip_index: 1 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB8, 0x99, 0x29]);

        // Test HuC6280Write - chip 0
        let cmd = Commands::HuC6280Write { register: 0x1A, value: 0x2A, chip_index: 0 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB9, 0x1A, 0x2A]);

        // Test HuC6280Write - chip 1 (sets bit 7)
        let cmd = Commands::HuC6280Write { register: 0x1A, value: 0x2A, chip_index: 1 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xB9, 0x9A, 0x2A]);

        // Test K053260Write - chip 0
        let cmd = Commands::K053260Write { register: 0x1B, value: 0x2B, chip_index: 0 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xBA, 0x1B, 0x2B]);

        // Test K053260Write - chip 1 (sets bit 7)
        let cmd = Commands::K053260Write { register: 0x1B, value: 0x2B, chip_index: 1 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xBA, 0x9B, 0x2B]);

        // Test PokeyWrite - chip 0
        let cmd = Commands::PokeyWrite { register: 0x1C, value: 0x2C, chip_index: 0 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xBB, 0x1C, 0x2C]);

        // Test PokeyWrite - chip 1 (sets bit 7)
        let cmd = Commands::PokeyWrite { register: 0x1C, value: 0x2C, chip_index: 1 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xBB, 0x9C, 0x2C]);

        // Test WonderSwanWrite - chip 0
        let cmd = Commands::WonderSwanWrite { register: 0x1D, value: 0x2D, chip_index: 0 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xBC, 0x1D, 0x2D]);

        // Test WonderSwanWrite - chip 1 (sets bit 7)
        let cmd = Commands::WonderSwanWrite { register: 0x1D, value: 0x2D, chip_index: 1 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xBC, 0x9D, 0x2D]);

        // Test SAA1099Write - chip 0
        let cmd = Commands::SAA1099Write { register: 0x1E, value: 0x2E, chip_index: 0 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xBD, 0x1E, 0x2E]);

        // Test SAA1099Write - chip 1 (sets bit 7)
        let cmd = Commands::SAA1099Write { register: 0x1E, value: 0x2E, chip_index: 1 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xBD, 0x9E, 0x2E]);

        // Test ES5506Write - chip 0
        let cmd = Commands::ES5506Write { register: 0x1F, value: 0x2F, chip_index: 0 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xBE, 0x1F, 0x2F]);

        // Test ES5506Write - chip 1 (sets bit 7)
        let cmd = Commands::ES5506Write { register: 0x1F, value: 0x2F, chip_index: 1 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xBE, 0x9F, 0x2F]);

        // Test GA20Write - chip 0
        let cmd = Commands::GA20Write { register: 0x20, value: 0x30, chip_index: 0 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xBF, 0x20, 0x30]);

        // Test GA20Write - chip 1 (sets bit 7)
        let cmd = Commands::GA20Write { register: 0x20, value: 0x30, chip_index: 1 };
        assert_eq!(cmd.to_bytes().unwrap(), vec![0xBF, 0xA0, 0x30]);
    }

    // ========== PARSING ERROR PATH TESTS ==========
    // Tests to improve coverage of parsing.rs error handling

    #[test]
    fn test_datablock_invalid_compatibility_byte() {
        // Test DataBlock command with invalid compatibility byte
        let mut data = Bytes::from(vec![
            0x67, 0x65, // Invalid compatibility byte (should be 0x66)
            0x00, // block_type
            0x04, 0x00, 0x00, 0x00, // data_size
            0x01, 0x02, 0x03, 0x04, // data
        ]);
        
        let result = Commands::from_bytes(&mut data);
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::InvalidCommandParameters { opcode, reason, .. } => {
                assert_eq!(opcode, 0x67);
                assert!(reason.contains("Expected compatibility byte 0x66"));
            },
            _ => panic!("Expected InvalidCommandParameters error"),
        }
    }

    #[test]
    fn test_pcm_ram_write_invalid_compatibility_byte() {
        // Test PCM RAM Write command with invalid compatibility byte
        let mut data = Bytes::from(vec![
            0x68, 0x65, // Invalid compatibility byte (should be 0x66)
            0x02, // chip_type
            0x00, 0x10, 0x00, // read_offset (24-bit LE)
            0x00, 0x20, 0x00, // write_offset (24-bit LE)
            0x04, 0x00, 0x00, // size (24-bit LE)
            0xAA, 0xBB, 0xCC, 0xDD, // data
        ]);
        
        let result = Commands::from_bytes(&mut data);
        assert!(result.is_err());
        match result.unwrap_err() {
            VgmError::InvalidCommandParameters { opcode, reason, .. } => {
                assert_eq!(opcode, 0x68);
                assert!(reason.contains("Expected compatibility byte 0x66"));
            },
            _ => panic!("Expected InvalidCommandParameters error"),
        }
    }

    #[test]
    fn test_datablock_buffer_underflow() {
        // Test DataBlock command with insufficient data
        let mut data = Bytes::from(vec![
            0x67, 0x66, 0x00, // DataBlock header
            0x10, 0x00, 0x00, 0x00, // Claim 16 bytes of data
            0x01, 0x02, // Only provide 2 bytes
        ]);
        
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        let result = Commands::from_bytes_with_config(&mut data, &config, &mut tracker);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::BufferUnderflow { .. }));
    }

    #[test]
    fn test_pcm_ram_write_buffer_underflow() {
        // Test PCM RAM Write command with insufficient data
        let mut data = Bytes::from(vec![
            0x68, 0x66, // PCM RAM Write header
            0x02, // chip_type
            0x00, 0x10, 0x00, // read_offset
            0x00, 0x20, 0x00, // write_offset
            0x10, 0x00, 0x00, // Claim 16 bytes
            0xAA, 0xBB, // Only provide 2 bytes
        ]);
        
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        let result = Commands::from_bytes_with_config(&mut data, &config, &mut tracker);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::BufferUnderflow { .. }));
    }

    #[test]
    fn test_pcm_ram_write_zero_size_special_case() {
        // Test PCM RAM Write command with size=0 (should become 0x01000000)
        let large_data = vec![0xAA; 1000]; // Can't test full 16MB in unit test
        let mut data = Bytes::from([
            vec![
                0x68, 0x66, // PCM RAM Write header
                0x02, // chip_type
                0x00, 0x10, 0x00, // read_offset
                0x00, 0x20, 0x00, // write_offset
                0x00, 0x00, 0x00, // size = 0 (special case)
            ],
            large_data,
        ].concat());
        
        // Use a config that allows large data blocks for this test
        let config = ParserConfig {
            max_data_block_size: 2000,
            ..Default::default()
        };
        let mut tracker = ResourceTracker::new();
        
        // This should fail due to insufficient data, but should recognize the special case
        let result = Commands::from_bytes_with_config(&mut data, &config, &mut tracker);
        assert!(result.is_err());
        // Should be a BufferUnderflow because we don't have 16MB of data
        assert!(matches!(result.unwrap_err(), VgmError::BufferUnderflow { .. }));
    }

    #[test]
    fn test_datablock_size_limit_exceeded() {
        // Test DataBlock command exceeding size limits
        let mut data = Bytes::from(vec![
            0x67, 0x66, 0x00, // DataBlock header
            0xFF, 0xFF, 0xFF, 0x7F, // Very large size (2GB-1)
        ]);
        
        let config = ParserConfig {
            max_data_block_size: 1000, // Small limit
            ..Default::default()
        };
        let mut tracker = ResourceTracker::new();
        let result = Commands::from_bytes_with_config(&mut data, &config, &mut tracker);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::DataSizeExceedsLimit { .. }));
    }

    #[test]
    fn test_pcm_ram_write_size_limit_exceeded() {
        // Test PCM RAM Write command exceeding size limits
        // Using a smaller size (2000 bytes) with enough data provided
        let data_payload = vec![0xAA; 2000]; // Provide exactly 2000 bytes of data
        let mut data = Bytes::from([
            vec![
                0x68, 0x66, // PCM RAM Write header
                0x02, // chip_type
                0x00, 0x10, 0x00, // read_offset
                0x00, 0x20, 0x00, // write_offset
                0xD0, 0x07, 0x00, // Size: 2000 bytes (0x07D0)
            ],
            data_payload,
        ].concat());
        
        let config = ParserConfig {
            max_data_block_size: 1000, // Limit smaller than requested size
            ..Default::default()
        };
        let mut tracker = ResourceTracker::new();
        let result = Commands::from_bytes_with_config(&mut data, &config, &mut tracker);
        
        assert!(result.is_err());
        // Should get DataSizeExceedsLimit because we have enough data for buffer check
        // but the requested size (2000) exceeds the config limit (1000)
        assert!(matches!(result.unwrap_err(), VgmError::DataSizeExceedsLimit { .. }));
    }

    #[test]
    fn test_command_count_limit_exceeded() {
        // Test exceeding command count limit in parse_commands_with_config
        let mut data = Bytes::from(vec![
            0x62, // Wait735Samples
            0x62, // Wait735Samples
            0x66, // EndOfSoundData
        ]);
        
        let config = ParserConfig {
            max_commands: 1, // Very small limit
            ..Default::default()
        };
        let mut tracker = ResourceTracker::new();
        
        let result = crate::vgm_commands::parse_commands_with_config(&mut data, &config, &mut tracker);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_commands_safe_mode() {
        // Test safe parsing mode with mixed valid and invalid commands
        let mut data = Bytes::from(vec![
            0x62, // Valid: Wait735Samples
            0xFF, // Invalid command (should cause parsing to stop)
            0x63, // This shouldn't be parsed due to error above
        ]);
        
        let commands = crate::vgm_commands::parse_commands_safe(&mut data);
        assert_eq!(commands.len(), 1);
        assert!(matches!(commands[0], Commands::Wait735Samples));
    }

    #[test]
    fn test_parse_commands_backward_compatibility() {
        // Test backward compatibility parsing that returns empty vec on error
        let mut data = Bytes::from(vec![
            0xFF, // Invalid command
        ]);
        
        let commands = crate::vgm_commands::parse_commands(&mut data);
        assert!(commands.is_empty());
    }

    #[test]
    fn test_pcm_ram_write_24bit_values() {
        // Test PCM RAM Write with proper 24-bit value parsing
        let mut data = Bytes::from(vec![
            0x68, 0x66, // PCM RAM Write header
            0x02, // chip_type
            0x34, 0x12, 0x56, // read_offset = 0x561234 (24-bit LE)
            0x78, 0x9A, 0xBC, // write_offset = 0xBC9A78 (24-bit LE)
            0x04, 0x00, 0x00, // size = 4
            0xAA, 0xBB, 0xCC, 0xDD, // data
        ]);
        
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        let result = Commands::from_bytes_with_config(&mut data, &config, &mut tracker).unwrap();
        
        match result {
            Commands::PCMRAMWrite { chip_type, read_offset, write_offset, size, data } => {
                assert_eq!(chip_type, 0x02);
                assert_eq!(read_offset, 0x561234);
                assert_eq!(write_offset, 0xBC9A78);
                assert_eq!(size, 4);
                assert_eq!(data, vec![0xAA, 0xBB, 0xCC, 0xDD]);
            },
            _ => panic!("Expected PCMRAMWrite command"),
        }
    }

    #[test]
    fn test_unknown_command_fallback_logic() {
        // Test fallback logic for unknown commands that should route to standard parsing
        let mut data = Bytes::from(vec![
            0x70, // WaitNSamplesPlus1 with n=0 (should be parsed by standard logic)
        ]);
        
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        let result = Commands::from_bytes_with_config(&mut data, &config, &mut tracker).unwrap();
        
        assert!(matches!(result, Commands::WaitNSamplesPlus1 { n: 0 }));
    }

    #[test]
    fn test_datablock_with_all_content_types() {
        // Test DataBlock parsing with different content types using from_bytes_with_config
        let test_cases = vec![
            // UncompressedStream
            (0x00, vec![0x01, 0x02, 0x03, 0x04]),
            // CompressedStream (will be parsed by DataBlockContent)
            (0x40, vec![
                0x00, // BitPacking compression
                0xE8, 0x03, 0x00, 0x00, // uncompressed_size = 1000
                16, 8, 1, // bits_decompressed, bits_compressed, sub_type
                0x64, 0x00, // add_value = 100
                0xAA, 0xBB, // compressed data
            ]),
            // DecompressionTable
            (0x7F, vec![
                0x00, 0x01, 16, 8, // compression_type, sub_type, bits_decompressed, bits_compressed
                0x0A, 0x00, // value_count = 10
                0x01, 0x02, 0x03, 0x04, // table_data
            ]),
            // ROMDump
            (0x80, vec![
                0x00, 0x10, 0x00, 0x00, // total_size = 0x1000
                0x00, 0x80, 0x00, 0x00, // start_address = 0x8000
                0xDE, 0xAD, 0xBE, 0xEF, // data
            ]),
            // RAMWriteSmall
            (0xC0, vec![
                0x00, 0x10, // start_address = 0x1000
                0xCA, 0xFE, 0xBA, 0xBE, // data
            ]),
            // RAMWriteLarge
            (0xE0, vec![
                0x00, 0x00, 0x02, 0x00, // start_address = 0x20000
                0x12, 0x34, 0x56, 0x78, // data
            ]),
        ];
        
        for (block_type, data_content) in test_cases {
            let data_size = data_content.len() as u32;
            let mut command_data = vec![
                0x67, 0x66, block_type, // DataBlock header
            ];
            command_data.extend_from_slice(&data_size.to_le_bytes());
            command_data.extend_from_slice(&data_content);
            
            let mut data = Bytes::from(command_data);
            let config = ParserConfig::default();
            let mut tracker = ResourceTracker::new();
            let result = Commands::from_bytes_with_config(&mut data, &config, &mut tracker);
            
            assert!(result.is_ok(), "Failed to parse DataBlock with block_type 0x{:02X}", block_type);
            match result.unwrap() {
                Commands::DataBlock { block_type: parsed_type, .. } => {
                    assert_eq!(parsed_type, block_type);
                },
                _ => panic!("Expected DataBlock command for block_type 0x{:02X}", block_type),
            }
        }
    }

    #[test]
    fn test_resource_tracker_integration() {
        // Test resource tracking integration in parsing
        let mut data = Bytes::from(vec![
            0x67, 0x66, 0x00, // DataBlock header
            0x08, 0x00, 0x00, 0x00, // data_size = 8
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // data
            0x62, // Wait735Samples
            0x66, // EndOfSoundData
        ]);
        
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        
        let commands = crate::vgm_commands::parse_commands_with_config(&mut data, &config, &mut tracker).unwrap();
        assert_eq!(commands.len(), 3);
        
        // Verify we tracked the resources correctly
        assert!(matches!(commands[0], Commands::DataBlock { .. }));
        assert!(matches!(commands[1], Commands::Wait735Samples));
        assert!(matches!(commands[2], Commands::EndOfSoundData));
    }

    #[test]
    fn test_write_commands_function() {
        // Test the write_commands function
        let commands = vec![
            Commands::PSGWrite { value: 0x9F, chip_index: 0 },
            Commands::Wait735Samples,
            Commands::EndOfSoundData,
        ];
        
        let mut buffer = BytesMut::new();
        let result = crate::vgm_commands::write_commands(&mut buffer, &commands);
        
        assert!(result.is_ok());
        let expected = vec![
            0x50, 0x9F, // PSGWrite
            0x62,       // Wait735Samples  
            0x66,       // EndOfSoundData
        ];
        assert_eq!(buffer.to_vec(), expected);
    }

    #[test]
    fn test_write_commands_serialization_error() {
        // Test write_commands with a command that fails serialization
        let commands = vec![
            Commands::PCMRAMWrite {
                chip_type: 0x02,
                read_offset: 0x1000,
                write_offset: 0x2000,
                size: 0x100,
                data: vec![0xAA; 0x100],
            },
        ];
        
        let mut buffer = BytesMut::new();
        let result = crate::vgm_commands::write_commands(&mut buffer, &commands);
        
        // PCMRAMWrite serialization should fail with FeatureNotSupported
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VgmError::FeatureNotSupported { .. }));
    }
}
