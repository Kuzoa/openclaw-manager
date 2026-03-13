import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { ServiceStatus, SystemInfo } from '../lib/tauri';
import type { EnvironmentStatus } from '../types';

interface AppState {
  // Service status
  serviceStatus: ServiceStatus | null;
  setServiceStatus: (status: ServiceStatus | null) => void;

  // System information
  systemInfo: SystemInfo | null;
  setSystemInfo: (info: SystemInfo | null) => void;

  // Environment status (unified)
  environment: EnvironmentStatus | null;
  isCheckingEnvironment: boolean;
  environmentError: string | null;
  checkEnvironment: () => Promise<void>;
  refreshEnvironment: () => Promise<void>;

  // UI state
  loading: boolean;
  setLoading: (loading: boolean) => void;

  // Notifications
  notifications: Notification[];
  addNotification: (notification: Omit<Notification, 'id'>) => void;
  removeNotification: (id: string) => void;
}

interface Notification {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message?: string;
}

// Store pending promise to prevent concurrent checks
let pendingEnvironmentCheck: Promise<void> | null = null;

export const useAppStore = create<AppState>((set, get) => ({
  // Service status
  serviceStatus: null,
  setServiceStatus: (status) => set({ serviceStatus: status }),

  // System information
  systemInfo: null,
  setSystemInfo: (info) => set({ systemInfo: info }),

  // Environment status
  environment: null,
  isCheckingEnvironment: false,
  environmentError: null,

  checkEnvironment: async () => {
    // If already checking, return the pending promise to prevent duplicate calls
    if (get().isCheckingEnvironment && pendingEnvironmentCheck) {
      return pendingEnvironmentCheck;
    }

    // If environment already loaded, no need to check again
    if (get().environment) {
      return;
    }

    set({ isCheckingEnvironment: true, environmentError: null });

    pendingEnvironmentCheck = (async () => {
      try {
        const status = await invoke<EnvironmentStatus>('check_environment');
        set({ environment: status, isCheckingEnvironment: false });
      } catch (error) {
        set({
          environmentError: `Failed to check environment: ${error}`,
          isCheckingEnvironment: false,
        });
      } finally {
        pendingEnvironmentCheck = null;
      }
    })();

    return pendingEnvironmentCheck;
  },

  refreshEnvironment: async () => {
    // If already checking, return the pending promise
    if (get().isCheckingEnvironment && pendingEnvironmentCheck) {
      return pendingEnvironmentCheck;
    }

    set({ isCheckingEnvironment: true, environmentError: null });

    pendingEnvironmentCheck = (async () => {
      try {
        // First invalidate cache, then check environment
        await invoke('invalidate_environment_cache');
        const status = await invoke<EnvironmentStatus>('check_environment');
        set({ environment: status, isCheckingEnvironment: false });
      } catch (error) {
        set({
          environmentError: `Failed to refresh environment: ${error}`,
          isCheckingEnvironment: false,
        });
      } finally {
        pendingEnvironmentCheck = null;
      }
    })();

    return pendingEnvironmentCheck;
  },

  // UI state
  loading: false,
  setLoading: (loading) => set({ loading }),

  // Notifications
  notifications: [],
  addNotification: (notification) =>
    set((state) => ({
      notifications: [
        ...state.notifications,
        { ...notification, id: Date.now().toString() },
      ],
    })),
  removeNotification: (id) =>
    set((state) => ({
      notifications: state.notifications.filter((n) => n.id !== id),
    })),
}));
