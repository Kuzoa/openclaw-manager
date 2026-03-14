import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { act } from '@testing-library/react';
import { useAppStore } from '@/stores/appStore';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const { invoke } = await import('@tauri-apps/api/core');
const mockInvoke = vi.mocked(invoke);

describe('AppStore', () => {
  beforeEach(() => {
    // Reset store to initial state
    useAppStore.setState({
      serviceStatus: null,
      systemInfo: null,
      environment: null,
      isCheckingEnvironment: false,
      environmentError: null,
      loading: false,
      notifications: [],
    });
    
    // Reset mock
    mockInvoke.mockReset();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Initial State', () => {
    it('should have correct initial values', () => {
      const state = useAppStore.getState();
      
      expect(state.serviceStatus).toBeNull();
      expect(state.systemInfo).toBeNull();
      expect(state.environment).toBeNull();
      expect(state.isCheckingEnvironment).toBe(false);
      expect(state.environmentError).toBeNull();
      expect(state.loading).toBe(false);
      expect(state.notifications).toEqual([]);
    });
  });

  describe('setServiceStatus', () => {
    it('should update service status', () => {
      const { setServiceStatus } = useAppStore.getState();
      
      const status = { running: true, pid: 1234, version: '1.0.0' };
      
      act(() => {
        setServiceStatus(status);
      });

      expect(useAppStore.getState().serviceStatus).toEqual(status);
    });

    it('should allow setting null', () => {
      const { setServiceStatus } = useAppStore.getState();
      
      act(() => {
        setServiceStatus({ running: true, pid: 1234, version: '1.0.0' });
        setServiceStatus(null);
      });

      expect(useAppStore.getState().serviceStatus).toBeNull();
    });
  });

  describe('setSystemInfo', () => {
    it('should update system info', () => {
      const { setSystemInfo } = useAppStore.getState();
      
      const info = {
        os: 'Windows',
        version: '10.0.19045',
        arch: 'x64',
        hostname: 'test-host',
      };
      
      act(() => {
        setSystemInfo(info);
      });

      expect(useAppStore.getState().systemInfo).toEqual(info);
    });
  });

  describe('setLoading', () => {
    it('should update loading state', () => {
      const { setLoading } = useAppStore.getState();
      
      act(() => {
        setLoading(true);
      });

      expect(useAppStore.getState().loading).toBe(true);
      
      act(() => {
        setLoading(false);
      });

      expect(useAppStore.getState().loading).toBe(false);
    });
  });

  describe('Notifications', () => {
    it('should add a notification with generated id', () => {
      const { addNotification } = useAppStore.getState();
      
      act(() => {
        addNotification({
          type: 'success',
          title: 'Test Title',
          message: 'Test Message',
        });
      });

      const notifications = useAppStore.getState().notifications;
      expect(notifications).toHaveLength(1);
      expect(notifications[0].type).toBe('success');
      expect(notifications[0].title).toBe('Test Title');
      expect(notifications[0].message).toBe('Test Message');
      expect(notifications[0].id).toBeDefined();
    });

    it('should add multiple notifications', () => {
      const { addNotification } = useAppStore.getState();
      
      act(() => {
        addNotification({ type: 'success', title: 'First' });
        addNotification({ type: 'error', title: 'Second' });
      });

      const notifications = useAppStore.getState().notifications;
      expect(notifications).toHaveLength(2);
    });

    it('should remove a notification by id', () => {
      const { addNotification, removeNotification } = useAppStore.getState();
      
      act(() => {
        addNotification({ type: 'info', title: 'Test' });
      });

      const id = useAppStore.getState().notifications[0].id;
      
      act(() => {
        removeNotification(id);
      });

      expect(useAppStore.getState().notifications).toHaveLength(0);
    });

    it('should not affect other notifications when removing', async () => {
      const { addNotification, removeNotification } = useAppStore.getState();
      
      act(() => {
        addNotification({ type: 'info', title: 'First' });
      });
      
      // Small delay to ensure different IDs (Date.now() based)
      await new Promise(resolve => setTimeout(resolve, 2));
      
      act(() => {
        addNotification({ type: 'success', title: 'Second' });
      });

      const notifications = useAppStore.getState().notifications;
      expect(notifications).toHaveLength(2);
      
      const firstId = notifications[0].id;
      const secondId = notifications[1].id;
      
      // Verify IDs are different
      expect(firstId).not.toBe(secondId);
      
      act(() => {
        removeNotification(firstId);
      });

      const remaining = useAppStore.getState().notifications;
      expect(remaining).toHaveLength(1);
      expect(remaining[0].title).toBe('Second');
    });
  });
});
