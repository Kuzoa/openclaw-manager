//! Performance exploration tests for environment cache optimization
//!
//! This file contains two types of tests:
//! 1. Simulated tests (CI-friendly) - Test cache I/O performance without real shell commands
//! 2. Real tests (local only) - Test actual performance with real shell commands
//!
//! # Running Tests
//!
//! ```bash
//! # Run simulated tests (CI-friendly)
//! cargo test --test performance_tests
//!
//! # Run real performance tests (requires local environment)
//! # Note: Use --test-threads=1 to run tests serially and avoid cache contention
//! cargo test --test performance_tests -- --ignored --test-threads=1
//! ```

use openclaw_manager::utils::cache::{
    CacheEntry, EnvironmentCache, EnvironmentCacheFile, ENVIRONMENT_CACHE,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tempfile::TempDir;

// =============================================================================
// Simulated Tests (CI-friendly)
// =============================================================================

/// Test 7.9.1: Measure cache file I/O performance (simulated)
#[test]
fn test_cache_file_io_performance_simulated() {
    println!("\n═══════════════════════════════════════════════════════");
    println!("Simulated Test: Cache File I/O Performance");
    println!("═══════════════════════════════════════════════════════\n");

    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    // Create a mock cache instance for testing (NOT using global instance)
    let cache = EnvironmentCache::new();
    cache.set_cache_dir(cache_dir.clone());

    // Prepare test data
    let mut cache_data = EnvironmentCacheFile {
        version: 1,
        cache: HashMap::new(),
    };

    cache_data.cache.insert(
        "npm_prefix".to_string(),
        CacheEntry {
            value: r"C:\Users\test\AppData\Roaming\npm".to_string(),
            cached_at: chrono::Utc::now().to_rfc3339(),
            ttl_seconds: 604800,
        },
    );

    // Measure save performance (10 iterations)
    let mut save_times = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        cache.save_cache_to_file(&cache_data);
        save_times.push(start.elapsed());
    }

    // Measure load performance (10 iterations)
    let mut load_times = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        let _ = cache.load_cache_from_file();
        load_times.push(start.elapsed());
    }

    let avg_save = average_duration(&save_times);
    let avg_load = average_duration(&load_times);

    println!("Cache File Save Performance: {:?}", avg_save);
    println!("Cache File Load Performance: {:?}", avg_load);

    assert!(avg_save < Duration::from_millis(50));
    assert!(avg_load < Duration::from_millis(50));

    println!("✅ Cache file I/O performance is acceptable");
}

/// Test 7.9.2: Measure cache validation performance (simulated)
#[test]
fn test_cache_validation_performance_simulated() {
    println!("\n═══════════════════════════════════════════════════════");
    println!("Simulated Test: Cache Validation Performance");
    println!("═══════════════════════════════════════════════════════\n");

    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    let cache = EnvironmentCache::new();
    cache.set_cache_dir(cache_dir.clone());

    let mut cache_data = EnvironmentCacheFile {
        version: 1,
        cache: HashMap::new(),
    };

    cache_data.cache.insert(
        "npm_prefix".to_string(),
        CacheEntry {
            value: temp_dir.path().to_string_lossy().to_string(),
            cached_at: chrono::Utc::now().to_rfc3339(),
            ttl_seconds: 604800,
        },
    );

    cache.save_cache_to_file(&cache_data);

    let mut validation_times = Vec::new();
    for _ in 0..100 {
        let start = Instant::now();
        let _ = cache.load_cache_from_file();
        validation_times.push(start.elapsed());
    }

    let avg_validation = average_duration(&validation_times);
    println!("Cache Validation Performance: {:?}", avg_validation);

    assert!(avg_validation < Duration::from_millis(10));

    println!("✅ Cache validation performance is acceptable");
}

// =============================================================================
// Real Tests (Local only, requires real environment)
// =============================================================================
//
// IMPORTANT: These tests use the global ENVIRONMENT_CACHE instance.
// Run with --test-threads=1 to avoid parallel execution and cache contention.
//

/// Test 7.9.3: Real npm prefix detection performance (simplified)
///
/// This test measures the performance of npm_prefix detection.
#[test]
#[ignore = "Requires real environment. Run with: cargo test -- --ignored --test-threads=1"]
fn test_real_npm_prefix_performance() {
    println!("\n═══════════════════════════════════════════════════════");
    println!("Real Test: npm prefix Detection Performance");
    println!("═══════════════════════════════════════════════════════\n");

    let temp_dir = TempDir::new().unwrap();
    let cache = &*ENVIRONMENT_CACHE;
    cache.set_cache_dir(temp_dir.path().to_path_buf());

    // Clear cache
    cache.invalidate();

    // Measure uncached
    println!("Measuring uncached npm_prefix detection...");
    let start = Instant::now();
    let npm_prefix = cache.get_npm_prefix();
    let uncached_time = start.elapsed();
    println!("  Result: {:?}", npm_prefix);
    println!("  Time:   {:?}", uncached_time);
    println!();

    // Measure cached (memory)
    println!("Measuring cached npm_prefix detection...");
    let start = Instant::now();
    let npm_prefix_cached = cache.get_npm_prefix();
    let cached_time = start.elapsed();
    println!("  Result: {:?}", npm_prefix_cached);
    println!("  Time:   {:?}", cached_time);
    println!();

    // Report
    println!("─────────────────────────────────────────────────────────");
    println!("Performance Comparison:");
    println!("  Uncached: {:?}", uncached_time);
    println!("  Cached:   {:?}", cached_time);
    if cached_time.as_nanos() > 0 {
        println!(
            "  Speedup:  {:.1}x",
            uncached_time.as_secs_f64() / cached_time.as_secs_f64()
        );
    }
    println!("─────────────────────────────────────────────────────────");

    println!("✅ Test completed");
}

