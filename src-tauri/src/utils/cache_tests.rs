#[cfg(test)]
mod tests {
    use crate::utils::cache::EnvironmentCache;
    use std::sync::Arc;
    use std::thread;

    /// Test that invalidate() clears all cached values to None
    #[test]
    fn test_invalidate_clears_all_fields() {
        let cache = EnvironmentCache::new();

        // Initialize some values by manually setting them
        // Using the new CachedValue format: Some(Some(value)) = initialized with value
        {
            let mut npm_prefix = cache.npm_prefix.write().unwrap();
            *npm_prefix = Some(Some("/test/npm".to_string()));
        }
        {
            let mut openclaw_path = cache.openclaw_path.write().unwrap();
            *openclaw_path = Some(Some("/test/openclaw".to_string()));
        }
        {
            let mut openclaw_version = cache.openclaw_version.write().unwrap();
            *openclaw_version = Some(Some("2026.1.29".to_string()));
        }
        {
            let mut node_version = cache.node_version.write().unwrap();
            *node_version = Some(Some("v22.0.0".to_string()));
        }
        {
            let mut git_version = cache.git_version.write().unwrap();
            *git_version = Some(Some("2.43.0".to_string()));
        }
        {
            let mut is_secure = cache.is_secure.write().unwrap();
            *is_secure = Some(Some(true));
        }

        // Verify values are set (is_some() returns true for initialized cache)
        assert!(cache.npm_prefix.read().unwrap().is_some());
        assert!(cache.openclaw_path.read().unwrap().is_some());
        assert!(cache.openclaw_version.read().unwrap().is_some());
        assert!(cache.node_version.read().unwrap().is_some());
        assert!(cache.git_version.read().unwrap().is_some());
        assert!(cache.is_secure.read().unwrap().is_some());

        // Invalidate cache
        cache.invalidate();

        // Verify all values are now None (not initialized)
        assert!(cache.npm_prefix.read().unwrap().is_none());
        assert!(cache.openclaw_path.read().unwrap().is_none());
        assert!(cache.openclaw_version.read().unwrap().is_none());
        assert!(cache.node_version.read().unwrap().is_none());
        assert!(cache.git_version.read().unwrap().is_none());
        assert!(cache.is_secure.read().unwrap().is_none());
    }

    /// Test that cache correctly stores None values (e.g., when OpenClaw is not installed)
    #[test]
    fn test_cache_stores_none_value() {
        let cache = EnvironmentCache::new();

        // Simulate a failed detection by setting Some(None)
        {
            let mut openclaw_path = cache.openclaw_path.write().unwrap();
            *openclaw_path = Some(None); // Initialized but value is None
        }

        // The cache should recognize this as initialized
        assert!(cache.openclaw_path.read().unwrap().is_some());
        // But the inner value should be None
        assert!(cache
            .openclaw_path
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .is_none());
    }

    /// Test concurrent read access to cache
    #[test]
    fn test_concurrent_read_access() {
        let cache = Arc::new(EnvironmentCache::new());

        // Set a value
        {
            let mut version = cache.openclaw_version.write().unwrap();
            *version = Some(Some("2026.1.29".to_string()));
        }

        let mut handles = vec![];

        // Spawn multiple threads that read from the cache
        for _ in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                let version = cache_clone.openclaw_version.read().unwrap();
                version.as_ref().unwrap().clone()
            });
            handles.push(handle);
        }

        // All threads should read the same value
        for handle in handles {
            let result = handle.join().unwrap();
            assert_eq!(result, Some("2026.1.29".to_string()));
        }
    }

    /// Test concurrent write access to cache (invalidate)
    #[test]
    fn test_concurrent_write_access() {
        let cache = Arc::new(EnvironmentCache::new());

        // Set some values
        {
            let mut version = cache.openclaw_version.write().unwrap();
            *version = Some(Some("2026.1.29".to_string()));
        }

        let cache_clone = Arc::clone(&cache);

        // Invalidate from one thread while reading from another
        let invalidate_handle = thread::spawn(move || {
            cache_clone.invalidate();
        });

        // Read should block or complete, but not panic
        let read_handle = thread::spawn(move || {
            let version = cache.openclaw_version.read().unwrap();
            version.clone()
        });

        invalidate_handle.join().unwrap();
        let result = read_handle.join().unwrap();

        // After invalidate, result should be None
        assert!(result.is_none());
    }

    /// Test that multiple invalidates are safe
    #[test]
    fn test_multiple_invalidates() {
        let cache = EnvironmentCache::new();

        // Set a value
        {
            let mut version = cache.openclaw_version.write().unwrap();
            *version = Some(Some("2026.1.29".to_string()));
        }

        // Multiple invalidates should be safe
        cache.invalidate();
        cache.invalidate();
        cache.invalidate();

        // Value should still be None
        assert!(cache.openclaw_version.read().unwrap().is_none());
    }

    /// Test that cache can be used after invalidate
    #[test]
    fn test_cache_usable_after_invalidate() {
        let cache = EnvironmentCache::new();

        // Set and then invalidate
        {
            let mut version = cache.openclaw_version.write().unwrap();
            *version = Some(Some("2026.1.29".to_string()));
        }
        cache.invalidate();

        // Should be able to set again
        {
            let mut version = cache.openclaw_version.write().unwrap();
            *version = Some(Some("2026.2.1".to_string()));
        }

        // And read the new value
        assert_eq!(
            cache
                .openclaw_version
                .read()
                .unwrap()
                .as_ref()
                .unwrap()
                .clone(),
            Some("2026.2.1".to_string())
        );
    }
}
