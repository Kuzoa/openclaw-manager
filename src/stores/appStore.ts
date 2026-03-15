import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { ServiceStatus, SystemInfo } from '../lib/tauri';
import type { EnvironmentStatus, DetectionStep } from '../types';
import { setupLogger } from '../lib/logger';

/**
 * Log detection steps to the Setup logger
 */
function logDetectionSteps(steps: DetectionStep[], openclawInstalled: boolean, openclawVersion: string | null): void {
  setupLogger.info('🔍 开始环境检查...');
  
  if (steps.length === 0) {
    // No detection steps - just log final status
    if (openclawInstalled) {
      setupLogger.info(`✅ 环境检查完成: ${openclawVersion || 'OpenClaw 已安装'}`);
    } else {
      setupLogger.warn('⚠️ 环境检查完成: OpenClaw 未安装');
    }
    return;
  }
  
  setupLogger.info('📋 检测过程:');
  
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
      setupLogger.info(`${stepPrefix} 检查: ${step.target}`);
      
      let resultIcon: string;
      if (step.result === 'found') {
        resultIcon = '✓ 找到';
      } else if (step.result === 'error') {
        resultIcon = `⚠ 执行失败: ${step.message || '未知错误'}`;
      } else {
        resultIcon = '✗ 文件不存在';
      }
      setupLogger.info(`${stepPrefix} ${resultIcon}`);
    });
  });
  
  setupLogger.info('  └─ 检测完成');
  
  // Log final status
  if (openclawInstalled) {
    setupLogger.info(`✅ 环境检查完成: ${openclawVersion || 'OpenClaw 已安装'}`);
  } else {
    setupLogger.warn('⚠️ 环境检查完成: OpenClaw 未安装');
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
        // Log detection steps
        logDetectionSteps(status.detection_steps || [], status.openclaw_installed, status.openclaw_version);
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
        // Log detection steps
        logDetectionSteps(status.detection_steps || [], status.openclaw_installed, status.openclaw_version);
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
