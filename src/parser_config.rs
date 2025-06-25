use crate::errors::{VgmError, VgmResult};

/// Configuration for resource management and security limits during VGM parsing
///
/// This struct controls memory allocation limits and parsing constraints to prevent
/// DoS attacks and resource exhaustion. It's separate from ValidationConfig which
/// handles semantic validation of parsed data.
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Maximum number of commands to parse before stopping (prevents infinite loops)
    pub max_commands: usize,

    /// Maximum size for a single DataBlock allocation (bytes)
    pub max_data_block_size: u32,

    /// Maximum total memory that can be allocated for DataBlocks per file (bytes)
    pub max_total_data_block_memory: usize,

    /// Maximum size for metadata section before processing (bytes)
    pub max_metadata_size: usize,

    /// Maximum number of chip clock entries allowed per extra header
    pub max_chip_clock_entries: u8,

    /// Maximum number of chip volume entries allowed per extra header
    pub max_chip_volume_entries: u8,

    /// Whether to enable aggressive resource tracking and limits
    pub strict_resource_limits: bool,

    /// Maximum memory that can be allocated for command vector (bytes)
    pub max_command_memory: usize,

    /// Maximum depth for nested parsing operations
    pub max_parsing_depth: u32,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_commands: 500_000, // 500K commands should be sufficient for most VGM files
            max_data_block_size: 4 * 1024 * 1024, // 4MB per DataBlock (reduced from previous 16MB)
            max_total_data_block_memory: 32 * 1024 * 1024, // 32MB total for all DataBlocks
            max_metadata_size: 256 * 1024, // 256KB metadata (UTF-16 can be large)
            max_chip_clock_entries: 32, // Reasonable limit for chip configurations
            max_chip_volume_entries: 32, // Reasonable limit for volume configurations
            strict_resource_limits: false, // Conservative default
            max_command_memory: 64 * 1024 * 1024, // 64MB for command vector
            max_parsing_depth: 16, // Prevent deep recursion
        }
    }
}

impl ParserConfig {
    /// Create a security-focused configuration with strict limits
    pub fn security_focused() -> Self {
        Self {
            max_commands: 100_000,                        // Stricter command limit
            max_data_block_size: 1024 * 1024,             // 1MB per DataBlock
            max_total_data_block_memory: 8 * 1024 * 1024, // 8MB total DataBlocks
            max_metadata_size: 64 * 1024,                 // 64KB metadata
            max_chip_clock_entries: 16,                   // Stricter chip limits
            max_chip_volume_entries: 16,                  // Stricter volume limits
            strict_resource_limits: true,                 // Enable all limits
            max_command_memory: 16 * 1024 * 1024,         // 16MB for commands
            max_parsing_depth: 8,                         // Shallow recursion only
        }
    }

    /// Create a permissive configuration for large/complex VGM files
    pub fn permissive() -> Self {
        Self {
            max_commands: 2_000_000,                        // Allow more commands
            max_data_block_size: 16 * 1024 * 1024,          // Original 16MB per block
            max_total_data_block_memory: 128 * 1024 * 1024, // 128MB total
            max_metadata_size: 1024 * 1024,                 // 1MB metadata
            max_chip_clock_entries: 64,                     // More chip entries
            max_chip_volume_entries: 64,                    // More volume entries
            strict_resource_limits: false,                  // Relaxed limits
            max_command_memory: 256 * 1024 * 1024,          // 256MB for commands
            max_parsing_depth: 32,                          // Deeper recursion allowed
        }
    }

    /// Estimate memory usage for a given number of commands
    pub fn estimate_command_memory(&self, command_count: usize) -> usize {
        // Conservative estimate: each command takes ~100 bytes on average
        // (this includes the enum variant overhead and potential data)
        command_count * 100
    }

    /// Check if command count is within limits
    pub fn check_command_count(&self, count: usize) -> VgmResult<()> {
        if count > self.max_commands {
            return Err(VgmError::DataSizeExceedsLimit {
                field: "command_count".to_string(),
                size: count,
                limit: self.max_commands,
            });
        }
        Ok(())
    }

