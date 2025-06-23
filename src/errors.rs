use std::fmt;
use thiserror::Error;

/// Comprehensive error type for VGM parsing operations
/// 
/// This enum covers all possible error conditions that can occur during VGM file
/// parsing, validation, and processing. Each error includes contextual information
/// and machine-readable error codes for programmatic handling.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum VgmError {
    // ========== I/O ERRORS (1000-1099) ==========
    /// File not found at the specified path
    #[error("File not found: {path}")]
    FileNotFound { 
        path: String,
        io_kind: Option<std::io::ErrorKind>,
    },

    /// Error reading file contents
    #[error("Failed to read file {path}: {reason}")]
    FileReadError { 
        path: String, 
        reason: String,
    },

    /// Permission denied when accessing file
    #[error("Permission denied accessing file: {path}")]
    PermissionDenied { path: String },

    /// File is empty or too small to be a valid VGM
    #[error("File too small to be valid VGM: {path} ({size} bytes, minimum 64 required)")]
    FileTooSmall { path: String, size: usize },

    // ========== FORMAT VALIDATION ERRORS (2000-2099) ==========
    /// Invalid VGM magic bytes
    #[error("Invalid VGM magic bytes: expected 'Vgm ', found '{found}' at offset {offset}")]
    InvalidMagicBytes { 
        expected: String, 
        found: String, 
        offset: usize,
    },

    /// File header is corrupted or malformed
    #[error("Corrupted VGM header: {reason} at offset {offset}")]
    CorruptedHeader { reason: String, offset: usize },

    /// Invalid offset value in header
    #[error("Invalid offset in header: {field}={offset}, file size={file_size}")]
    InvalidOffset { 
        field: String, 
        offset: u32, 
        file_size: usize,
    },

    /// VGM file appears to be truncated
    #[error("Truncated VGM file: expected {expected} bytes, file ends at {actual}")]
    TruncatedFile { expected: usize, actual: usize },

    // ========== DATA PARSING ERRORS (3000-3099) ==========
    /// Invalid UTF-16 encoding in metadata
    #[error("Invalid UTF-16 encoding in {field}: {details}")]
    InvalidUtf16Encoding { 
        field: String, 
        details: String,
    },

    /// Invalid BCD (Binary-Coded Decimal) data
    #[error("Invalid BCD data for {field}: {data:02X?}")]
    InvalidBcdData { 
        field: String, 
        data: Vec<u8>,
    },

    /// Buffer underflow during parsing
    #[error("Buffer underflow at offset {offset}: needed {needed} bytes, only {available} available")]
    BufferUnderflow { 
        offset: usize, 
        needed: usize, 
        available: usize,
    },

    /// Invalid data length
    #[error("Invalid data length for {field}: expected {expected}, got {actual}")]
    InvalidDataLength { 
        field: String, 
        expected: usize, 
        actual: usize,
    },

    // ========== COMMAND PARSING ERRORS (4000-4099) ==========
    /// Unknown or unsupported command opcode
    #[error("Unknown command opcode 0x{opcode:02X} at position {position}")]
    UnknownCommand { opcode: u8, position: usize },

    /// Incomplete command data
    #[error("Incomplete command 0x{opcode:02X} at position {position}: expected {expected_bytes} bytes, only {available_bytes} available")]
    IncompleteCommand { 
        opcode: u8, 
        position: usize, 
        expected_bytes: usize, 
        available_bytes: usize,
    },

    /// Invalid command parameters
    #[error("Invalid parameters for command 0x{opcode:02X} at position {position}: {reason}")]
    InvalidCommandParameters { 
        opcode: u8, 
        position: usize, 
        reason: String,
    },

    /// Command parsing stack overflow (too many nested operations)
    #[error("Command parsing stack overflow at position {position}: maximum depth {max_depth} exceeded")]
    ParseStackOverflow { 
        position: usize, 
        max_depth: usize,
    },

    // ========== VERSION COMPATIBILITY ERRORS (5000-5099) ==========
    /// Unsupported VGM version
    #[error("Unsupported VGM version {version} (0x{version:08X}): supported versions are {supported_range}")]
    UnsupportedVgmVersion { 
        version: u32, 
        supported_range: String,
    },

    /// Unsupported GD3 version
    #[error("Unsupported GD3 version {version}: supported versions are {supported_versions:?}")]
    UnsupportedGd3Version { 
        version: u32, 
        supported_versions: Vec<u32>,
    },

    /// Feature not supported in this VGM version
    #[error("Feature '{feature}' not supported in VGM version {version}: requires version {min_version} or higher")]
    FeatureNotSupported { 
        feature: String, 
        version: u32, 
        min_version: u32,
    },

    // ========== MEMORY AND RESOURCE ERRORS (6000-6099) ==========
    /// Memory allocation failed
    #[error("Memory allocation failed: attempted to allocate {size} bytes for {purpose}")]
    MemoryAllocationFailed { size: usize, purpose: String },

    /// Integer overflow in calculations
    #[error("Integer overflow in {operation}: {details}")]
    IntegerOverflow { operation: String, details: String },

    /// Data size exceeds reasonable limits
    #[error("Data size exceeds limit for {field}: {size} bytes (limit: {limit})")]
    DataSizeExceedsLimit { 
        field: String, 
        size: usize, 
        limit: usize,
    },

    // ========== LOGICAL VALIDATION ERRORS (7000-7099) ==========
    /// Inconsistent data detected
    #[error("Data inconsistency in {context}: {reason}")]
    InconsistentData { context: String, reason: String },

    /// Invalid checksum or validation failure
    #[error("Validation failed for {field}: {reason}")]
    ValidationFailed { field: String, reason: String },

    /// Circular reference detected
    #[error("Circular reference detected in {structure} at {location}")]
    CircularReference { structure: String, location: String },

    // ========== DATA BLOCK ERRORS (8000-8099) ==========
    /// Invalid data block type
    #[error("Invalid data block type 0x{block_type:02X} at offset {offset}")]
    InvalidDataBlockType { block_type: u8, offset: usize },

    /// Data block size mismatch
    #[error("Data block size mismatch: header claims {header_size} bytes, actual block is {actual_size} bytes")]
    DataBlockSizeMismatch { 
        header_size: u32, 
        actual_size: usize,
    },

    /// Unsupported compression algorithm
    #[error("Unsupported compression algorithm in data block: {algorithm}")]
    UnsupportedCompression { algorithm: String },

    // ========== LEGACY COMPATIBILITY ==========
    /// Legacy error types for backward compatibility
    #[error("Invalid input provided to GD3 parser: {details}")]
    InvalidInputGd3Parser { details: String },

    #[error("Failed to parse GD3 data: {reason}")]
    FailedParseGd3 { reason: String },
}

