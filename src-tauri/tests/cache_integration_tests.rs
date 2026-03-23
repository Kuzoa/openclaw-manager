use openclaw_manager::utils::cache::{CacheEntry, EnvironmentCache, EnvironmentCacheFile, ENVIRONMENT_CACHE};
use std::collections::HashMap;
use tempfile::TempDir;

/// Test 7.4: Integration test for cache lifecycle
/// Tests: first check (cache miss), second check (cache hit), and invalidation
#[test]
fn test_cache_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    // Use global cache instance
    let cache = &*ENVIRONMENT_CACHE;
    cache.set_cache_dir(cache_dir.clone());

    let cache_file = cache_dir.join("environment.json");

    // Step 1: First check - cache miss, should create cache file
    assert!(
        !cache_file.exists(),
        "Cache file should not exist initially"
    );

    let mut cache_data = EnvironmentCacheFile {
        version: 1,
        cache: HashMap::new(),
    };

    cache_data.cache.insert(
        "npm_prefix".to_string(),
        CacheEntry {
            value: "/usr/local".to_string(),
            cached_at: chrono::Utc::now().to_rfc3339(),
            ttl_seconds: 604800,
        },
    );

    cache.save_cache_to_file(&cache_data);

    assert!(
        cache_file.exists(),
        "Cache file should be created after first check"
    );

    // Step 2: Second check - cache hit, should use cached paths
    let loaded = cache.load_cache_from_file();
    assert!(
        loaded.is_some(),
        "Cache should be loaded on second check"
    );

    let loaded_data = loaded.unwrap();
    assert_eq!(
        loaded_data.cache.get("npm_prefix").unwrap().value,
        "/usr/local",
        "Should use cached npm_prefix path"
    );

    // Step 3: Invalidate cache
    cache.invalidate();

    assert!(
        !cache_file.exists(),
        "Cache file should be deleted after invalidation"
    );

    let loaded = cache.load_cache_from_file();
    assert_eq!(
        loaded, None,
        "Should be cache miss after invalidation"
    );
}