    /// Check if command memory usage is within limits
    pub fn check_command_memory(&self, count: usize) -> VgmResult<()> {
        let estimated_memory = self.estimate_command_memory(count);
        if estimated_memory > self.max_command_memory {
            return Err(VgmError::DataSizeExceedsLimit {
                field: "command_memory".to_string(),
                size: estimated_memory,
                limit: self.max_command_memory,
            });
        }
        Ok(())
    }

    /// Check if DataBlock size is acceptable
    pub fn check_data_block_size(&self, size: u32) -> VgmResult<()> {
        if size > self.max_data_block_size {
            return Err(VgmError::DataSizeExceedsLimit {
                field: "data_block_size".to_string(),
                size: size as usize,
                limit: self.max_data_block_size as usize,
            });
        }
        Ok(())
    }

    /// Check if metadata size is acceptable before processing
    pub fn check_metadata_size(&self, size: usize) -> VgmResult<()> {
        if size > self.max_metadata_size {
            return Err(VgmError::DataSizeExceedsLimit {
                field: "metadata_size".to_string(),
                size,
                limit: self.max_metadata_size,
            });
        }
        Ok(())
    }

    /// Check if chip entry count is reasonable
    pub fn check_chip_entries(&self, clock_entries: u8, volume_entries: u8) -> VgmResult<()> {
        if clock_entries > self.max_chip_clock_entries {
            return Err(VgmError::DataSizeExceedsLimit {
                field: "chip_clock_entries".to_string(),
                size: clock_entries as usize,
                limit: self.max_chip_clock_entries as usize,
            });
        }

        if volume_entries > self.max_chip_volume_entries {
            return Err(VgmError::DataSizeExceedsLimit {
                field: "chip_volume_entries".to_string(),
                size: volume_entries as usize,
                limit: self.max_chip_volume_entries as usize,
            });
        }

        Ok(())
    }
}

/// Resource tracker for monitoring memory usage during parsing
#[derive(Debug, Default)]
pub struct ResourceTracker {
    /// Current number of parsed commands
    pub command_count: usize,

    /// Total memory allocated for DataBlocks
    pub data_block_memory: usize,

    /// Current parsing depth
    pub parsing_depth: u32,

    /// Number of DataBlocks encountered
    pub data_block_count: usize,
}

impl ResourceTracker {
    /// Create a new resource tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Track a new command being parsed
    pub fn track_command(&mut self, config: &ParserConfig) -> VgmResult<()> {
        self.command_count += 1;

        // Check command count limit
        config.check_command_count(self.command_count)?;

        // Check command memory estimate
        if config.strict_resource_limits {
            config.check_command_memory(self.command_count)?;
        }

        Ok(())
    }

    /// Track a DataBlock allocation
    pub fn track_data_block(&mut self, config: &ParserConfig, size: u32) -> VgmResult<()> {
        // Check individual block size
        config.check_data_block_size(size)?;

        // Check total memory usage
        let new_total = self.data_block_memory + size as usize;
        if new_total > config.max_total_data_block_memory {
            return Err(VgmError::DataSizeExceedsLimit {
                field: "total_data_block_memory".to_string(),
                size: new_total,
                limit: config.max_total_data_block_memory,
            });
        }

        self.data_block_memory = new_total;
        self.data_block_count += 1;

        Ok(())
    }

    /// Track parsing depth (for nested operations)
    pub fn enter_parsing_context(&mut self, config: &ParserConfig) -> VgmResult<()> {
        self.parsing_depth += 1;

        if self.parsing_depth > config.max_parsing_depth {
            return Err(VgmError::ParseStackOverflow {
                position: 0, // TODO: Track actual position
                max_depth: config.max_parsing_depth as usize,
            });
        }

        Ok(())
    }

    /// Exit parsing context
    pub fn exit_parsing_context(&mut self) {
        if self.parsing_depth > 0 {
            self.parsing_depth -= 1;
        }
    }

    /// Get current resource usage summary
    pub fn get_usage_summary(&self) -> ResourceUsageSummary {
        ResourceUsageSummary {
            command_count: self.command_count,
            data_block_memory_mb: self.data_block_memory as f64 / (1024.0 * 1024.0),
            data_block_count: self.data_block_count,
            parsing_depth: self.parsing_depth,
        }
    }
}

/// Summary of resource usage for monitoring and debugging
#[derive(Debug, Clone)]
pub struct ResourceUsageSummary {
    pub command_count: usize,
    pub data_block_memory_mb: f64,
    pub data_block_count: usize,
    pub parsing_depth: u32,
}