impl VgmError {
    /// Get the error code for machine-readable processing
    pub fn code(&self) -> u16 {
        match self {
            // I/O Errors (1000-1099)
            Self::FileNotFound { .. } => 1001,
            Self::FileReadError { .. } => 1002,
            Self::PermissionDenied { .. } => 1003,
            Self::FileTooSmall { .. } => 1004,
            
            // Format Validation Errors (2000-2099)
            Self::InvalidMagicBytes { .. } => 2001,
            Self::CorruptedHeader { .. } => 2002,
            Self::InvalidOffset { .. } => 2003,
            Self::TruncatedFile { .. } => 2004,
            
            // Data Parsing Errors (3000-3099)
            Self::InvalidUtf16Encoding { .. } => 3001,
            Self::InvalidBcdData { .. } => 3002,
            Self::BufferUnderflow { .. } => 3003,
            Self::InvalidDataLength { .. } => 3004,
            
            // Command Parsing Errors (4000-4099)
            Self::UnknownCommand { .. } => 4001,
            Self::IncompleteCommand { .. } => 4002,
            Self::InvalidCommandParameters { .. } => 4003,
            Self::ParseStackOverflow { .. } => 4004,
            
            // Version Compatibility Errors (5000-5099)
            Self::UnsupportedVgmVersion { .. } => 5001,
            Self::UnsupportedGd3Version { .. } => 5002,
            Self::FeatureNotSupported { .. } => 5003,
            
            // Memory and Resource Errors (6000-6099)
            Self::MemoryAllocationFailed { .. } => 6001,
            Self::IntegerOverflow { .. } => 6002,
            Self::DataSizeExceedsLimit { .. } => 6003,
            
            // Logical Validation Errors (7000-7099)
            Self::InconsistentData { .. } => 7001,
            Self::ValidationFailed { .. } => 7002,
            Self::CircularReference { .. } => 7003,
            
            // Data Block Errors (8000-8099)
            Self::InvalidDataBlockType { .. } => 8001,
            Self::DataBlockSizeMismatch { .. } => 8002,
            Self::UnsupportedCompression { .. } => 8003,
            
            // Legacy Compatibility
            Self::InvalidInputGd3Parser { .. } => 9001,
            Self::FailedParseGd3 { .. } => 9002,
        }
    }

