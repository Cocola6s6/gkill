# 内部 Skill 客户端开发文档

## 一、项目概述

### 1.1 背景

内部 SkillHub 平台（`https://skills.gydev.cn`）已部署完毕，提供 skill 的存储、版本管理、搜索、发布等完整后端能力。当前团队使用 Node.js 编写的 `gkill` CLI 作为客户端工具。

**现有系统组成：**

| 组件 | 状态 | 说明 |
| --- | --- | --- |
| SkillHub 后端 | ✅ 已部署 | Spring Boot + PostgreSQL + MinIO，提供 REST API |
| SkillHub Web UI | ✅ 已部署 | React 19，浏览器访问 |
| gyskill CLI（Node.js） | ✅ 已存在 | 参考实现，npm 分发 |

### 1.2 目标

基于已有的 SkillHub 后端 API，分两个阶段提供更好的客户端体验：

- **阶段一**：用 Rust 重写 CLI，对齐 gyskill（Node.js）全部功能，提供单二进制分发
- **阶段二**：基于 Tauri + Sycamore 构建桌面图形应用

### 1.3 成功标准

**阶段一：**
- 单二进制文件，无需 Node.js 运行时
- 覆盖 gkill 所有命令：install / update / search / publish / add / remove / login / logout / whoami
- 支持所有 Agent 目录（claude-code / codex / cursor / antigravity 等）

**阶段二：**
- 桌面应用可视化浏览、安装、管理 skill
- 复用阶段一的核心逻辑库

---

## 二、整体架构

```text
┌─────────────────────────────────────────────┐
│              客户端层（本项目）                │
│                                             │
│  ┌──────────────┐    ┌───────────────────┐  │
│  │  Rust CLI    │    │  Tauri 桌面应用    │  │
│  │ (gkill) │    │ (Sycamore + WASM) │  │
│  └──────┬───────┘    └─────────┬─────────┘  │
│         └────────────┬─────────┘            │
│               ┌──────▼──────┐               │
│               │  gkill-  │               │
│               │    core     │               │
│               │  （共享库）  │               │
│               └──────┬──────┘               │
└──────────────────────┼──────────────────────┘
                       │ HTTP / REST
                       ▼
          ┌────────────────────────┐
          │  SkillHub 后端（已有）  │
          │  skills.gydev.cn       │
          │  Spring Boot + PG      │
          │  + MinIO               │
          └────────────────────────┘
```

### 2.1 Cargo Workspace 结构

```text
gkill/
├── Cargo.toml              # workspace
├── crates/
│   ├── gkill-core/       # 共享核心：API 客户端、安装逻辑、配置管理
│   ├── gkill-cli/    # 阶段一：CLI 入口
│   └── gkill-app/        # 阶段二：Tauri 桌面应用
```

---

## 三、SkillHub API 接口（已有，只需对接）

所有请求携带 `Authorization: Bearer <token>`（匿名命令除外）。

| 操作 | Method | Endpoint |
| --- | --- | --- |
| 搜索 | GET | `/api/web/skills?q=&sort=relevance&page=0&size=12` |
| 获取 skill 详情 | GET | `/api/v1/skills/{namespace}/{slug}` |
| 获取版本详情 | GET | `/api/v1/skills/{namespace}/{slug}/versions/{version}` |
| 下载（ZIP） | GET | `/api/v1/skills/{namespace}/{slug}/versions/{version}/download` |
| 发布 | POST | `/api/v1/skills/{namespace}/publish` (multipart/form-data) |
| 当前用户 | GET | `/api/v1/whoami` |

**版本号格式**：`YYYYMMDDHHmmss`（发布时由服务端生成时间戳）

**更新判断**：比较本地 `_meta.json.publishedAt` 与远端版本的 `publishedAt`（ISO 8601 字符串字典序比较）

---

## 四、阶段一：Rust CLI

### 4.1 技术栈

| 功能 | crate |
| --- | --- |
| CLI 框架 | `clap` v4（derive 模式） |
| HTTP 客户端 | `reqwest`（async，支持 multipart） |
| 异步运行时 | `tokio` |
| JSON 序列化 | `serde` + `serde_json` |
| ZIP 处理 | `zip` |
| 交互式 TUI | `ratatui` + `crossterm` |
| 交互提示 | `dialoguer`（select / multiselect / input） |
| 进度/spinner | `indicatif` |
| 系统目录 | `dirs` |
| 错误处理 | `anyhow` + `thiserror` |

### 4.2 命令全览

```bash
gkill install <slug> [--ns <ns>] [--agent <agent>] [--mode global|project] [--version <ver>] [--registry <url>]
gkill update  [slug]  [--agent <agent>] [--mode <mode>] [--registry <url>]
gkill search  [query] [--sort relevance|downloads] [--page <n>] [--size <n>] [--registry <url>]
gkill publish [path]  [--ns <ns>] [--visibility PUBLIC|NAMESPACE_ONLY|PRIVATE]
gkill add     <source>  [--agent <agent>] [--mode <mode>]   # GitHub 仓库
gkill remove  <slug>  [--agent <agent>] [--mode <mode>]
gkill login   [--token <token>]
gkill logout
gkill whoami  [--registry <url>]
gkill -V / --cli-version
```

### 4.3 支持的 Agent 目录

