import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { ServiceStatus, SystemInfo } from '../lib/tauri';
import type { EnvironmentStatus, DetectionStep, CheckProgress } from '../types';
import { setupLogger } from '../lib/logger';
import { isTauri } from '../lib/tauri';
import i18n from '../i18n';

/**
 * Log detection steps to the Setup logger
 */
function logDetectionSteps(steps: DetectionStep[], openclawInstalled: boolean, openclawVersion: string | null): void {
  const t = i18n.getFixedT(null, 'logs');
  
  setupLogger.info(`🔍 ${t('detection.start')}`);
  
  if (steps.length === 0) {
    // No detection steps - just log final status
    if (openclawInstalled) {
      setupLogger.info(`✅ ${t('detection.completeInstalled', { version: openclawVersion || t('detection.installed') })}`);
    } else {
      setupLogger.warn(`⚠️ ${t('detection.completeNotInstalled')}`);
    }
    return;
  }
  
  setupLogger.info(`📋 ${t('detection.process')}:`);
  
  // Group steps by phase
  const phaseMap = new Map<string, DetectionStep[]>();
  for (const step of steps) {
    const existing = phaseMap.get(step.phase) || [];
    existing.push(step);
    phaseMap.set(step.phase, existing);
  }
  
  const phases = Array.from(phaseMap.keys());
  phases.forEach((phase, phaseIndex) => {
    const isLastPhase = phaseIndex === phases.length - 1;
    const phasePrefix = isLastPhase ? '  └─' : '  ├─';
    setupLogger.info(`${phasePrefix} ${phase}`);
    
    const phaseSteps = phaseMap.get(phase)!;
    phaseSteps.forEach((step) => {
      const stepPrefix = isLastPhase ? '        └─' : '  │     └─';
      setupLogger.info(`${stepPrefix} ${t('detection.check')}: ${step.target}`);
      
      let resultIcon: string;
      if (step.result === 'found') {
        resultIcon = `✓ ${t('detection.found')}`;
      } else if (step.result === 'error') {
        resultIcon = `⚠ ${t('detection.error', { message: step.message || t('detection.unknownError') })}`;
      } else {
        resultIcon = `✗ ${t('detection.notFound')}`;
      }
      setupLogger.info(`${stepPrefix} ${resultIcon}`);
    });
  });
  
  setupLogger.info(`  └─ ${t('detection.done')}`);
  
  // Log final status
  if (openclawInstalled) {
    setupLogger.info(`✅ ${t('detection.completeInstalled', { version: openclawVersion || t('detection.installed') })}`);
  } else {
    setupLogger.warn(`⚠️ ${t('detection.completeNotInstalled')}`);
  }
}

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

  // Check progress state
  /** Check progress percentage (0-100) */
  checkProgress: number;
  /** The step that just completed (for UI display) */
  checkCompletedStep: string | null;
  setupProgressListener: () => Promise<void>;
  cleanupProgressListener: () => void;

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

// Progress listener reference (singleton pattern for React Strict Mode compatibility)
let progressListenerRef: UnlistenFn | null = null;

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

  // Check progress state
  checkProgress: 0,
  checkCompletedStep: null,

  checkEnvironment: async () => {
    // If already checking, return the pending promise to prevent duplicate calls
    if (get().isCheckingEnvironment && pendingEnvironmentCheck) {
      return pendingEnvironmentCheck;
    }

    // If environment already loaded, no need to check again
    if (get().environment) {
      return;
    }

    // Reset progress state at the start
    set({ isCheckingEnvironment: true, environmentError: null, checkProgress: 0, checkCompletedStep: null });

    pendingEnvironmentCheck = (async () => {
      try {
        const status = await invoke<EnvironmentStatus>('check_environment');
        // Log detection steps
        logDetectionSteps(status.detection_steps || [], status.openclaw_installed, status.openclaw_version);
        set({ environment: status, isCheckingEnvironment: false, checkProgress: 0, checkCompletedStep: null });
      } catch (error) {
        set({
          environmentError: `Failed to check environment: ${error}`,
          isCheckingEnvironment: false,
          checkProgress: 0,
          checkCompletedStep: null,
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

    // Reset progress state at the start
    set({ isCheckingEnvironment: true, environmentError: null, checkProgress: 0, checkCompletedStep: null });

    pendingEnvironmentCheck = (async () => {
      try {
        // First invalidate cache, then check environment
        await invoke('invalidate_environment_cache');
        const status = await invoke<EnvironmentStatus>('check_environment');
        // Log detection steps
        logDetectionSteps(status.detection_steps || [], status.openclaw_installed, status.openclaw_version);
        set({ environment: status, isCheckingEnvironment: false, checkProgress: 0, checkCompletedStep: null });
      } catch (error) {
        set({
          environmentError: `Failed to refresh environment: ${error}`,
          isCheckingEnvironment: false,
          checkProgress: 0,
          checkCompletedStep: null,
        });
      } finally {
        pendingEnvironmentCheck = null;
      }
    })();

    return pendingEnvironmentCheck;
  },

  setupProgressListener: async () => {
    // Singleton pattern: prevent duplicate registration
    if (progressListenerRef) return;

    // Guard: Only register listener in Tauri environment
    if (!isTauri()) return;

    progressListenerRef = await listen<CheckProgress>('env-check-progress', (event) => {
      const { completed_count, total_count, completed_step } = event.payload;
      set({
        checkProgress: Math.round((completed_count / total_count) * 100),
        checkCompletedStep: completed_step,
      });
    });
  },

  cleanupProgressListener: () => {
    if (progressListenerRef) {
      progressListenerRef();
      progressListenerRef = null;
      set({ checkProgress: 0, checkCompletedStep: null });
    }
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
