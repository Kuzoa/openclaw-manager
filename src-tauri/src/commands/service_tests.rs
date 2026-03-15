#[cfg(test)]
mod tests {
    use crate::commands::service::{get_or_init_cache, log_status_change, ServiceStatusCache};
    use std::sync::Arc;
    use std::thread;

    /// Test ServiceStatusCache::default() returns { running: false, pid: None }
    #[test]
    fn test_cache_default() {
        let cache = ServiceStatusCache::default();
        assert!(!cache.running);
        assert!(cache.pid.is_none());
    }

    /// Test get_or_init_cache() returns the same static reference on multiple calls
    #[test]
    fn test_get_or_init_cache_returns_same_reference() {
        let cache1 = get_or_init_cache();
        let cache2 = get_or_init_cache();

        // Both should point to the same address
        assert!(std::ptr::eq(cache1, cache2));
    }

    /// Test log_status_change() detects state change (false -> true)
    #[test]
    fn test_status_change_false_to_true() {
        // Get the cache and reset it to a known state
        let cache = get_or_init_cache();
        {
            let mut last = cache.lock().unwrap();
            last.running = false;
            last.pid = None;
        }

        // Call log_status_change with running=true
        log_status_change(true, Some(12345));

        // Verify cache was updated
        let last = cache.lock().unwrap();
        assert!(last.running);
        assert_eq!(last.pid, Some(12345));
    }

    /// Test log_status_change() detects state change (true -> false)
    #[test]
    fn test_status_change_true_to_false() {
        // Get the cache and set it to running state
        let cache = get_or_init_cache();
        {
            let mut last = cache.lock().unwrap();
            last.running = true;
            last.pid = Some(12345);
        }

        // Call log_status_change with running=false
        log_status_change(false, None);

        // Verify cache was updated
        let last = cache.lock().unwrap();
        assert!(!last.running);
        assert!(last.pid.is_none());
    }

    /// Test log_status_change() detects no change (consecutive same values)
    #[test]
    fn test_status_unchanged() {
        // Get the cache and set it to running state
        let cache = get_or_init_cache();
        {
            let mut last = cache.lock().unwrap();
            last.running = true;
            last.pid = Some(12345);
        }

        // Call log_status_change with same state
        log_status_change(true, Some(12345));

        // Verify cache still has same values
        let last = cache.lock().unwrap();
        assert!(last.running);
        assert_eq!(last.pid, Some(12345));
    }

    /// Test LAST_STATUS cache concurrent access (multiple threads, no panic)
    #[test]
    fn test_concurrent_access() {
        let cache = Arc::new(get_or_init_cache());

        // Reset to known state
        {
            let mut last = cache.lock().unwrap();
            last.running = false;
            last.pid = None;
        }

        let mut handles = vec![];

        // Spawn multiple threads that call log_status_change
        for i in 0..10 {
            let handle = thread::spawn(move || {
                // Alternate between running and not running
                let running = i % 2 == 0;
                let pid = if running { Some(i * 1000) } else { None };
                log_status_change(running, pid);
            });
            handles.push(handle);
        }

        // All threads should complete without panic
        for handle in handles {
            handle.join().expect("Thread should not panic");
        }
    }
}
