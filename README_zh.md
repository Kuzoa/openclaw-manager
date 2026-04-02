# 🦞 OpenClaw Manager

**[English](./README.md)** | 简体中文

[OpenClaw](https://github.com/miaoxworld/OpenClawInstaller) 的一键安装器与管理界面 —— 开源 AI 助手框架。

基于 **Tauri 2.0 + React 18 + TypeScript + Rust** 构建，为桌面平台提供原生级性能体验。

![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux-blue)
![Tauri](https://img.shields.io/badge/Tauri-2.0-orange)
![React](https://img.shields.io/badge/React-18-61DAFB)
![Rust](https://img.shields.io/badge/Rust-1.70+-red)

---

## ✨ 功能特性

### 🚀 一键安装向导
告别命令行操作。内置安装向导自动检测您的环境，安装 Node.js 和 OpenClaw，完成所有初始化 —— 全程在图形界面中完成。

### 📊 仪表盘与服务控制
OpenClaw 服务的实时监控与全生命周期管理。
- **服务状态**：端口、进程 ID、内存占用、运行时长
- **服务守护**：当网关通过 Telegram 命令重启或意外崩溃时自动恢复
- **日志查看器**：结构化的本地应用日志，支持按警告、错误过滤，一键导出
- **Web 控制界面**：直接与您的 Agent 对话 (`http://localhost:{GATEWAY_PORT}`)
- **进度反馈**：环境检测过程中的可视化进度指示器，实时显示检测状态

### 🤖 全面的 AI 配置
灵活的多提供商 AI 连接，无缝集成 **Ollama**。

**支持的提供商：**
- **Google Gemini**（新！✨）：Gemini 3 Pro、Gemini 3 Flash
- **Anthropic**：Claude 3.5 Sonnet、Opus
- **OpenAI**：GPT-4o、GPT-4o-mini
- **DeepSeek**：DeepSeek V3（Chat）、DeepSeek R1（Reasoner）
- **本地模型（Ollama）**：自动检测 Ollama 安装，直接在界面中搜索、拉取和管理本地模型（如 `llama3`、`qwen3.5:9b`）
- **自定义提供商配置**：添加任何兼容 OpenAI 或 Anthropic API 的端点，设置您的专属模型

### ⚙️ 高级设置与调优
通过图形界面精细配置整个 OpenClaw 生态系统。

- **压缩与内存优化**：在压缩触发前映射 token，管理上下文修剪，限制消息保留，使用 Ollama 映射离线本地嵌入
- **子 Agent 全局默认值**：管理复杂的 Agent 嵌套限制，定义最大生成深度、每个 Agent 的最大子级数量，限制并发子 Agent 处理
- **工具与安全配置**：为您的实例设置严格的安全边界（消息、最小、编码、完全访问）
- **原生 PDF 支持**：配置附加复杂文档处理的最大 token 页数和负载大小（MB）限制
- **内联文件附件**：启用/禁用子 Agent 分析拖放的标准会话附件，定义每个会话的最大字节阈值
- **浏览器控制与网络搜索**：集成您的 Brave Search API 密钥，让 Agent 能够探索网络，自定义内部 Agent 浏览器窗口的 Chrome 配色
- **网络自定义**：动态调整网关端口（如标准 `3000`）和全局调试日志级别（如 debug、info、warn）
- **工作区本地化**：配置本地时区和首选时间格式（如 12 小时制 AM/PM 或 24 小时制）

### 📋 配置管理
再也不用担心丢失 `.openclaw.json` 或模型配置！
- 图形界面配置直接同步到 `.openclaw.json`
- 在界面中提供 Schema 验证
- 使用 JSON 在本地导入、导出、备份和恢复整个配置

### 🌐 本地化与性能
- **多语言支持**：完整的国际化（i18n）支持，提供中英文界面，在设置中无缝切换语言
- **环境缓存**：环境检测结果持久化缓存，减少重复检测，提升启动性能

### 🧩 MCP 管理
完整的 [Model Context Protocol](https://modelcontextprotocol.io/) 服务器管理，集成 **mcporter** 支持。动态设置简单的 StdIo 本地命令或远程 SSE 钩子，更改自动同步到本地 `~/.mcporter/mcporter.json`。

### 📚 技能管理
浏览、安装和管理通过 **ClawHub** 分发的 OpenClaw 能力（如专业编码、Web 开发）。

### 📱 消息通道
将 OpenClaw 连接到多个全渠道聊天平台。
**支持的通道：** Telegram、飞书、Discord、Slack、WhatsApp。在界面中完成需要令牌、密钥哈希、ID、授权群组/用户的完整配置，立即绑定到网关。

### 🔄 OpenClaw Manager 自更新
在应用设置中获得自动空中升级（OTA）更新！当新版本构建时，收到通知、拉取最新版本、安全重启以使用新功能 —— 无需手动重新安装！

---

## 📁 项目结构

```
openclaw-manager/
├── src-tauri/                 # Rust 后端
│   ├── src/
│   │   ├── main.rs            # 入口点
│   │   ├── commands/          # 后端逻辑（配置、安装、服务等）
│   │   ├── models/            # 数据结构
│   │   └── utils/             # 辅助工具
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                       # React 前端
│   ├── components/            # UI 组件（仪表盘、设置等）
│   ├── hooks/                 # 自定义 Hooks
│   ├── lib/                   # API 绑定
│   ├── stores/                # 状态管理（Zustand）
│   └── styles/                # Tailwind CSS
│
├── package.json
└── vite.config.ts
```

---

## 🛠️ 技术栈

| 层级 | 技术 | 用途 |
|------|------|------|
| 前端 | React 18 | UI 框架 |
| 状态 | Zustand | 轻量级响应式状态 |
| 样式 | TailwindCSS | 实用优先的 CSS |
| 动画 | Framer Motion | 流畅过渡与微交互 |
| 后端 | Rust | 高性能系统操作 |
| 桌面 | Tauri 2.0 | 原生跨平台外壳 |
| 测试 | Vitest + Cargo test | 前后端单元测试 |

---

## 🚀 快速开始（开发）

### 环境要求

| 工具 | 版本 | 下载 |
|------|------|------|
| **Node.js** | >= 18.0 | [nodejs.org](https://nodejs.org/) |
| **Rust** | >= 1.70 | [rustup.rs](https://rustup.rs/) |
| **pnpm** 或 npm | 最新版 | 随 Node.js 安装 |

### 克隆并运行

```bash
git clone https://github.com/MrFadiAi/openclaw-one-click-installer.git
cd openclaw-one-click-installer

npm install          # 安装依赖
npm run tauri:dev    # 启动开发模式（热重载）
```

> **注意：** 首次构建会编译所有 Rust 依赖，需要 **3–5 分钟**。后续运行会快很多。

### 运行测试

```bash
npm test             # 运行前端单元测试
cd src-tauri && cargo test  # 运行后端测试
```

### 构建发布版

```bash
npm run tauri:build
```

输出位于 `src-tauri/target/release/bundle/`：

| 平台 | 格式 |
|------|------|
| Windows | `.msi`、`.exe` |
| Linux | `.deb`、`.AppImage` |

---

## 🤝 参与贡献

1. Fork 本项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 提交 Pull Request

---

## 📄 许可证

MIT 许可证 — 详情见 [LICENSE](LICENSE)。

---

**由 OpenClaw 社区用 ❤️ 构建**
