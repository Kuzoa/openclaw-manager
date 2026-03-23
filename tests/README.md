# OpenClaw Manager 测试指南

本文档介绍项目的测试体系架构、组织结构、运行方式及最佳实践。

## 测试体系架构

本项目采用**分层测试策略**，按前后端分离，后端进一步区分内嵌与分离式测试：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          测试体系架构                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  前端测试（分离式，位于 tests/frontend/）                                │
│  ├── 单元测试：tests/frontend/unit/*.test.ts                           │
│  └── 框架：Vitest + jsdom + @testing-library/react                     │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  后端测试（Rust）                                                        │
│  ├── 内嵌单元测试：src-tauri/src/**/*_tests.rs（与源码同目录）            │
│  │   └── 位置：src-tauri/src/utils/cache/cache_tests.rs                 │
│  │              src-tauri/src/utils/log_sanitizer_tests.rs              │
│  │              src-tauri/src/commands/service_tests.rs                 │
│  │              src-tauri/src/models/detection_tests.rs                 │
│  │                                                                      │
│  └── 分离式集成测试：src-tauri/tests/*.rs（独立 tests 目录）              │
│      └── 位置：src-tauri/tests/cache_integration_tests.rs               │
│                 src-tauri/tests/config_tests.rs                         │
│                 src-tauri/tests/detection_tests.rs                      │
│                 src-tauri/tests/service_tests.rs                        │
│                 src-tauri/tests/performance_tests.rs                    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

**注意：重复文件名说明**

部分测试文件在不同目录中同名（如 `service_tests.rs`、`detection_tests.rs`），这是 **Rust 惯例**：

| 文件 | 位置 | 访问范围 | 测试类型 |
|------|------|----------|----------|
| `service_tests.rs` | `src/commands/` | 私有函数 | 单元测试 |
| `service_tests.rs` | `tests/` | 公共 API | 集成测试 |
| `detection_tests.rs` | `src/models/` | 私有函数 | 单元测试 |
| `detection_tests.rs` | `tests/` | 公共 API | 集成测试 |

**关键设计决策：**

| 类型 | 位置 | 原因 |
|------|------|------|
| 前端测试 | `tests/frontend/` | 与源码分离，避免打包干扰，符合 Vitest 社区惯例 |
| 后端单元测试 | `src-tauri/src/**/*_tests.rs` | Rust 惯例，测试与源码同目录，可访问私有函数 |
| 后端集成测试 | `src-tauri/tests/` | Rust 惯例，独立目录，只能访问公开 API |

## 目录结构

```
tests/
├── frontend/                    # 前端测试
│   ├── vitest.config.ts         # Vitest 配置文件
│   ├── setup.ts                 # 测试环境初始化（localStorage/window mock）
│   ├── tsconfig.json            # TypeScript 配置
│   └── unit/                    # 单元测试
│       ├── cache.test.ts        # 缓存模块测试
│       ├── detection.test.ts    # 检测步骤测试
│       ├── i18n.test.ts         # 国际化测试
│       ├── logger.test.ts       # Logger 模块测试
│       └── store.test.ts        # Zustand Store 测试
│
└── README.md                    # 本文档

src-tauri/
├── src/                         # 源码 + 内嵌单元测试
│   ├── commands/
│   │   └── service_tests.rs     # 服务命令单元测试
│   ├── models/
│   │   └── detection_tests.rs   # 检测模型单元测试
│   └── utils/
│       ├── cache/
│       │   └── cache_tests.rs   # 缓存单元测试
│       └── log_sanitizer_tests.rs # 日志脱敏单元测试
│
└── tests/                       # Rust 集成测试
    ├── cache_integration_tests.rs  # 缓存集成测试
    ├── config_tests.rs             # 配置相关测试
    ├── detection_tests.rs          # 检测步骤集成测试
    ├── performance_tests.rs        # 性能测试
    ├── README_PERFORMANCE_TESTS.md # 性能测试文档
    └── service_tests.rs            # 服务状态集成测试
