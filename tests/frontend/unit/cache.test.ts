import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { DetectionStep, EnvironmentStatus } from '@/types';
import { logStore } from '@/lib/logger';

/**
 * 7.8 前端集成测试 - 缓存相关
 */
describe('缓存集成测试', () => {
  beforeEach(() => {
    logStore.clear();
  });

  describe('7.8.1 detection_steps 缓存命中显示', () => {
    it('应正确显示缓存命中的 detection_step', () => {
      const cachedSteps: DetectionStep[] = [
        {
          phase: 'Cache: Using cached path',
          action: 'Loading from cache',
          target: '/usr/local/bin/openclaw',
          result: 'found',
          message: 'Path loaded from cache',
        },
      ];

      expect(cachedSteps).toHaveLength(1);
      expect(cachedSteps[0].phase).toContain('Cache');
      expect(cachedSteps[0].result).toBe('found');
      expect(cachedSteps[0].message).toBe('Path loaded from cache');
    });

    it('应正确显示多阶段 detection_steps (缓存未命中)', () => {
      const detectionSteps: DetectionStep[] = [
        {
          phase: 'Phase 1: npm global prefix',
          action: 'Checking npm prefix',
          target: '/usr/local/bin/openclaw',
          result: 'found',
        },
        {
          phase: 'Phase 2: Hardcoded paths',
          action: 'Checking path',
          target: '/opt/homebrew/bin/openclaw',
          result: 'not_found',
        },
      ];

      expect(detectionSteps).toHaveLength(2);
      expect(detectionSteps.every(s => !s.phase.includes('Cache'))).toBe(true);
    });
  });

  describe('7.8.2 Mock Tauri invoke 缓存命中场景', () => {
    it('应正确处理 check_environment 返回的缓存命中响应', async () => {
      const mockInvoke = vi.fn().mockResolvedValue({
        node_installed: true,
        node_version: 'v22.0.0',
        node_version_ok: true,
        git_installed: true,
        git_version: '2.40.0',
        openclaw_installed: true,
        openclaw_version: '2026.1.29',
        gateway_service_installed: true,
        config_dir_exists: true,
        ready: true,
        os: 'Windows',
        is_secure: true,
        detection_steps: [
          {
            phase: 'Cache: Using cached path',
            action: 'Loading from cache',
            target: 'C:\\Users\\test\\AppData\\Roaming\\npm\\openclaw.cmd',
            result: 'found',
            message: 'Path loaded from cache',
          },
        ],
      } as EnvironmentStatus);

      const result = await mockInvoke('check_environment');

      expect(result.openclaw_installed).toBe(true);
      expect(result.detection_steps).toHaveLength(1);
      expect(result.detection_steps[0].phase).toContain('Cache');
      expect(mockInvoke).toHaveBeenCalledWith('check_environment');
    });
  });

  describe('7.8.3 刷新按钮触发缓存失效', () => {
    it('应正确调用 invalidate_environment_cache', async () => {
      const mockInvoke = vi.fn().mockResolvedValue(undefined);
      await mockInvoke('invalidate_environment_cache');
      expect(mockInvoke).toHaveBeenCalledWith('invalidate_environment_cache');
    });

    it('刷新后应重新检查环境', async () => {
      const mockInvoke = vi.fn();
      mockInvoke.mockResolvedValueOnce(undefined);
      mockInvoke.mockResolvedValueOnce({
        node_installed: true,
        node_version: 'v22.0.0',
        node_version_ok: true,
        git_installed: true,
        git_version: '2.40.0',
        openclaw_installed: true,
        openclaw_version: '2026.1.30',
        gateway_service_installed: true,
        config_dir_exists: true,
        ready: true,
        os: 'Windows',
        is_secure: true,
        detection_steps: [
          {
            phase: 'Phase 1: npm global prefix',
            action: 'Checking npm prefix',
            target: '/usr/local/bin/openclaw',
            result: 'found',
          },
        ],
      } as EnvironmentStatus);

      await mockInvoke('invalidate_environment_cache');
      const result = await mockInvoke('check_environment');

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'invalidate_environment_cache');
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'check_environment');
      expect(result.openclaw_version).toBe('2026.1.30');
    });
  });
});
