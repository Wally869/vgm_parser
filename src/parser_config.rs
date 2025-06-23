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
            max_commands: 500_000,                      // 500K commands should be sufficient for most VGM files
            max_data_block_size: 4 * 1024 * 1024,      // 4MB per DataBlock (reduced from previous 16MB)
            max_total_data_block_memory: 32 * 1024 * 1024, // 32MB total for all DataBlocks
            max_metadata_size: 256 * 1024,             // 256KB metadata (UTF-16 can be large)
            max_chip_clock_entries: 32,                // Reasonable limit for chip configurations
            max_chip_volume_entries: 32,               // Reasonable limit for volume configurations
            strict_resource_limits: false,             // Conservative default
            max_command_memory: 64 * 1024 * 1024,      // 64MB for command vector
            max_parsing_depth: 16,                     // Prevent deep recursion
        }
    }
}

impl ParserConfig {
    /// Create a security-focused configuration with strict limits
    pub fn security_focused() -> Self {
        Self {
            max_commands: 100_000,                     // Stricter command limit
            max_data_block_size: 1 * 1024 * 1024,     // 1MB per DataBlock
            max_total_data_block_memory: 8 * 1024 * 1024, // 8MB total DataBlocks
            max_metadata_size: 64 * 1024,             // 64KB metadata
            max_chip_clock_entries: 16,               // Stricter chip limits
            max_chip_volume_entries: 16,              // Stricter volume limits
            strict_resource_limits: true,             // Enable all limits
            max_command_memory: 16 * 1024 * 1024,     // 16MB for commands
            max_parsing_depth: 8,                     // Shallow recursion only
        }
    }
    
    /// Create a permissive configuration for large/complex VGM files
    pub fn permissive() -> Self {
        Self {
            max_commands: 2_000_000,                   // Allow more commands
            max_data_block_size: 16 * 1024 * 1024,    // Original 16MB per block
            max_total_data_block_memory: 128 * 1024 * 1024, // 128MB total
            max_metadata_size: 1024 * 1024,           // 1MB metadata
            max_chip_clock_entries: 64,               // More chip entries
            max_chip_volume_entries: 64,              // More volume entries
            strict_resource_limits: false,            // Relaxed limits
            max_command_memory: 256 * 1024 * 1024,    // 256MB for commands
            max_parsing_depth: 32,                    // Deeper recursion allowed
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
    pub fn collect_with_limit<T, I>(&mut self, iter: I, expected_size: usize, purpose: &str) -> VgmResult<Vec<T>>
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
        assert!(config.max_commands > 0);
        assert!(config.max_data_block_size > 0);
        assert!(config.max_metadata_size > 0);
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
    fn test_allocation_guard() {
        let config = ParserConfig::default();
        let mut tracker = ResourceTracker::new();
        let mut guard = AllocationGuard::new(&mut tracker, &config);
        
        // Should be able to allocate reasonable sizes
        let vec: VgmResult<Vec<u8>> = guard.allocate_vec(1024, "test");
        assert!(vec.is_ok());
        
        // Should reject excessive allocations
        let huge_vec: VgmResult<Vec<u8>> = guard.allocate_vec(usize::MAX / 2, "huge_test");
        assert!(huge_vec.is_err());
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
}