impl std::fmt::Display for ResourceUsageSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Commands: {}, DataBlocks: {} ({:.1}MB), Depth: {}",
            self.command_count,
            self.data_block_count,
            self.data_block_memory_mb,
            self.parsing_depth
        )
    }
}

/// Allocation guard for safe memory allocation with limits
pub struct AllocationGuard<'a> {
    tracker: &'a mut ResourceTracker,
    config: &'a ParserConfig,
}

impl<'a> AllocationGuard<'a> {
    pub fn new(tracker: &'a mut ResourceTracker, config: &'a ParserConfig) -> Self {
        Self { tracker, config }
    }

    /// Safely allocate a vector with size checking
    pub fn allocate_vec<T>(&mut self, size: usize, purpose: &str) -> VgmResult<Vec<T>> {
        let byte_size = size * std::mem::size_of::<T>();

        // Basic size sanity check
        if byte_size > self.config.max_command_memory {
            return Err(VgmError::MemoryAllocationFailed {
                size: byte_size,
                purpose: purpose.to_string(),
            });
        }

        // Attempt allocation
        let mut vec = Vec::new();
        match vec.try_reserve(size) {
            Ok(()) => Ok(vec),
            Err(_) => Err(VgmError::MemoryAllocationFailed {
                size: byte_size,
                purpose: purpose.to_string(),
            }),
        }
    }