```

## 测试框架

### 前端测试

| 技术 | 版本 | 用途 |
|------|------|------|
| Vitest | ^4.1.0 | 测试运行器 |
| jsdom | - | DOM 环境模拟 |
| @testing-library/react | - | React 组件测试工具 |

### 后端测试

| 技术 | 版本 | 用途 |
|------|------|------|
| Cargo test | Rust 内置 | 单元测试与集成测试 |
| tempfile | ^3 | 临时文件/目录管理 |

## 快速开始

### 运行所有前端测试

```bash
npm test
```

### 运行所有后端测试

```bash
cd src-tauri && cargo test
```

### 运行完整测试套件

```bash
npm test && cd src-tauri && cargo test
```

## 前端测试

### 环境配置

前端测试环境由以下文件配置：

**vitest.config.ts** - 主配置文件
```typescript
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, '../../src'),
    },
  },
  test: {
    environment: 'jsdom',
    include: ['tests/frontend/**/*.test.ts'],
    setupFiles: [path.resolve(__dirname, './setup.ts')],
  },
});
```

**setup.ts** - 测试环境初始化
- 提供 `localStorage` mock
- 提供 `window` 对象 mock
- 设置默认日志级别

### 运行命令

```bash
# 运行测试（单次执行）
npm test

# 等同于
npm run test:run

# 监听模式（开发时使用）
npx vitest --config tests/frontend/vitest.config.ts
```

### 测试文件命名规范

| 模式 | 说明 |
|------|------|
| `*.test.ts` | 单元测试文件 |
| `*.spec.ts` | 规格测试文件（可选） |

### 现有测试模块

#### logger.test.ts（16 个测试）

测试 `src/lib/logger.ts` 模块：

- **createLogger** - 工厂函数返回正确实例
- **Logger methods** - debug/info/warn/error 方法调用正确的 console 方法
- **logStore** - 日志存储、订阅、自动裁剪
- **日志级别过滤** - 不同级别下的日志输出控制

#### store.test.ts（9 个测试）

测试 `src/stores/appStore.ts` Zustand store：

- **初始状态** - 验证默认值
- **setServiceStatus** - 服务状态更新
- **setSystemInfo** - 系统信息更新
- **setLoading** - 加载状态控制
- **Notifications** - 通知的增删管理

### 编写新测试

```typescript
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

describe('模块名称', () => {
  beforeEach(() => {
    // 测试前准备
  });

  afterEach(() => {
    // 测试后清理
  });

  it('应该正确处理某种情况', () => {
    // Arrange
    const input = 'test';
    
    // Act
    const result = someFunction(input);
    
    // Assert
    expect(result).toBe('expected');
  });
});
```

### Mock 策略

**Tauri API Mock**
```typescript
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const { invoke } = await import('@tauri-apps/api/core');
const mockInvoke = vi.mocked(invoke);
```

**Console 方法 Spy**
```typescript
let consoleLogSpy: ReturnType<typeof vi.spyOn>;

beforeEach(() => {
  consoleLogSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
});

afterEach(() => {
  consoleLogSpy.mockRestore();
});
```

## 后端测试

### 环境配置

后端测试在 `src-tauri/Cargo.toml` 中配置：

```toml
[lib]
name = "openclaw_manager"
path = "src/lib.rs"

[dev-dependencies]
tempfile = "3"
```

### 运行命令

```bash
cd src-tauri

# 运行所有测试
cargo test

# 运行特定测试文件
cargo test --test config_tests
cargo test --test service_tests

# 显示测试输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_name
```

### 测试类型

#### 单元测试（内嵌）

位于源文件中的 `#[cfg(test)]` 模块，测试与源码同目录，可访问私有函数：

```
src-tauri/src/
├── commands/
│   └── service_tests.rs      # 服务命令测试
├── models/
│   └── detection_tests.rs    # 检测模型测试
└── utils/
    ├── cache/
    │   └── cache_tests.rs    # 缓存模块测试
    └── log_sanitizer_tests.rs # 日志脱敏测试
```

现有单元测试：

| 文件 | 测试数 | 测试内容 |
|------|--------|----------|
| `cache_tests.rs` | 6+ | 缓存读写、并发、失效、TTL 验证 |
| `log_sanitizer_tests.rs` | 5 | 敏感信息脱敏 |
| `service_tests.rs` | 8 | 服务状态缓存、状态变更检测 |
| `detection_tests.rs` | 10+ | DetectionResult/DetectionStep 序列化 |

#### 集成测试

位于 `src-tauri/tests/` 目录，只能访问公开 API：

| 文件 | 测试数 | 测试内容 |
|------|--------|----------|
| `cache_integration_tests.rs` | 3 | 缓存生命周期、文件持久化 |
| `config_tests.rs` | 10 | 文件工具、环境变量、日志脱敏 |
| `detection_tests.rs` | 5 | 检测步骤结构、EnvironmentStatus 字段 |
| `service_tests.rs` | 9 | 服务状态序列化、平台检测、端口检查 |
| `performance_tests.rs` | 6 | 缓存 I/O 性能、环境检测性能 |

