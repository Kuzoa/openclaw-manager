# Performance Tests Guide

This document explains how to run and interpret the performance tests for the environment cache optimization.

## Overview

The performance tests are located in `src-tauri/tests/performance_tests.rs` and are divided into two categories:

| Type | Purpose | CI-Friendly | Real Shell Commands |
|------|---------|-------------|---------------------|
| Simulated Tests | Validate cache I/O performance | Yes | No |
| Real Tests | Measure actual performance gains | No | Yes |

## Running Tests

### Simulated Tests (Default)

These tests run in CI and don't require a real environment:

```bash
# Run simulated tests only
cargo test --test performance_tests

# Or run all non-ignored tests
cargo test
```

### Real Tests (Local Only)

These tests require a real environment with Node.js, npm, and Git installed:

```bash
# Run real performance tests
# IMPORTANT: Use --test-threads=1 to run tests serially and avoid cache contention
cargo test --test performance_tests -- --ignored --test-threads=1

# Run all tests including ignored
cargo test --test performance_tests -- --include-ignored --test-threads=1
```

**Why --test-threads=1?**

The real tests use the global `ENVIRONMENT_CACHE` instance. Running them in parallel causes:
- Cache contention between tests
- RwLock contention leading to slow execution or deadlocks
- Inconsistent results

Using `--test-threads=1` ensures tests run serially and produce reliable results.

## Test Descriptions

### Simulated Tests

#### `test_cache_file_io_performance_simulated`

Measures the time to serialize/deserialize cache data to/from JSON files.

**Expected Results:**
- Save: < 50ms
- Load: < 50ms

#### `test_cache_validation_performance_simulated`

Measures the time to validate cache entries (TTL check, path existence).

**Expected Results:**
- Validation: < 10ms

### Real Tests

#### `test_real_npm_prefix_detection_performance`

Measures the performance of `npm config get prefix` detection:

1. **Cache Miss**: Executes real `npm config get prefix` command
2. **Memory Cache Hit**: Returns cached value from memory
3. **File Cache Hit**: Loads cache from file (after memory invalidate)

**Expected Results:**
- Cache miss: 1-3 seconds (depends on system)
- Memory cache hit: < 1ms
- File cache hit: < 50ms

#### `test_real_full_environment_check_performance`

Measures the performance of full environment detection including:
- npm_prefix
- openclaw_path
- node_version
- git_version

**Expected Results:**
- Full check (uncached): 3-8 seconds
- Full check (cached): < 100ms

#### `test_real_performance_comparison_report`

Runs multiple iterations and produces a detailed performance report with:
- Warmup runs to eliminate cold start effects
- Multiple measurement runs for statistical significance
- Average, min, max statistics
- Speedup calculations

## Interpreting Results

### Performance Report Example

```
╔════════════════════════════════════════════════════════════════╗
║         Performance Exploration Test Report                    ║
╚════════════════════════════════════════════════════════════════╝

┌─────────────────────────────────────────────────────────────────┐
│ npm_prefix Detection Performance                                │
├─────────────────────────────────────────────────────────────────┤
│ Run 1: Uncached 1.234s      │
│ Run 2: Uncached 1.189s      │
│ ...
│ Run 1: Cached   45µs       │
│ Run 2: Cached   38µs       │
│ ...
├─────────────────────────────────────────────────────────────────┤
│ Average Uncached: 1.210s                                        │
│ Average Cached:   42µs                                          │
│ Speedup:          28,809.5x                                     │
└─────────────────────────────────────────────────────────────────┘
```

### Key Metrics

- **Speedup**: Ratio of uncached time to cached time
- **Expected speedup for memory cache**: 10,000x - 100,000x
- **Expected speedup for file cache**: 10x - 100x

## Manual Testing in Full Application

For complete end-to-end testing, run the full Tauri application:

### Windows

1. Build and run the application:
   ```bash
   npm run tauri dev
   ```

2. Observe the startup time in the console logs

3. Check the cache file location:
   ```
   %APPDATA%\com.openclaw.manager\cache\environment.json
   ```

4. To test cache hit:
   - Close and restart the application
   - The second startup should be significantly faster

5. To test cache miss:
   - Delete the cache file
   - Restart the application
   - The startup should take longer

### macOS

1. Build and run the application:
   ```bash
   npm run tauri dev
   ```

2. Check the cache file location:
   ```
   ~/Library/Application Support/com.openclaw.manager/cache/environment.json
   ```

### Linux

1. Build and run the application:
   ```bash
   npm run tauri dev
   ```

2. Check the cache file location:
   ```
   ~/.config/com.openclaw.manager/cache/environment.json
   ```

## Troubleshooting

### Tests Fail with "npm not found"

Ensure Node.js and npm are installed and accessible in your PATH:

```bash
npm --version
node --version
```

### Real Tests Show No Performance Difference

This could indicate:
1. Cache is not being used properly
2. Cache invalidation is not working
3. The environment is already cached from a previous run

Try running with a fresh cache:
```bash
# Delete cache and run tests
rm -rf ~/.config/com.openclaw.manager/cache
cargo test --test performance_tests -- --ignored
```

### Inconsistent Results

Performance can vary due to:
- System load
- Disk I/O performance
- Antivirus software (Windows)
- Background processes

For more consistent results:
- Close other applications
- Run multiple times and compare averages
- Use the `test_real_performance_comparison_report` test which runs multiple iterations

## Notes

- Real tests are marked with `#[ignore]` to prevent them from running in CI
- No hard performance thresholds are enforced in real tests - they only report data
- The tests use `tempfile::TempDir` to avoid polluting the real cache directory