    /// Safely allocate with capacity and collect from iterator
    pub fn collect_with_limit<T, I>(
        &mut self,
        iter: I,
        expected_size: usize,
        purpose: &str,
    ) -> VgmResult<Vec<T>>
    where
        I: Iterator<Item = T>,
        T: Clone,
    {
        let mut vec = self.allocate_vec::<T>(expected_size, purpose)?;

        for (index, item) in iter.enumerate() {
            if index >= expected_size {
                return Err(VgmError::DataSizeExceedsLimit {
                    field: purpose.to_string(),
                    size: index + 1,
                    limit: expected_size,
                });
            }
            vec.push(item);
        }

        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_config_defaults() {
        let config = ParserConfig::default();
        
        // Test all default values are reasonable
        assert_eq!(config.max_commands, 500_000);
        assert_eq!(config.max_data_block_size, 4 * 1024 * 1024);
        assert_eq!(config.max_total_data_block_memory, 32 * 1024 * 1024);
        assert_eq!(config.max_metadata_size, 256 * 1024);
        assert_eq!(config.max_chip_clock_entries, 32);
        assert_eq!(config.max_chip_volume_entries, 32);
        assert!(!config.strict_resource_limits);
        assert_eq!(config.max_command_memory, 64 * 1024 * 1024);
        assert_eq!(config.max_parsing_depth, 16);
    }

    #[test]
    fn test_security_focused_config() {
        let default_config = ParserConfig::default();
        let security_config = ParserConfig::security_focused();

        // Security config should have stricter limits
        assert!(security_config.max_commands <= default_config.max_commands);
        assert!(security_config.max_data_block_size <= default_config.max_data_block_size);
        assert!(security_config.max_metadata_size <= default_config.max_metadata_size);
        assert!(security_config.strict_resource_limits);
        
        // Test specific security values
        assert_eq!(security_config.max_commands, 100_000);
        assert_eq!(security_config.max_data_block_size, 1024 * 1024);
        assert_eq!(security_config.max_total_data_block_memory, 8 * 1024 * 1024);
        assert_eq!(security_config.max_metadata_size, 64 * 1024);
        assert_eq!(security_config.max_chip_clock_entries, 16);
        assert_eq!(security_config.max_chip_volume_entries, 16);
        assert_eq!(security_config.max_command_memory, 16 * 1024 * 1024);
        assert_eq!(security_config.max_parsing_depth, 8);
    }

    #[test]
    fn test_permissive_config() {
        let default_config = ParserConfig::default();
        let permissive_config = ParserConfig::permissive();

        // Permissive config should have more generous limits
        assert!(permissive_config.max_commands >= default_config.max_commands);
        assert!(permissive_config.max_data_block_size >= default_config.max_data_block_size);
        assert!(permissive_config.max_metadata_size >= default_config.max_metadata_size);
        assert!(!permissive_config.strict_resource_limits);
        
        // Test specific permissive values
        assert_eq!(permissive_config.max_commands, 2_000_000);
        assert_eq!(permissive_config.max_data_block_size, 16 * 1024 * 1024);
        assert_eq!(permissive_config.max_total_data_block_memory, 128 * 1024 * 1024);
        assert_eq!(permissive_config.max_metadata_size, 1024 * 1024);
        assert_eq!(permissive_config.max_chip_clock_entries, 64);
        assert_eq!(permissive_config.max_chip_volume_entries, 64);
        assert_eq!(permissive_config.max_command_memory, 256 * 1024 * 1024);
        assert_eq!(permissive_config.max_parsing_depth, 32);
    }

    #[test]
    fn test_estimate_command_memory() {
        let config = ParserConfig::default();
        
        // Test memory estimation
        assert_eq!(config.estimate_command_memory(0), 0);
        assert_eq!(config.estimate_command_memory(1), 100);
        assert_eq!(config.estimate_command_memory(10), 1000);
        assert_eq!(config.estimate_command_memory(1000), 100_000);
    }

    #[test]
    fn test_check_command_count() {
        let config = ParserConfig::default();
        
        // Should accept counts within limit
        assert!(config.check_command_count(0).is_ok());
        assert!(config.check_command_count(1000).is_ok());
        assert!(config.check_command_count(config.max_commands).is_ok());
        
        // Should reject counts exceeding limit
        assert!(config.check_command_count(config.max_commands + 1).is_err());
        assert!(config.check_command_count(usize::MAX).is_err());
        
        // Test error type
        let error = config.check_command_count(config.max_commands + 1).unwrap_err();
        assert!(matches!(error, VgmError::DataSizeExceedsLimit { .. }));
    }

    #[test]
    fn test_check_command_memory() {
        let config = ParserConfig::default();
        
        // Calculate command count that would exceed memory limit
        let max_commands_by_memory = config.max_command_memory / 100;
        
        // Should accept reasonable command counts
        assert!(config.check_command_memory(1000).is_ok());
        assert!(config.check_command_memory(max_commands_by_memory).is_ok());
        
        // Should reject excessive memory usage
        assert!(config.check_command_memory(max_commands_by_memory + 1).is_err());
    }

    #[test]
    fn test_check_data_block_size() {
        let config = ParserConfig::default();
        
        // Should accept sizes within limit
        assert!(config.check_data_block_size(0).is_ok());
        assert!(config.check_data_block_size(1024).is_ok());
        assert!(config.check_data_block_size(config.max_data_block_size).is_ok());
        
        // Should reject sizes exceeding limit
        assert!(config.check_data_block_size(config.max_data_block_size + 1).is_err());
        assert!(config.check_data_block_size(u32::MAX).is_err());
    }

    #[test]
    fn test_check_metadata_size() {
        let config = ParserConfig::default();
        
        // Should accept sizes within limit
        assert!(config.check_metadata_size(0).is_ok());
        assert!(config.check_metadata_size(1024).is_ok());
        assert!(config.check_metadata_size(config.max_metadata_size).is_ok());
        
        // Should reject sizes exceeding limit
        assert!(config.check_metadata_size(config.max_metadata_size + 1).is_err());
        assert!(config.check_metadata_size(usize::MAX).is_err());
    }

    #[test]
    fn test_check_chip_entries() {
        let config = ParserConfig::default();
        
        // Should accept entries within limits
        assert!(config.check_chip_entries(0, 0).is_ok());
        assert!(config.check_chip_entries(16, 16).is_ok());
        assert!(config.check_chip_entries(config.max_chip_clock_entries, config.max_chip_volume_entries).is_ok());
        
        // Should reject clock entries exceeding limit
        assert!(config.check_chip_entries(config.max_chip_clock_entries + 1, 0).is_err());
        
        // Should reject volume entries exceeding limit
        assert!(config.check_chip_entries(0, config.max_chip_volume_entries + 1).is_err());
        
        // Should reject both exceeding limits
        assert!(config.check_chip_entries(config.max_chip_clock_entries + 1, config.max_chip_volume_entries + 1).is_err());
    }

    #[test]
    fn test_resource_tracker_new() {
        let tracker = ResourceTracker::new();
        
        // Should initialize with zero values
        assert_eq!(tracker.command_count, 0);
        assert_eq!(tracker.data_block_memory, 0);
        assert_eq!(tracker.parsing_depth, 0);
        assert_eq!(tracker.data_block_count, 0);
    }

    #[test]
    fn test_resource_tracker_default() {
        let tracker = ResourceTracker::default();
        
        // Should be same as new()
        assert_eq!(tracker.command_count, 0);
        assert_eq!(tracker.data_block_memory, 0);
        assert_eq!(tracker.parsing_depth, 0);
        assert_eq!(tracker.data_block_count, 0);
    }

    #[test]
    fn test_resource_tracker() {
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();

        // Should be able to track commands within limit
        for _ in 0..100 {
            assert!(tracker.track_command(&config).is_ok());
        }

        assert_eq!(tracker.command_count, 100);
    }

    #[test]
    fn test_track_command_with_strict_limits() {
        let mut config = ParserConfig::default();
        config.strict_resource_limits = true;
        config.max_command_memory = 400; // Very low limit - allows 4 commands (4 * 100 = 400)
        
        let mut tracker = ResourceTracker::new();
        
        // Should accept 4 commands (400 bytes / 100 bytes per command = 4)
        for _ in 0..4 {
            assert!(tracker.track_command(&config).is_ok());
        }
        
        // Should reject 5th command that would exceed memory limit (5 * 100 = 500 > 400)
        assert!(tracker.track_command(&config).is_err());
    }

    #[test]
    fn test_data_block_tracking() {
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();

        // Should be able to track reasonable DataBlock sizes
        assert!(tracker.track_data_block(&config, 1024).is_ok());
        assert!(tracker.track_data_block(&config, 2048).is_ok());

        assert_eq!(tracker.data_block_memory, 1024 + 2048);
        assert_eq!(tracker.data_block_count, 2);
    }

    #[test]
    fn test_data_block_size_limit() {
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();

        // Should reject blocks that are too large
        let oversized_block = config.max_data_block_size + 1;
        assert!(tracker.track_data_block(&config, oversized_block).is_err());
    }

    #[test]
    fn test_total_data_block_memory_limit() {
        let mut config = ParserConfig::default();
        config.max_total_data_block_memory = 1024; // 1KB limit
        config.max_data_block_size = 512; // 512B per block

        let mut tracker = ResourceTracker::new();

        // Should accept first block
        assert!(tracker.track_data_block(&config, 512).is_ok());

        // Should accept second block
        assert!(tracker.track_data_block(&config, 512).is_ok());

        // Should reject third block (would exceed total limit)
        assert!(tracker.track_data_block(&config, 1).is_err());
    }

    #[test]
    fn test_command_count_limit() {
        let mut config = ParserConfig::default();
        config.max_commands = 5; // Very low limit for testing

        let mut tracker = ResourceTracker::new();

        // Should accept commands within limit
        for _ in 0..5 {
            assert!(tracker.track_command(&config).is_ok());
        }

        // Should reject command that exceeds limit
        assert!(tracker.track_command(&config).is_err());
    }

    #[test]
    fn test_parsing_depth_tracking() {
        let mut config = ParserConfig::default();
        config.max_parsing_depth = 3;

        let mut tracker = ResourceTracker::new();

        // Should accept depth within limit
        assert!(tracker.enter_parsing_context(&config).is_ok()); // depth 1
        assert!(tracker.enter_parsing_context(&config).is_ok()); // depth 2
        assert!(tracker.enter_parsing_context(&config).is_ok()); // depth 3

        // Should reject depth that exceeds limit
        assert!(tracker.enter_parsing_context(&config).is_err()); // depth 4 - should fail

        // Should allow depth to decrease and then succeed again
        tracker.exit_parsing_context(); // depth 3
        tracker.exit_parsing_context(); // depth 2
        assert!(tracker.enter_parsing_context(&config).is_ok()); // depth 3 again
    }

    #[test]
    fn test_parsing_depth_underflow_protection() {
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        
        // Should handle exit when depth is 0
        tracker.exit_parsing_context();
        assert_eq!(tracker.parsing_depth, 0);
        
        // Should still work normally after underflow attempt
        assert!(tracker.enter_parsing_context(&config).is_ok());
        assert_eq!(tracker.parsing_depth, 1);
    }

    #[test]
    fn test_get_usage_summary() {
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        
        // Add some usage
        tracker.track_command(&config).unwrap();
        tracker.track_command(&config).unwrap();
        tracker.track_data_block(&config, 1024 * 1024).unwrap(); // 1MB
        tracker.enter_parsing_context(&config).unwrap();
        
        let summary = tracker.get_usage_summary();
        
        assert_eq!(summary.command_count, 2);
        assert_eq!(summary.data_block_count, 1);
        assert_eq!(summary.parsing_depth, 1);
        assert!((summary.data_block_memory_mb - 1.0).abs() < 0.01); // Should be ~1.0 MB
    }

    #[test]
    fn test_resource_usage_summary_display() {
        let summary = ResourceUsageSummary {
            command_count: 1000,
            data_block_memory_mb: 2.5,
            data_block_count: 3,
            parsing_depth: 2,
        };
        
        let display_str = format!("{}", summary);
        assert!(display_str.contains("1000"));
        assert!(display_str.contains("2.5"));
        assert!(display_str.contains("3"));
        assert!(display_str.contains("2"));
        assert!(display_str.contains("Commands"));
        assert!(display_str.contains("DataBlocks"));
        assert!(display_str.contains("MB"));
        assert!(display_str.contains("Depth"));
    }

    #[test]
    fn test_allocation_guard_new() {
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        let _guard = AllocationGuard::new(&mut tracker, &config);
        // Just test that creation works
    }

    #[test]
    fn test_allocation_guard() {
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        let mut guard = AllocationGuard::new(&mut tracker, &config);

        // Should be able to allocate reasonable sizes
        let vec: VgmResult<Vec<u8>> = guard.allocate_vec(1024, "test");
        assert!(vec.is_ok());
        let vec = vec.unwrap();
        assert!(vec.capacity() >= 1024); // Should have reserved capacity

        // Should reject excessive allocations
        let huge_vec: VgmResult<Vec<u8>> = guard.allocate_vec(usize::MAX / 2, "huge_test");
        assert!(huge_vec.is_err());
    }

    #[test]
    fn test_allocation_guard_different_types() {
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        let mut guard = AllocationGuard::new(&mut tracker, &config);

        // Test different types
        let u8_vec: VgmResult<Vec<u8>> = guard.allocate_vec(100, "u8_test");
        assert!(u8_vec.is_ok());
        
        let u32_vec: VgmResult<Vec<u32>> = guard.allocate_vec(100, "u32_test");
        assert!(u32_vec.is_ok());
        
        let string_vec: VgmResult<Vec<String>> = guard.allocate_vec(100, "string_test");
        assert!(string_vec.is_ok());
    }

    #[test]
    fn test_collect_with_limit() {
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        let mut guard = AllocationGuard::new(&mut tracker, &config);

        // Should collect within limits
        let data = vec![1, 2, 3, 4, 5];
        let result = guard.collect_with_limit(data.into_iter(), 10, "test_collect");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 5);

        // Should reject collections that exceed expected size
        let large_data: Vec<u32> = (0..1000).collect();
        let result = guard.collect_with_limit(large_data.into_iter(), 5, "test_exceed");
        assert!(result.is_err());
    }

