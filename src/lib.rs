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
    use std::path::PathBuf;
    use super::*;

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

    #[test]
    fn test_vgm_parse_write_cycle() {
        // Use project-relative paths
        let test_file = project_path("vgm_files/Into Battle.vgm");
        
        // Skip test if no test files available
        if !test_file.exists() {
            println!("Skipping test_vgm_parse_write_cycle - test VGM file not found at {:?}", test_file);
            return;
        }

        // Parse the file
        let vgm = VgmFile::from_path(test_file.to_str().expect("Invalid path encoding"));

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
