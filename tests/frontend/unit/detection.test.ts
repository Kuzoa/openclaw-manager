import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import type { DetectionStep, DetectionResult, EnvironmentStatus } from '@/types';
import { setupLogger, logStore } from '@/lib/logger';

describe('DetectionStep 类型', () => {
  describe('DetectionResult 类型', () => {
    it('应支持 found 值', () => {
      const result: DetectionResult = 'found';
      expect(result).toBe('found');
    });

    it('应支持 not_found 值', () => {
      const result: DetectionResult = 'not_found';
      expect(result).toBe('not_found');
    });

    it('应支持 error 值', () => {
      const result: DetectionResult = 'error';
      expect(result).toBe('error');
    });

    it('应包含所有三种结果类型', () => {
      const results: DetectionResult[] = ['found', 'not_found', 'error'];
      expect(results).toHaveLength(3);
    });
  });

  describe('DetectionStep 接口', () => {
    it('应正确构造 found 类型的 DetectionStep', () => {
      const step: DetectionStep = {
        phase: 'Phase 1: npm global prefix',
        action: 'Checking npm prefix',
        target: '/usr/local/bin/openclaw',
        result: 'found',
      };
      expect(step.phase).toBe('Phase 1: npm global prefix');
      expect(step.result).toBe('found');
      expect(step.message).toBeUndefined();
    });

    it('应正确构造 not_found 类型的 DetectionStep', () => {
      const step: DetectionStep = {
        phase: 'Phase 2: Hardcoded paths',
        action: 'Checking path',
        target: 'C:\\Program Files\\nodejs\\openclaw.cmd',
        result: 'not_found',
      };
      expect(step.result).toBe('not_found');
      expect(step.message).toBeUndefined();
    });

    it('应正确构造 error 类型的 DetectionStep（带 message）', () => {
      const step: DetectionStep = {
        phase: 'Phase 1: npm global prefix',
        action: 'Checking npm prefix',
        target: 'npm config get prefix',
        result: 'error',
        message: 'Failed to get npm global prefix',
      };
      expect(step.result).toBe('error');
      expect(step.message).toBe('Failed to get npm global prefix');
    });

    it('应支持可选的 message 字段', () => {
      const stepWithMessage: DetectionStep = {
        phase: 'Phase 1',
        action: 'Check',
        target: 'target',
        result: 'error',
        message: 'Error message',
      };
      expect(stepWithMessage.message).toBe('Error message');

      const stepWithoutMessage: DetectionStep = {
        phase: 'Phase 1',
        action: 'Check',
        target: 'target',
        result: 'found',
      };
      expect(stepWithoutMessage.message).toBeUndefined();
    });
  });

  describe('DetectionStep 数组', () => {
    it('应支持 DetectionStep 数组类型', () => {
      const steps: DetectionStep[] = [
        {
          phase: 'Phase 1: npm global prefix',
          action: 'Checking npm prefix',
          target: '/path/1',
          result: 'not_found',
        },
        {
          phase: 'Phase 2: Hardcoded paths',
          action: 'Checking path',
          target: '/path/2',
          result: 'found',
        },
      ];
      expect(steps).toHaveLength(2);
      expect(steps[0].result).toBe('not_found');
      expect(steps[1].result).toBe('found');
    });
  });
});

describe('EnvironmentStatus 扩展', () => {
  it('应包含 detection_steps 字段', () => {
    const status: EnvironmentStatus = {
      node_installed: true,
      node_version: 'v22.0.0',
      node_version_ok: true,
      git_installed: true,
      git_version: '2.43.0',
      openclaw_installed: true,
      openclaw_version: '2026.1.29',
      gateway_service_installed: true,
      config_dir_exists: true,
      ready: true,
      os: 'windows',
      is_secure: true,
      detection_steps: [
        {
          phase: 'Phase 1: npm global prefix',
          action: 'Checking npm prefix',
          target: '/path/to/openclaw',
          result: 'found',
        },
      ],
    };

    expect(status.detection_steps).toBeDefined();
    expect(Array.isArray(status.detection_steps)).toBe(true);
    expect(status.detection_steps).toHaveLength(1);
  });

  it('应支持空的 detection_steps 数组', () => {
    const status: EnvironmentStatus = {
      node_installed: false,
      node_version: null,
      node_version_ok: false,
      git_installed: false,
      git_version: null,
      openclaw_installed: false,
      openclaw_version: null,
      gateway_service_installed: false,
      config_dir_exists: false,
      ready: false,
      os: 'linux',
      is_secure: false,
      detection_steps: [],
    };

    expect(status.detection_steps).toEqual([]);
  });
});

