import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { act } from '@testing-library/react';
import { useAppStore } from '@/stores/appStore';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

vi.mock('@/lib/tauri', () => ({
  isTauri: vi.fn(() => true),
}));

const { invoke } = await import('@tauri-apps/api/core');
const { listen } = await import('@tauri-apps/api/event');
const { isTauri } = await import('@/lib/tauri');

const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);
const mockIsTauri = vi.mocked(isTauri);

describe('Progress Listener', () => {
  // Store the original progressListenerRef to reset between tests
  let mockUnlisten: vi.Mock;

  beforeEach(() => {
    // Reset store to initial state
    useAppStore.setState({
      serviceStatus: null,
      systemInfo: null,
      environment: null,
      isCheckingEnvironment: false,
      environmentError: null,
      checkProgress: 0,
      checkCompletedStep: null,
      loading: false,
      notifications: [],
    });

    // Reset mocks
    mockInvoke.mockReset();
    mockListen.mockReset();
    mockIsTauri.mockReset();
    mockIsTauri.mockReturnValue(true);

    // Create mock unlisten function
    mockUnlisten = vi.fn();

    // Reset the internal progressListenerRef by calling cleanup
    const { cleanupProgressListener } = useAppStore.getState();
    cleanupProgressListener();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('12.5 Singleton Pattern - Prevent Duplicate Registration', () => {
    it('should register listener only once', async () => {
      mockListen.mockResolvedValueOnce(mockUnlisten);

      const { setupProgressListener } = useAppStore.getState();

      // First call - should register
      await setupProgressListener();
      expect(mockListen).toHaveBeenCalledTimes(1);

      // Second call - should be skipped (singleton)
      await setupProgressListener();
      expect(mockListen).toHaveBeenCalledTimes(1); // Still 1, not 2
    });

    it('should not register if not in Tauri environment', async () => {
      mockIsTauri.mockReturnValue(false);

      const { setupProgressListener } = useAppStore.getState();
      await setupProgressListener();

      expect(mockListen).not.toHaveBeenCalled();
    });

    it('should allow re-registration after cleanup', async () => {
      mockListen.mockResolvedValueOnce(mockUnlisten);

      const { setupProgressListener, cleanupProgressListener } = useAppStore.getState();

      // First registration
      await setupProgressListener();
      expect(mockListen).toHaveBeenCalledTimes(1);

      // Cleanup
      cleanupProgressListener();

      // Re-registration should work
      mockListen.mockResolvedValueOnce(mockUnlisten);
      await setupProgressListener();
      expect(mockListen).toHaveBeenCalledTimes(2);
    });
  });

  describe('12.6 Progress State Reset Logic', () => {
    it('should reset progress state when refreshEnvironment starts', async () => {
      // Set initial progress state
      useAppStore.setState({
        checkProgress: 75,
        checkCompletedStep: 'openclaw',
      });

      // Mock successful environment check
      mockInvoke.mockResolvedValueOnce(undefined); // invalidate_environment_cache
      mockInvoke.mockResolvedValueOnce({
        node_installed: true,
        node_version: 'v22.0.0',
        node_version_ok: true,
        git_installed: true,
        git_version: '2.42.0',
        openclaw_installed: true,
        openclaw_version: '1.0.0',
        gateway_service_installed: true,
        config_dir_exists: true,
        ready: true,
        os: 'Windows',
        is_secure: true,
        detection_steps: [],
      });

      const { refreshEnvironment } = useAppStore.getState();

      // Start refresh (don't await to check intermediate state)
      const refreshPromise = refreshEnvironment();

      // Check state was reset at start
      const stateDuringRefresh = useAppStore.getState();
      expect(stateDuringRefresh.checkProgress).toBe(0);
      expect(stateDuringRefresh.checkCompletedStep).toBeNull();
      expect(stateDuringRefresh.isCheckingEnvironment).toBe(true);

      await refreshPromise;

      // After completion, progress should still be reset
      const finalState = useAppStore.getState();
      expect(finalState.checkProgress).toBe(0);
      expect(finalState.checkCompletedStep).toBeNull();
    });

    it('should reset progress state when checkEnvironment starts', async () => {
      // Set initial progress state
      useAppStore.setState({
        checkProgress: 50,
        checkCompletedStep: 'git',
        environment: null, // Ensure check will run
      });

      // Mock successful environment check
      mockInvoke.mockResolvedValueOnce({
        node_installed: true,
        node_version: 'v22.0.0',
        node_version_ok: true,
        git_installed: true,
        git_version: '2.42.0',
        openclaw_installed: true,
        openclaw_version: '1.0.0',
        gateway_service_installed: true,
        config_dir_exists: true,
        ready: true,
        os: 'Windows',
        is_secure: true,
        detection_steps: [],
      });

      const { checkEnvironment } = useAppStore.getState();

      const checkPromise = checkEnvironment();

      // Check state was reset at start
      const stateDuringCheck = useAppStore.getState();
      expect(stateDuringCheck.checkProgress).toBe(0);
      expect(stateDuringCheck.checkCompletedStep).toBeNull();

      await checkPromise;
    });

    it('should reset progress state on error', async () => {
      // Set initial progress state
      useAppStore.setState({
        checkProgress: 80,
        checkCompletedStep: 'gateway',
      });

      // Mock failed environment check
      mockInvoke.mockRejectedValueOnce(new Error('Network error'));

      const { refreshEnvironment } = useAppStore.getState();

      await refreshEnvironment();

      const state = useAppStore.getState();
      expect(state.checkProgress).toBe(0);
      expect(state.checkCompletedStep).toBeNull();
      expect(state.environmentError).toContain('Network error');
    });
  });

  describe('12.7 React Strict Mode Compatibility', () => {
    it('should correctly cleanup and re-register listener (simulating Strict Mode)', async () => {
      mockListen.mockResolvedValue(mockUnlisten);

      const { setupProgressListener, cleanupProgressListener } = useAppStore.getState();

      // Simulate Strict Mode: mount -> unmount -> remount
      // Phase 1: First mount
      await setupProgressListener();
      expect(mockListen).toHaveBeenCalledTimes(1);

      // Phase 2: Unmount (cleanup)
      cleanupProgressListener();
      expect(mockUnlisten).toHaveBeenCalledTimes(1);

      // Verify state was reset
      expect(useAppStore.getState().checkProgress).toBe(0);
      expect(useAppStore.getState().checkCompletedStep).toBeNull();

      // Phase 3: Remount
      await setupProgressListener();
      expect(mockListen).toHaveBeenCalledTimes(2);
    });

    it('should handle multiple cleanup calls gracefully', () => {
      const { cleanupProgressListener } = useAppStore.getState();

      // First cleanup
      cleanupProgressListener();

      // Second cleanup should not throw
      expect(() => cleanupProgressListener()).not.toThrow();

      // State should remain reset
      expect(useAppStore.getState().checkProgress).toBe(0);
    });

    it('should handle setup after multiple cleanups', async () => {
      mockListen.mockResolvedValueOnce(mockUnlisten);

      const { setupProgressListener, cleanupProgressListener } = useAppStore.getState();

      // Multiple cleanups
      cleanupProgressListener();
      cleanupProgressListener();
      cleanupProgressListener();

      // Setup should still work
      await setupProgressListener();
      expect(mockListen).toHaveBeenCalledTimes(1);
    });

    it('should update progress when event is received', async () => {
      let eventCallback: ((event: { payload: { completed_count: number; total_count: number; completed_step: string } }) => void) | null = null;

      mockListen.mockImplementationOnce(async (eventName, callback) => {
        if (eventName === 'env-check-progress') {
          eventCallback = callback as typeof eventCallback;
        }
        return mockUnlisten;
      });

      const { setupProgressListener } = useAppStore.getState();
      await setupProgressListener();

      // Simulate progress event
      act(() => {
        if (eventCallback) {
          eventCallback({
            payload: {
              completed_count: 2,
              total_count: 4,
              completed_step: 'git',
            },
          });
        }
      });

      const state = useAppStore.getState();
      expect(state.checkProgress).toBe(50); // 2/4 * 100
      expect(state.checkCompletedStep).toBe('git');
    });

    it('should handle progress events in rapid succession', async () => {
      let eventCallback: ((event: { payload: { completed_count: number; total_count: number; completed_step: string } }) => void) | null = null;

      mockListen.mockImplementationOnce(async (eventName, callback) => {
        if (eventName === 'env-check-progress') {
          eventCallback = callback as typeof eventCallback;
        }
        return mockUnlisten;
      });

      const { setupProgressListener } = useAppStore.getState();
      await setupProgressListener();

      // Simulate rapid progress events
      act(() => {
        if (eventCallback) {
          eventCallback({ payload: { completed_count: 1, total_count: 4, completed_step: 'nodejs' } });
          eventCallback({ payload: { completed_count: 2, total_count: 4, completed_step: 'git' } });
          eventCallback({ payload: { completed_count: 3, total_count: 4, completed_step: 'openclaw' } });
          eventCallback({ payload: { completed_count: 4, total_count: 4, completed_step: 'gateway' } });
        }
      });

      const state = useAppStore.getState();
      expect(state.checkProgress).toBe(100);
      expect(state.checkCompletedStep).toBe('gateway');
    });
  });
});