    /// Get the error category for grouping related errors
    pub fn category(&self) -> ErrorCategory {
        match self.code() {
            1000..=1099 => ErrorCategory::IO,
            2000..=2099 => ErrorCategory::FormatValidation,
            3000..=3099 => ErrorCategory::DataParsing,
            4000..=4099 => ErrorCategory::CommandParsing,
            5000..=5099 => ErrorCategory::VersionCompatibility,
            6000..=6099 => ErrorCategory::MemoryResource,
            7000..=7099 => ErrorCategory::LogicalValidation,
            8000..=8099 => ErrorCategory::DataBlock,
            9000..=9099 => ErrorCategory::Legacy,
            _ => ErrorCategory::Unknown,
        }
    }

    /// Check if the error is recoverable (parsing can continue)
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Non-recoverable I/O errors
            Self::FileNotFound { .. } 
            | Self::PermissionDenied { .. } 
            | Self::FileTooSmall { .. } => false,
            
            // Non-recoverable format errors
            Self::InvalidMagicBytes { .. } 
            | Self::CorruptedHeader { .. } 
            | Self::TruncatedFile { .. } => false,
            
            // Non-recoverable memory errors
            Self::MemoryAllocationFailed { .. } 
            | Self::IntegerOverflow { .. } 
            | Self::ParseStackOverflow { .. } => false,
            
            // Potentially recoverable errors
            Self::UnknownCommand { .. } 
            | Self::InvalidCommandParameters { .. } 
            | Self::UnsupportedGd3Version { .. } => true,
            
            // Other errors are generally non-recoverable
            _ => false,
        }
    }

    /// Get suggested action for handling this error
    pub fn suggested_action(&self) -> &'static str {
        match self {
            Self::FileNotFound { .. } => "Check file path and ensure file exists",
            Self::PermissionDenied { .. } => "Check file permissions and user access rights",
            Self::InvalidMagicBytes { .. } => "Verify this is a valid VGM file",
            Self::UnsupportedVgmVersion { .. } => "Use a VGM file with a supported version",
            Self::UnknownCommand { .. } => "File may use commands from a newer VGM specification",
            Self::BufferUnderflow { .. } => "File appears to be corrupted or truncated",
            Self::InvalidUtf16Encoding { .. } => "Metadata contains invalid text encoding",
            Self::MemoryAllocationFailed { .. } => "Reduce file size or increase available memory",
            _ => "Check file integrity and VGM specification compliance",
        }
    }
}