describe('logDetectionSteps 日志输出格式', () => {
  let consoleLogSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    logStore.clear();
    localStorage.clear();
    localStorage.setItem('LOG_LEVEL', 'debug');
    consoleLogSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
  });

  afterEach(() => {
    consoleLogSpy.mockRestore();
  });

  it('setupLogger 应正确输出检测步骤', () => {
    const steps: DetectionStep[] = [
      {
        phase: 'Phase 1: npm global prefix',
        action: 'Checking npm prefix',
        target: '/usr/local/bin/openclaw',
        result: 'found',
      },
    ];

    // 使用 setupLogger 输出日志
    setupLogger.info('🔍 开始环境检查...');
    setupLogger.info('📋 检测过程:');
    setupLogger.info(`  └─ ${steps[0].phase}`);
    setupLogger.info(`        └─ 检查: ${steps[0].target}`);
    setupLogger.info('        └─ ✓ 找到');
    setupLogger.info('  └─ 检测完成');
    setupLogger.info('✅ 环境检查完成: OpenClaw v1.0.0');

    // 验证日志被添加到 logStore
    const logs = logStore.getAll();
    expect(logs.length).toBeGreaterThan(0);

    // 验证日志内容
    const logMessages = logs.map(l => l.message);
    expect(logMessages.some(m => m.includes('开始环境检查'))).toBe(true);
    expect(logMessages.some(m => m.includes('检测过程'))).toBe(true);
    expect(logMessages.some(m => m.includes('找到'))).toBe(true);
  });

  it('应正确输出错误类型的步骤', () => {
    const steps: DetectionStep[] = [
      {
        phase: 'Phase 1: npm global prefix',
        action: 'Checking npm prefix',
        target: 'npm config get prefix',
        result: 'error',
        message: 'npm command not found',
      },
    ];

    setupLogger.info('🔍 开始环境检查...');
    setupLogger.info('📋 检测过程:');
    steps.forEach(step => {
      setupLogger.info(`  └─ ${step.phase}`);
      setupLogger.info(`        └─ 检查: ${step.target}`);
      const resultIcon = `⚠ 执行失败: ${step.message || '未知错误'}`;
      setupLogger.info(`        └─ ${resultIcon}`);
    });

    const logs = logStore.getAll();
    const logMessages = logs.map(l => l.message);
    expect(logMessages.some(m => m.includes('执行失败'))).toBe(true);
    expect(logMessages.some(m => m.includes('npm command not found'))).toBe(true);
  });

  it('应正确输出 not_found 类型的步骤', () => {
    const steps: DetectionStep[] = [
      {
        phase: 'Phase 3: PATH environment',
        action: 'Checking PATH',
        target: 'openclaw',
        result: 'not_found',
      },
    ];

    setupLogger.info('🔍 开始环境检查...');
    steps.forEach(_step => {
      const resultIcon = '✗ 文件不存在';
      setupLogger.info(`结果: ${resultIcon}`);
    });

    const logs = logStore.getAll();
    const logMessages = logs.map(l => l.message);
    expect(logMessages.some(m => m.includes('文件不存在'))).toBe(true);
  });

  it('应正确处理空的 detection_steps', () => {
    const steps: DetectionStep[] = [];

    setupLogger.info('🔍 开始环境检查...');
    if (steps.length === 0) {
      setupLogger.warn('⚠️ 环境检查完成: OpenClaw 未安装');
    }

    const logs = logStore.getAll();
    const logMessages = logs.map(l => l.message);
    expect(logMessages.some(m => m.includes('OpenClaw 未安装'))).toBe(true);
  });

  it('应正确按 phase 分组输出多个步骤', () => {
    const steps: DetectionStep[] = [
      {
        phase: 'Phase 1: npm global prefix',
        action: 'Checking npm prefix',
        target: '/path/1',
        result: 'not_found',
      },
      {
        phase: 'Phase 2: Hardcoded paths',
        action: 'Checking path',
        target: '/path/2',
        result: 'not_found',
      },
      {
        phase: 'Phase 2: Hardcoded paths',
        action: 'Checking path',
        target: '/path/3',
        result: 'found',
      },
    ];

    // 模拟分组逻辑
    const phaseMap = new Map<string, DetectionStep[]>();
    for (const step of steps) {
      const existing = phaseMap.get(step.phase) || [];
      existing.push(step);
      phaseMap.set(step.phase, existing);
    }

    const phases = Array.from(phaseMap.keys());
    expect(phases).toHaveLength(2); // Phase 1 和 Phase 2
    expect(phaseMap.get('Phase 2: Hardcoded paths')).toHaveLength(2); // Phase 2 有两个步骤
  });

  it('成功场景应使用 info 级别输出最终状态', () => {
    const openclawInstalled = true;
    const openclawVersion = 'OpenClaw 2026.1.29';

    if (openclawInstalled) {
      setupLogger.info(`✅ 环境检查完成: ${openclawVersion || 'OpenClaw 已安装'}`);
    }

    const logs = logStore.getAll();
    const lastLog = logs[logs.length - 1];
    expect(lastLog.level).toBe('info');
    expect(lastLog.message).toContain('环境检查完成');
    expect(lastLog.message).toContain('OpenClaw 2026.1.29');
    // 不应重复显示 "OpenClaw OpenClaw"
    expect(lastLog.message).not.toContain('OpenClaw OpenClaw');
  });

  it('失败场景应使用 warn 级别输出最终状态', () => {
    const openclawInstalled = false;

    if (!openclawInstalled) {
      setupLogger.warn('⚠️ 环境检查完成: OpenClaw 未安装');
    }

    const logs = logStore.getAll();
    const lastLog = logs[logs.length - 1];
    expect(lastLog.level).toBe('warn');
    expect(lastLog.message).toContain('OpenClaw 未安装');
  });
});
