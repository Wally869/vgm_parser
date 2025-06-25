#[cfg(test)]
mod tests {
    use crate::vgm_commands::compression::{decompress_bit_packing, decompress_dpcm, BitReader};
    use crate::{Commands, CompressionType, DataBlockContent, StreamChipType};
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
        let data_size = 15; // 9 bytes header + 6 bytes data

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
}