/// Error categories for grouping related error types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    IO,
    FormatValidation,
    DataParsing,
    CommandParsing,
    VersionCompatibility,
    MemoryResource,
    LogicalValidation,
    DataBlock,
    Legacy,
    Unknown,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO => write!(f, "I/O"),
            Self::FormatValidation => write!(f, "Format Validation"),
            Self::DataParsing => write!(f, "Data Parsing"),
            Self::CommandParsing => write!(f, "Command Parsing"),
            Self::VersionCompatibility => write!(f, "Version Compatibility"),
            Self::MemoryResource => write!(f, "Memory/Resource"),
            Self::LogicalValidation => write!(f, "Logical Validation"),
            Self::DataBlock => write!(f, "Data Block"),
            Self::Legacy => write!(f, "Legacy"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Result type alias for VGM operations
pub type VgmResult<T> = Result<T, VgmError>;

/// Context extension trait for adding context to errors
pub trait VgmErrorContext<T> {
    /// Add context to an error
    fn with_context<F>(self, f: F) -> VgmResult<T>
    where
        F: FnOnce() -> String;
    
    /// Add context with format arguments
    fn with_context_fmt<F>(self, f: F) -> VgmResult<T>
    where
        F: FnOnce() -> (String, String);
}

impl<T> VgmErrorContext<T> for VgmResult<T> {
    fn with_context<F>(self, f: F) -> VgmResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| match e {
            VgmError::InconsistentData { context, reason } => {
                VgmError::InconsistentData {
                    context: format!("{}: {}", f(), context),
                    reason,
                }
            }
            other => other,
        })
    }
    
    fn with_context_fmt<F>(self, f: F) -> VgmResult<T>
    where
        F: FnOnce() -> (String, String),
    {
        self.map_err(|_e| {
            let (context, reason) = f();
            VgmError::InconsistentData { context, reason }
        })
    }
}

// Implement From traits for common error conversions
impl From<std::io::Error> for VgmError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => VgmError::FileNotFound {
                path: "unknown".to_string(),
                io_kind: Some(err.kind()),
            },
            std::io::ErrorKind::PermissionDenied => VgmError::PermissionDenied {
                path: "unknown".to_string(),
            },
            _ => VgmError::FileReadError {
                path: "unknown".to_string(),
                reason: err.to_string(),
            },
        }
    }
}

impl From<std::string::FromUtf16Error> for VgmError {
    fn from(err: std::string::FromUtf16Error) -> Self {
        VgmError::InvalidUtf16Encoding {
            field: "unknown".to_string(),
            details: err.to_string(),
        }
    }
}