    #[test]
    fn test_collect_with_limit_exact_size() {
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        let mut guard = AllocationGuard::new(&mut tracker, &config);

        // Test exact size match
        let data = vec![1, 2, 3, 4, 5];
        let result = guard.collect_with_limit(data.into_iter(), 5, "exact_size");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 5);
    }

    #[test]
    fn test_collect_with_limit_empty() {
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        let mut guard = AllocationGuard::new(&mut tracker, &config);

        // Test empty collection
        let data: Vec<u32> = vec![];
        let result = guard.collect_with_limit(data.into_iter(), 10, "empty_test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_config_debugging() {
        let config = ParserConfig::default();
        
        // Test Debug formatting works
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ParserConfig"));
        assert!(debug_str.contains("max_commands"));
    }

    #[test]
    fn test_tracker_debugging() {
        let tracker = ResourceTracker::new();
        
        // Test Debug formatting works
        let debug_str = format!("{:?}", tracker);
        assert!(debug_str.contains("ResourceTracker"));
        assert!(debug_str.contains("command_count"));
    }

    #[test]
    fn test_resource_summary_debugging() {
        let summary = ResourceUsageSummary {
            command_count: 100,
            data_block_memory_mb: 1.5,
            data_block_count: 2,
            parsing_depth: 1,
        };
        
        // Test Debug formatting works
        let debug_str = format!("{:?}", summary);
        assert!(debug_str.contains("ResourceUsageSummary"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_error_types_from_config_checks() {
        let config = ParserConfig::default();
        
        // Test all check methods return proper error types
        let command_error = config.check_command_count(config.max_commands + 1).unwrap_err();
        assert!(matches!(command_error, VgmError::DataSizeExceedsLimit { field, .. } if field == "command_count"));
        
        // Use a large but safe value to avoid overflow
        let large_count = config.max_command_memory / 50; // Safe multiplication
        let memory_error = config.check_command_memory(large_count).unwrap_err();
        assert!(matches!(memory_error, VgmError::DataSizeExceedsLimit { field, .. } if field == "command_memory"));
        
        let block_error = config.check_data_block_size(config.max_data_block_size + 1).unwrap_err();
        assert!(matches!(block_error, VgmError::DataSizeExceedsLimit { field, .. } if field == "data_block_size"));
        
        let meta_error = config.check_metadata_size(config.max_metadata_size + 1).unwrap_err();
        assert!(matches!(meta_error, VgmError::DataSizeExceedsLimit { field, .. } if field == "metadata_size"));
        
        let chip_clock_error = config.check_chip_entries(config.max_chip_clock_entries + 1, 0).unwrap_err();
        assert!(matches!(chip_clock_error, VgmError::DataSizeExceedsLimit { field, .. } if field == "chip_clock_entries"));
        
        let chip_volume_error = config.check_chip_entries(0, config.max_chip_volume_entries + 1).unwrap_err();
        assert!(matches!(chip_volume_error, VgmError::DataSizeExceedsLimit { field, .. } if field == "chip_volume_entries"));
    }

    #[test]
    fn test_error_types_from_tracker() {
        let mut config = ParserConfig::default();
        config.max_parsing_depth = 1;
        
        let mut tracker = ResourceTracker::new();
        tracker.enter_parsing_context(&config).unwrap();
        
        let depth_error = tracker.enter_parsing_context(&config).unwrap_err();
        assert!(matches!(depth_error, VgmError::ParseStackOverflow { .. }));
    }

    #[test] 
    fn test_edge_case_zero_limits() {
        let mut config = ParserConfig::default();
        config.max_commands = 0;
        config.max_data_block_size = 0;
        config.max_metadata_size = 0;
        config.max_parsing_depth = 0;
        
        let mut tracker = ResourceTracker::new();
        
        // Should reject any command with zero limit
        assert!(tracker.track_command(&config).is_err());
        
        // Should reject any non-zero data block with zero limit  
        assert!(tracker.track_data_block(&config, 1).is_err());
        // Zero-sized data block should still pass since check is size > max_size
        assert!(tracker.track_data_block(&config, 0).is_ok());
        
        // Should reject any non-zero metadata with zero limit
        assert!(config.check_metadata_size(1).is_err());
        // Zero-sized metadata should still pass
        assert!(config.check_metadata_size(0).is_ok());
        
        // Should reject any parsing depth with zero limit
        assert!(tracker.enter_parsing_context(&config).is_err());
    }

    #[test]
    fn test_resource_tracker_integration() {
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        
        // Simulate realistic parsing scenario
        for _ in 0..1000 {
            tracker.track_command(&config).unwrap();
        }
        
        tracker.track_data_block(&config, 1024 * 1024).unwrap(); // 1MB
        tracker.track_data_block(&config, 512 * 1024).unwrap();  // 512KB
        
        tracker.enter_parsing_context(&config).unwrap();
        tracker.enter_parsing_context(&config).unwrap();
        
        let summary = tracker.get_usage_summary();
        assert_eq!(summary.command_count, 1000);
        assert_eq!(summary.data_block_count, 2);
        assert_eq!(summary.parsing_depth, 2);
        assert!((summary.data_block_memory_mb - 1.5).abs() < 0.01); // ~1.5MB
        
        tracker.exit_parsing_context();
        tracker.exit_parsing_context();
        
        assert_eq!(tracker.parsing_depth, 0);
    }
}
