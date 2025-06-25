//! Data Compression Module
//!
//! Handles VGM data block compression and decompression algorithms including
//! bit-packing and DPCM (Differential PCM) methods.

use crate::errors::{VgmError, VgmResult};

/// Decompress bit-packed data according to VGM specification
pub fn decompress_bit_packing(
    compressed_data: &[u8],
    bits_compressed: u8,
    bits_decompressed: u8,
    sub_type: u8,
    add_value: u16,
    uncompressed_size: u32,
    decompression_table: Option<&[u8]>,
) -> VgmResult<Vec<u8>> {
    let mut result = Vec::with_capacity(uncompressed_size as usize);
    let mut bit_reader = BitReader::new(compressed_data);

    // Calculate bytes per decompressed value
    let bytes_per_value = (bits_decompressed as usize).div_ceil(8);

    while result.len() < uncompressed_size as usize {
        // Read compressed bits
        let compressed_value = bit_reader.read_bits(bits_compressed)?;

        // Apply decompression based on sub-type
        let decompressed_value = match sub_type {
            0x00 => {
                // Copy: high bits aren't used
                compressed_value as u32
            },
            0x01 => {
                // Shift left: low bits aren't used
                (compressed_value as u32) << (bits_decompressed - bits_compressed)
            },
            0x02 => {
                // Use table
                let table = decompression_table.ok_or_else(|| VgmError::InvalidDataFormat {
                    field: "decompression_table".to_string(),
                    details: "Bit packing sub-type 0x02 requires a decompression table".to_string(),
                })?;

                let index = compressed_value as usize * bytes_per_value;
                if index + bytes_per_value > table.len() {
                    return Err(VgmError::InvalidDataFormat {
                        field: "table_index".to_string(),
                        details: format!("Table index {} out of bounds", index),
                    });
                }

                // Read value from table based on bytes_per_value
                let mut table_value = 0u32;
                for i in 0..bytes_per_value {
                    table_value |= (table[index + i] as u32) << (i * 8);
                }
                table_value
            },
            _ => {
                return Err(VgmError::InvalidDataFormat {
                    field: "bit_packing_sub_type".to_string(),
                    details: format!("Unknown bit packing sub-type: 0x{:02X}", sub_type),
                });
            },
        };

        // Add the constant value (except for table lookup)
        let final_value = if sub_type != 0x02 {
            decompressed_value.wrapping_add(add_value as u32)
        } else {
            decompressed_value
        };

        // Write the decompressed value in little-endian format
        for i in 0..bytes_per_value.min(4) {
            if result.len() < uncompressed_size as usize {
                result.push((final_value >> (i * 8)) as u8);
            }
        }
    }

    // Ensure we have exactly the expected size
    result.truncate(uncompressed_size as usize);
    Ok(result)
}

/// Decompress DPCM data according to VGM specification
pub fn decompress_dpcm(
    compressed_data: &[u8],
    bits_compressed: u8,
    bits_decompressed: u8,
    start_value: u16,
    uncompressed_size: u32,
    decompression_table: &[u8],
) -> VgmResult<Vec<u8>> {
    let mut result = Vec::with_capacity(uncompressed_size as usize);
    let mut bit_reader = BitReader::new(compressed_data);
    let mut state = start_value as i32;

    // Calculate bytes per decompressed value
    let bytes_per_value = (bits_decompressed as usize).div_ceil(8);

    while result.len() < uncompressed_size as usize {
        // Read compressed bits as index
        let index = bit_reader.read_bits(bits_compressed)? as usize;

        // Look up delta value from table
        let table_index = index * bytes_per_value;
        if table_index + bytes_per_value > decompression_table.len() {
            return Err(VgmError::InvalidDataFormat {
                field: "dpcm_table_index".to_string(),
                details: format!("DPCM table index {} out of bounds", table_index),
            });
        }

        // Read delta value from table (signed)
        let mut delta = 0i32;
        for i in 0..bytes_per_value.min(4) {
            delta |= (decompression_table[table_index + i] as i32) << (i * 8);
        }

        // Sign extend if necessary
        if bytes_per_value < 4 && (delta & (1 << (bytes_per_value * 8 - 1))) != 0 {
            delta |= !0 << (bytes_per_value * 8);
        }

        // Update state with delta
        state = state.wrapping_add(delta);

        // Write the result value in little-endian format
        for i in 0..bytes_per_value.min(4) {
            if result.len() < uncompressed_size as usize {
                result.push((state >> (i * 8)) as u8);
            }
        }
    }

    // Ensure we have exactly the expected size
    result.truncate(uncompressed_size as usize);
    Ok(result)
}

/// Helper struct for reading bits from a byte stream
pub struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        BitReader {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    pub fn read_bits(&mut self, num_bits: u8) -> VgmResult<u16> {
        if num_bits > 16 {
            return Err(VgmError::InvalidDataFormat {
                field: "bit_count".to_string(),
                details: format!(
                    "Cannot read more than 16 bits at once, requested: {}",
                    num_bits
                ),
            });
        }

        let mut result = 0u16;
        let mut bits_read = 0;

        while bits_read < num_bits {
            if self.byte_pos >= self.data.len() {
                return Err(VgmError::BufferUnderflow {
                    offset: self.byte_pos,
                    needed: 1,
                    available: 0,
                });
            }

            let current_byte = self.data[self.byte_pos];
            let bits_available = 8 - self.bit_pos;
            let bits_to_read = (num_bits - bits_read).min(bits_available);

            // Extract bits from current byte (MSB first as per VGM spec)
            let mask = ((1u16 << bits_to_read) - 1) as u8;
            let shift = bits_available - bits_to_read;
            let bits = (current_byte >> shift) & mask;

            // Add to result
            result = (result << bits_to_read) | (bits as u16);
            bits_read += bits_to_read;

            // Update position
            self.bit_pos += bits_to_read;
            if self.bit_pos >= 8 {
                self.bit_pos = 0;
                self.byte_pos += 1;
            }
        }

        Ok(result)
    }
}