// Legacy type alias for backward compatibility
pub type LibError = VgmError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes_are_unique() {
        // Verify each error type has a unique code
        let mut codes = std::collections::HashSet::new();
        
        // Sample each error type with dummy data
        let errors = vec![
            VgmError::FileNotFound { path: "test".to_string(), io_kind: None },
            VgmError::FileReadError { path: "test".to_string(), reason: "test".to_string() },
            VgmError::PermissionDenied { path: "test".to_string() },
            VgmError::FileTooSmall { path: "test".to_string(), size: 0 },
            VgmError::InvalidMagicBytes { expected: "test".to_string(), found: "test".to_string(), offset: 0 },
            VgmError::CorruptedHeader { reason: "test".to_string(), offset: 0 },
            VgmError::InvalidOffset { field: "test".to_string(), offset: 0, file_size: 0 },
            VgmError::TruncatedFile { expected: 0, actual: 0 },
            VgmError::InvalidUtf16Encoding { field: "test".to_string(), details: "test".to_string() },
            VgmError::InvalidBcdData { field: "test".to_string(), data: vec![0] },
            VgmError::BufferUnderflow { offset: 0, needed: 0, available: 0 },
            VgmError::InvalidDataLength { field: "test".to_string(), expected: 0, actual: 0 },
            VgmError::UnknownCommand { opcode: 0, position: 0 },
            VgmError::IncompleteCommand { opcode: 0, position: 0, expected_bytes: 0, available_bytes: 0 },
            VgmError::InvalidCommandParameters { opcode: 0, position: 0, reason: "test".to_string() },
            VgmError::ParseStackOverflow { position: 0, max_depth: 0 },
            VgmError::UnsupportedVgmVersion { version: 0, supported_range: "test".to_string() },
            VgmError::UnsupportedGd3Version { version: 0, supported_versions: vec![1] },
            VgmError::FeatureNotSupported { feature: "test".to_string(), version: 0, min_version: 1 },
            VgmError::MemoryAllocationFailed { size: 0, purpose: "test".to_string() },
            VgmError::IntegerOverflow { operation: "test".to_string(), details: "test".to_string() },
            VgmError::DataSizeExceedsLimit { field: "test".to_string(), size: 0, limit: 0 },
            VgmError::InconsistentData { context: "test".to_string(), reason: "test".to_string() },
            VgmError::ValidationFailed { field: "test".to_string(), reason: "test".to_string() },
            VgmError::CircularReference { structure: "test".to_string(), location: "test".to_string() },
            VgmError::InvalidDataBlockType { block_type: 0, offset: 0 },
            VgmError::DataBlockSizeMismatch { header_size: 0, actual_size: 0 },
            VgmError::UnsupportedCompression { algorithm: "test".to_string() },
            VgmError::InvalidInputGd3Parser { details: "test".to_string() },
            VgmError::FailedParseGd3 { reason: "test".to_string() },
        ];

        for error in errors {
            let code = error.code();
            assert!(codes.insert(code), "Duplicate error code: {}", code);
        }
        
        // Verify we have at least 25 unique error types
        assert!(codes.len() >= 25, "Expected at least 25 unique error types, got {}", codes.len());
    }

    #[test]
    fn test_error_categories() {
        // Test that error codes map to correct categories
        let file_not_found = VgmError::FileNotFound { path: "test".to_string(), io_kind: None };
        assert_eq!(file_not_found.category(), ErrorCategory::IO);
        assert_eq!(file_not_found.code(), 1001);

        let invalid_magic = VgmError::InvalidMagicBytes { 
            expected: "Vgm ".to_string(), 
            found: "test".to_string(), 
            offset: 0 
        };
        assert_eq!(invalid_magic.category(), ErrorCategory::FormatValidation);
        assert_eq!(invalid_magic.code(), 2001);

        let unknown_command = VgmError::UnknownCommand { opcode: 0xFF, position: 100 };
        assert_eq!(unknown_command.category(), ErrorCategory::CommandParsing);
        assert_eq!(unknown_command.code(), 4001);
    }

    #[test]
    fn test_error_recoverability() {
        // Test recoverable errors
        let unknown_command = VgmError::UnknownCommand { opcode: 0xFF, position: 100 };
        assert!(unknown_command.is_recoverable());

        let unsupported_gd3 = VgmError::UnsupportedGd3Version { version: 999, supported_versions: vec![1, 2] };
        assert!(unsupported_gd3.is_recoverable());

        // Test non-recoverable errors
        let file_not_found = VgmError::FileNotFound { path: "test".to_string(), io_kind: None };
        assert!(!file_not_found.is_recoverable());

        let invalid_magic = VgmError::InvalidMagicBytes { 
            expected: "Vgm ".to_string(), 
            found: "test".to_string(), 
            offset: 0 
        };
        assert!(!invalid_magic.is_recoverable());
    }

    #[test]
    fn test_error_display() {
        // Test that error messages are properly formatted
        let file_error = VgmError::FileNotFound { 
            path: "/path/to/file.vgm".to_string(), 
            io_kind: Some(std::io::ErrorKind::NotFound)
        };
        let display_text = format!("{}", file_error);
        assert!(display_text.contains("/path/to/file.vgm"));
        assert!(display_text.contains("File not found"));

        let command_error = VgmError::UnknownCommand { opcode: 0xAB, position: 1234 };
        let display_text = format!("{}", command_error);
        assert!(display_text.contains("0xAB"));
        assert!(display_text.contains("1234"));
    }

    #[test]
    fn test_from_io_error() {
        // Test automatic conversion from std::io::Error
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let vgm_error = VgmError::from(io_error);
        
        match vgm_error {
            VgmError::FileNotFound { path, io_kind } => {
                assert_eq!(path, "unknown");
                assert_eq!(io_kind, Some(std::io::ErrorKind::NotFound));
            }
            _ => panic!("Expected FileNotFound error"),
        }

        let permission_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied");
        let vgm_error = VgmError::from(permission_error);
        
        match vgm_error {
            VgmError::PermissionDenied { path } => {
                assert_eq!(path, "unknown");
            }
            _ => panic!("Expected PermissionDenied error"),
        }
    }

    #[test]
    fn test_from_utf16_error() {
        // Test automatic conversion from UTF-16 errors
        let invalid_utf16 = vec![0xD800]; // Invalid UTF-16 surrogate
        let utf16_error = String::from_utf16(&invalid_utf16).unwrap_err();
        let vgm_error = VgmError::from(utf16_error);
        
        match vgm_error {
            VgmError::InvalidUtf16Encoding { field, details: _ } => {
                assert_eq!(field, "unknown");
            }
            _ => panic!("Expected InvalidUtf16Encoding error"),
        }
    }

    #[test]
    fn test_suggested_actions() {
        // Test that suggested actions are meaningful
        let file_error = VgmError::FileNotFound { path: "test".to_string(), io_kind: None };
        let suggestion = file_error.suggested_action();
        assert!(suggestion.to_lowercase().contains("file"));
        assert!(suggestion.to_lowercase().contains("path") || suggestion.to_lowercase().contains("exists"));

        let magic_error = VgmError::InvalidMagicBytes { 
            expected: "Vgm ".to_string(), 
            found: "test".to_string(), 
            offset: 0 
        };
        let suggestion = magic_error.suggested_action();
        assert!(suggestion.to_lowercase().contains("vgm"));
        assert!(suggestion.to_lowercase().contains("valid"));
    }

    #[test]
    fn test_error_category_display() {
        // Test error category display formatting
        assert_eq!(format!("{}", ErrorCategory::IO), "I/O");
        assert_eq!(format!("{}", ErrorCategory::FormatValidation), "Format Validation");
        assert_eq!(format!("{}", ErrorCategory::CommandParsing), "Command Parsing");
        assert_eq!(format!("{}", ErrorCategory::VersionCompatibility), "Version Compatibility");
    }

    #[test]
    fn test_vgm_result_type_alias() {
        // Test that VgmResult works as expected
        fn test_function() -> VgmResult<String> {
            Ok("success".to_string())
        }
        
        fn test_error_function() -> VgmResult<String> {
            Err(VgmError::FileNotFound { path: "test".to_string(), io_kind: None })
        }
        
        assert!(test_function().is_ok());
        assert!(test_error_function().is_err());
    }

    #[test]
    fn test_legacy_compatibility() {
        // Test that LibError type alias works
        let _legacy_error: LibError = VgmError::FileNotFound { 
            path: "test".to_string(), 
            io_kind: None 
        };
        
        // Test legacy error types
        let gd3_error = VgmError::InvalidInputGd3Parser { details: "test".to_string() };
        assert_eq!(gd3_error.code(), 9001);
        assert_eq!(gd3_error.category(), ErrorCategory::Legacy);
    }

    #[test]
    fn test_error_debug_formatting() {
        // Test that Debug trait works correctly
        let error = VgmError::BufferUnderflow { offset: 100, needed: 10, available: 5 };
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("BufferUnderflow"));
        assert!(debug_str.contains("100"));
        assert!(debug_str.contains("10"));
        assert!(debug_str.contains("5"));
    }
}