| Agent | `--agent` 值 | 项目目录 | 全局目录 |
| --- | --- | --- | --- |
| Claude Code | `claude-code` | `.claude/skills` | `~/.claude/skills` 或 `$CLAUDE_CONFIG_DIR/skills` |
| Codex | `codex` | `.agents/skills` | `~/.codex/skills` 或 `$CODEX_HOME/skills` |
| Cursor | `cursor` | `.agents/skills` | `~/.cursor/skills` |
| Antigravity | `antigravity` | `.agent/skills` | `~/.gemini/antigravity/skills` |
| OpenClaw | `openclaw` | `skills` | `~/.openclaw/skills` |
| OpenCode | `opencode` | `.agents/skills` | `~/.config/opencode/skills` |
| Trae | `trae` | `.trae/skills` | `~/.trae/skills` |
| Windsurf | `windsurf` | `.agents/skills` | `~/.windsurf/skills` |

### 4.4 本地配置与元数据

**认证配置**（与 gyskill Node.js 版兼容，可共享登录态）：

```
~/.config/gkill/config.json   →  { "token": "..." }
```

**已安装 skill 元数据**（每个 skill 目录下）：

```json
{
  "version": "20260415120000",
  "name": "demo-skill",
  "namespace": "global",
  "publishedAt": "2026-04-15T12:00:00.000Z"
}
```

### 4.5 核心流程

**install：**
1. 调 `/api/v1/skills/{ns}/{slug}` → 取 `publishedVersion.version`
2. 调版本详情接口 → 取 `parsedMetadataJson` + `publishedAt`
3. 调 download 接口 → 返回 ZIP 字节流
4. 清空目标目录，解压 ZIP
5. 写入 `_meta.json`

**publish：**
1. 递归查找目录下的 `SKILL.md`（单个或批量）
2. 将 skill 目录打包为 ZIP（校验路径安全，禁止 `..`）
3. POST multipart 到 `/api/v1/skills/{ns}/publish`

**update：**
1. 扫描 Agent 目录下所有含 `_meta.json` 的 skill
2. 逐一调接口获取远端 `publishedAt`
3. `remote_published_at > local_published_at`（字典序）→ 可更新
4. 多选后批量 install

**add（GitHub）：**
1. 解析 `owner/repo` 或 GitHub URL
2. 下载 `https://github.com/{owner}/{repo}/archive/HEAD.zip`
3. 扫描 `SKILL.md`（根目录或 `skills/*/SKILL.md`）
4. 多选后直接复制到 Agent 目录（不经过 Registry）

**search：**
- TTY 环境：ratatui TUI 列表，↑↓ 导航，←→ 翻页，Enter 安装
- 非 TTY：直接输出文本

### 4.6 错误处理

| 场景 | 处理 |
| --- | --- |
| HTTP 非 2xx | 输出 status + body，退出非 0 |
| 403 下载 | 提示无安装权限 |
| 未登录调需鉴权命令 | 提示先 `gkill login` |
| 本地 skill 不存在 | 提示先 `gkill install` |
| ZIP 路径含 `..` | 拒绝解压 |
| `_meta.json` 缺失 | update 时跳过该 skill |

---

## 五、阶段二：Tauri 桌面应用

### 5.1 技术栈

| 层 | 技术 |
| --- | --- |
| 应用框架 | Tauri v2 |
| 前端语言 | Rust |
| 前端框架 | Sycamore（响应式，编译为 WASM） |
| 前端构建 | Trunk |
| 后端逻辑 | 复用 `gkill-core` |

### 5.2 功能范围

- 浏览 / 搜索 SkillHub 上的 skill
- 可视化安装、更新、删除
- 管理已安装 skill 列表（分 Agent）
- 登录态管理
- 发布 skill（拖拽目录）

### 5.3 Tauri 命令设计

前端通过 `invoke()` 调用 Rust 后端命令，后端直接复用 `gkill-core`：

```rust
#[tauri::command]
async fn search_skills(query: String, page: u32) -> Result<SkillPage, String> { ... }

#[tauri::command]
async fn install_skill(slug: String, namespace: String, agent: String, mode: String) -> Result<(), String> { ... }

#[tauri::command]
async fn list_installed(agent: String, mode: String) -> Result<Vec<InstalledSkill>, String> { ... }

#[tauri::command]
async fn publish_skill(path: String, namespace: String, visibility: String) -> Result<(), String> { ... }
```

---

## 六、开发计划

### 阶段一（Rust CLI）

| 步骤 | 内容 |
| --- | --- |
| 1 | 初始化 Cargo workspace，搭建 gkill-core / gkill-cli 骨架 |
| 2 | 实现配置模块（token 读写）+ HTTP 客户端封装 |
| 3 | 实现 login / logout / whoami |
| 4 | 实现 install（核心流程：下载 ZIP + 解压 + _meta.json）|
| 5 | 实现 remove / update |
| 6 | 实现 publish（ZIP 打包 + multipart 上传）|
| 7 | 实现 search（ratatui TUI 交互列表）|
| 8 | 实现 add（GitHub 仓库 skill 发现）|
| 9 | 打包发布（cargo build --release，提供各平台二进制）|

### 阶段二（Tauri 桌面应用）

| 步骤 | 内容 |
| --- | --- |
| 1 | 初始化 gkill-app，配置 Tauri + Trunk + Sycamore |
| 2 | 暴露 gkill-core 为 Tauri commands |
| 3 | 实现 Sycamore 前端：搜索浏览页 |
| 4 | 实现已安装管理页（分 Agent 展示）|
| 5 | 实现发布页（拖拽上传）|
| 6 | 登录态管理 UI |
| 7 | 打包（macOS / Windows / Linux）|

---

## 七、参考

- SkillHub 后端：`https://skills.gydev.cn`
- gyskill（Node.js 参考实现）：内部 npm registry
- SkillHub 开源仓库：`https://github.com/iflytek/skillhub`
- OpenClaw CLI（同类参考）：`https://github.com/openclaw/openclaw`
