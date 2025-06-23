pub mod errors;
pub mod header;
pub mod metadata;
pub mod systems;
pub mod traits;
pub mod utils;
pub mod vgm_commands;

pub use errors::*;
pub use header::*;
pub use metadata::*;
pub use systems::*;
pub use traits::*;
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
    pub fn from_path(path: &str) -> Self {
        let file_data = std::fs::read(path).unwrap();
        let mut data = Bytes::from(file_data);
        
        VgmFile::from_bytes(&mut data)
    }

    pub fn has_data_block(&self) -> bool {
        for cmd in &self.commands {
            if let Commands::DataBlock { .. } = cmd { return true }
        }
        false
    }

    pub fn has_pcm_write(&self) -> bool {
        for cmd in &self.commands {
            if let Commands::PCMRAMWrite { .. } = cmd { return true }
        }
        false
    }
}

impl VgmParser for VgmFile {
    fn from_bytes(data: &mut Bytes) -> Self {
        let len_data = data.len();
        let header_data = HeaderData::from_bytes(data);
        let vgm_start_pos = header_data.vgm_data_offset as usize + 0x34;

        while len_data - data.len() < vgm_start_pos {
            data.get_u8();
        }

        VgmFile {
            header: header_data,
            commands: parse_commands(data),
            metadata: VgmMetadata::from_bytes(data),
        }
    }
}

impl VgmWriter for VgmFile {
    fn to_bytes(&self, buffer: &mut BytesMut) {
        self.header.to_bytes(buffer);
        write_commands(buffer, &self.commands);
        self.metadata.to_bytes(buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vgm_parse_write_cycle() {
        // Skip test if no test files available
        let test_file = "./vgm_files/Into Battle.vgm";
        if !std::path::Path::new(test_file).exists() {
            println!("Skipping test - no test VGM file found");
            return;
        }

        // Parse the file
        let vgm = VgmFile::from_path(test_file);

        // Basic assertions
        assert_eq!(vgm.header.version, 151); // v1.51
        assert!(!vgm.commands.is_empty());

        // Test round-trip
        let mut buffer = BytesMut::new();
        vgm.to_bytes(&mut buffer);

        // Parse again
        let mut data = Bytes::from(buffer.to_vec());
        let vgm2 = VgmFile::from_bytes(&mut data);

        // Compare
        assert_eq!(vgm.header.version, vgm2.header.version);
        assert_eq!(vgm.commands.len(), vgm2.commands.len());
    }
}