**性能测试说明**

`performance_tests.rs` 分为两类：
- **模拟测试**（默认运行）：CI 友好，无需真实环境
- **真实测试**（需 `--ignored`）：测量实际性能，需要本地环境

```bash
# 运行模拟性能测试
cargo test --test performance_tests

# 运行真实性能测试（本地环境）
cargo test --test performance_tests -- --ignored --test-threads=1
```

详见 `src-tauri/tests/README_PERFORMANCE_TESTS.md`。

### 编写新测试

**单元测试（内嵌）**
```rust
// src/utils/my_module.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        let result = my_function();
        assert_eq!(result, expected);
    }
}
```

**集成测试**
```rust
// src-tauri/tests/my_test.rs
use tempfile::TempDir;

#[test]
fn test_with_temp_dir() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    
    // 测试逻辑...
}
```

## 测试覆盖范围

### 当前覆盖

**前端**

| 模块 | 测试类型 | 覆盖程度 |
|------|----------|----------|
| `src/lib/logger.ts` | 单元 | 完整 |
| `src/stores/appStore.ts` | 单元 | 部分（状态管理） |
| `src/i18n/` | 单元 | 基本 |
| 缓存类型定义 | 单元 | 完整 |
| 检测步骤类型 | 单元 | 完整 |

**后端**

| 模块 | 测试类型 | 覆盖程度 |
|------|----------|----------|
| `src-tauri/utils/cache.rs` | 单元+集成+性能 | 完整 |
| `src-tauri/utils/log_sanitizer.rs` | 单元+集成 | 完整 |
| `src-tauri/utils/file.rs` | 集成 | 完整 |
| `src-tauri/utils/platform.rs` | 集成 | 基本 |
| `src-tauri/commands/service.rs` | 单元 | 完整 |
| `src-tauri/models/detection.rs` | 单元 | 完整 |
| `src-tauri/models/status.rs` | 集成 | 完整 |

### 暂未覆盖

- UI 组件测试
- E2E 端到端测试
- Tauri IPC 命令测试
- 进程管理测试（需系统调用 mock）

## 最佳实践

### 前端测试

1. **隔离测试** - 每个测试应独立运行，不依赖其他测试的副作用
2. **Mock 外部依赖** - 使用 `vi.mock()` 模拟 Tauri API、fetch 等
3. **清理副作用** - 在 `afterEach` 中恢复 mock 和清理状态
4. **测试用户行为** - 关注组件如何响应用户交互

### 后端测试

1. **使用临时目录** - 文件操作测试使用 `tempfile` 避免污染
2. **测试错误路径** - 不仅测试成功场景，也要测试错误处理
3. **避免平台依赖** - 测试应跨平台兼容（Windows/macOS/Linux）
4. **合理组织** - 相关测试放在同一模块，使用 `mod` 分组

### 通用原则

1. **AAA 模式** - Arrange（准备）、Act（执行）、Assert（断言）
2. **命名清晰** - 测试名称应描述预期行为
3. **单一职责** - 每个测试只验证一个行为
4. **快速反馈** - 测试应快速执行，便于频繁运行

## CI/CD 集成

测试命令已配置正确的退出码，可直接集成到 CI/CD 流程：

```yaml
# GitHub Actions 示例
- name: Run Frontend Tests
  run: npm test

- name: Run Backend Tests
  run: cd src-tauri && cargo test
```

## 常见问题

### Q: 前端测试找不到模块？

确保路径别名正确配置在 `vitest.config.ts` 中：
```typescript
resolve: {
  alias: {
    '@': path.resolve(__dirname, '../../src'),
  },
}
```

### Q: Rust 测试编译失败？

检查 `lib.rs` 是否正确导出需要测试的模块：
```rust
pub use utils::{file, log_sanitizer, platform, shell};
```

### Q: localStorage 测试失败？

确保 `setup.ts` 中的 mock 正确设置，并在 `beforeEach` 中重置：
```typescript
localStorage.clear();
localStorage.setItem('LOG_LEVEL', 'debug');
```

## 扩展阅读

- [Vitest 官方文档](https://vitest.dev/)
- [Testing Library 文档](https://testing-library.com/docs/react-testing-library/intro/)
- [Rust 测试指南](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Zustand 测试指南](https://zustand.docs.pmnd.rs/guides/testing)