/// Test 7.9.4: Full environment check performance
///
/// This test measures the performance of a complete environment check
/// including npm_prefix, openclaw_path, node_version, and git_version.
#[test]
#[ignore = "Requires real environment. Run with: cargo test -- --ignored --test-threads=1"]
fn test_real_full_environment_performance() {
    println!("\n═══════════════════════════════════════════════════════");
    println!("Real Test: Full Environment Check Performance");
    println!("═══════════════════════════════════════════════════════\n");

    let temp_dir = TempDir::new().unwrap();
    let cache = &*ENVIRONMENT_CACHE;
    cache.set_cache_dir(temp_dir.path().to_path_buf());

    // Clear cache
    cache.invalidate();

    // Measure full environment check (uncached)
    println!("Measuring full environment check (uncached)...");
    let start = Instant::now();
    
    let npm_prefix = cache.get_npm_prefix();
    let npm_time = start.elapsed();
    
    let start = Instant::now();
    let openclaw_path = cache.get_openclaw_path();
    let openclaw_time = start.elapsed();
    
    let start = Instant::now();
    let node_version = cache.get_node_version();
    let node_time = start.elapsed();
    
    let start = Instant::now();
    let git_version = cache.get_git_version();
    let git_time = start.elapsed();
    
    let total_uncached = npm_time + openclaw_time + node_time + git_time;

    println!("  npm_prefix:      {:?} ({:?})", npm_prefix, npm_time);
    println!("  openclaw_path:   {:?} ({:?})", openclaw_path, openclaw_time);
    println!("  node_version:    {:?} ({:?})", node_version, node_time);
    println!("  git_version:     {:?} ({:?})", git_version, git_time);
    println!("  Total:           {:?}", total_uncached);
    println!();

    // Measure full environment check (cached)
    println!("Measuring full environment check (cached)...");
    let start = Instant::now();
    
    let _ = cache.get_npm_prefix();
    let npm_time_cached = start.elapsed();
    
    let start = Instant::now();
    let _ = cache.get_openclaw_path();
    let openclaw_time_cached = start.elapsed();
    
    let start = Instant::now();
    let _ = cache.get_node_version();
    let node_time_cached = start.elapsed();
    
    let start = Instant::now();
    let _ = cache.get_git_version();
    let git_time_cached = start.elapsed();
    
    let total_cached = npm_time_cached + openclaw_time_cached + node_time_cached + git_time_cached;

    println!("  npm_prefix:      {:?}", npm_time_cached);
    println!("  openclaw_path:   {:?}", openclaw_time_cached);
    println!("  node_version:    {:?}", node_time_cached);
    println!("  git_version:     {:?}", git_time_cached);
    println!("  Total:           {:?}", total_cached);
    println!();

    // Report
    println!("─────────────────────────────────────────────────────────");
    println!("Performance Comparison:");
    println!("  Total Uncached: {:?}", total_uncached);
    println!("  Total Cached:   {:?}", total_cached);
    if total_cached.as_nanos() > 0 {
        println!(
            "  Speedup:        {:.1}x",
            total_uncached.as_secs_f64() / total_cached.as_secs_f64()
        );
    }
    println!("─────────────────────────────────────────────────────────");

    println!("✅ Test completed");
}

/// Test 7.9.5: Real openclaw_path detection performance
#[test]
#[ignore = "Requires real environment. Run with: cargo test -- --ignored --test-threads=1"]
fn test_real_openclaw_path_performance() {
    println!("\n═══════════════════════════════════════════════════════");
    println!("Real Test: openclaw_path Detection Performance");
    println!("═══════════════════════════════════════════════════════\n");

    let temp_dir = TempDir::new().unwrap();
    let cache = &*ENVIRONMENT_CACHE;
    cache.set_cache_dir(temp_dir.path().to_path_buf());

    // Clear cache
    cache.invalidate();

    // Measure uncached
    println!("Measuring uncached openclaw_path detection...");
    let start = Instant::now();
    let openclaw_path = cache.get_openclaw_path();
    let uncached_time = start.elapsed();
    println!("  Result: {:?}", openclaw_path);
    println!("  Time:   {:?}", uncached_time);
    println!();

    // Measure cached (memory)
    println!("Measuring cached openclaw_path detection...");
    let start = Instant::now();
    let openclaw_path_cached = cache.get_openclaw_path();
    let cached_time = start.elapsed();
    println!("  Result: {:?}", openclaw_path_cached);
    println!("  Time:   {:?}", cached_time);
    println!();

    // Report
    println!("─────────────────────────────────────────────────────────");
    println!("Performance Comparison:");
    println!("  Uncached: {:?}", uncached_time);
    println!("  Cached:   {:?}", cached_time);
    if cached_time.as_nanos() > 0 {
        println!(
            "  Speedup:  {:.1}x",
            uncached_time.as_secs_f64() / cached_time.as_secs_f64()
        );
    }
    println!("─────────────────────────────────────────────────────────");

    println!("✅ Test completed");
}

// =============================================================================
// Helper Functions
// =============================================================================

fn average_duration(durations: &[Duration]) -> Duration {
    if durations.is_empty() {
        return Duration::ZERO;
    }
    let total: Duration = durations.iter().sum();
    total / durations.len() as u32
}
