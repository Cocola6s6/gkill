# gkill 使用手册

> SkillHub CLI + 桌面客户端，用于管理、发布、搜索 AI Agent Skill。
>
> SkillHub 后端地址：**https://skills.gydev.cn**

---

## 目录

- [环境依赖](#环境依赖)
- [构建](#构建)
  - [CLI](#构建-cli)
  - [桌面应用 (Tauri)](#构建桌面应用)
- [CLI 使用](#cli-使用)
  - [认证](#认证)
  - [搜索](#搜索)
  - [安装](#安装)
  - [更新](#更新)
  - [删除](#删除)
  - [发布](#发布)
  - [从 GitHub 添加](#从-github-添加)
- [桌面应用使用](#桌面应用使用)
- [支持的 Agent](#支持的-agent)
- [安装目录说明](#安装目录说明)
- [配置文件](#配置文件)
- [重新安装 gkill](#重新安装-gkill)

---

## 环境依赖

| 工具 | 版本要求 | 用途 |
|------|---------|------|
| Rust + Cargo | ≥ 1.75 | 编译 CLI 与 Tauri 后端 |
| Node.js | ≥ 18（仅 Tauri bundle 时需要） | Tauri CLI |
| trunk | ≥ 0.21 | 编译 WASM 前端 |
| wasm32-unknown-unknown target | — | WASM 编译目标 |

```bash
# 安装 trunk
cargo install trunk

# 添加 WASM target
rustup target add wasm32-unknown-unknown
```

---

## 构建

### 构建 CLI

```bash
cd gkill

# 开发构建
cargo build -p gkill

# Release 构建
cargo build -p gkill --release

# 二进制位于
# target/debug/gkill  （或 target/release/gkill）
```

快速运行（无需安装）：

```bash
cargo run -p gkill -- <子命令> [参数]
```

全局安装到 `~/.cargo/bin`：

```bash
cargo install --path crates/gkill
```

---

### 构建桌面应用

桌面应用分两步：**先构建 WASM 前端，再构建 Tauri 后端**。

**第一步：构建前端**

```bash
cd gkill/crates/gkill-app/ui
trunk build              # 开发构建，输出到 ui/dist/
trunk build --release    # 优化构建
```

**第二步：构建 Tauri 后端**

```bash
cd gkill
cargo build -p gkill-app          # 开发
cargo build -p gkill-app --release
```

**直接运行（开发模式）：**

```bash
# 终端 1：前端热重载
cd gkill/crates/gkill-app/ui
trunk watch

# 终端 2：启动 Tauri 应用
cd gkill
cargo run -p gkill-app
```

> **⚠️ 重要：`cargo run -p gkill-app` 不会自动重新编译前端。**
> Tauri 在编译时将 `ui/dist/` 的静态产物打包进二进制，因此：
>
> | 改动范围 | 需要执行的命令 |
> |---|---|
> | UI 代码（`pages/*.rs`、`api.rs` 等 Sycamore 文件） | 先 `trunk build`（在 `ui/` 目录）→ 再 `cargo run -p gkill-app` |
> | Tauri 后端（`commands/*.rs`、`lib.rs`） | 直接 `cargo run -p gkill-app` |
> | CLI（`gkill`） | `cargo install --path crates/gkill`（独立二进制，与 Tauri app 无关） |

**打包分发（生成 .app / .exe / .deb 等）：**

```bash
# 需要先安装 tauri-cli
cargo install tauri-cli --version "^2"

cd gkill/crates/gkill-app
cargo tauri build
# 产物在 target/release/bundle/
```

---

## CLI 使用

所有命令格式：

```
gkill <子命令> [参数] [选项]
```

查看帮助：

```bash
gkill --help
gkill <子命令> --help
```

---

### 认证

**登录**（交互式输入 Token）：

```bash
gkill login
# 提示：请输入 SkillHub API Token
```

**登录**（直接传入 Token）：

```bash
gkill login --token <your-token>
```

> Token 保存于 `~/.config/gkill/config.json`（权限 0o600）。

**查看当前登录用户**：

```bash
gkill whoami
```

**登出**：

```bash
gkill logout
```

---

### 搜索

```bash
# 打开 TUI 搜索界面（交互式）
gkill search

# 带初始关键词
gkill search "代码审查"

# 按下载量排序
gkill search --sort downloads
```

**TUI 快捷键：**

| 按键 | 操作 |
|------|------|
| `↑` / `↓` | 移动选中 |
| `←` / `→` | 翻页 |
| `Enter` | 安装选中的 skill |
| `/` | 聚焦搜索框 |
| `q` / `Esc` | 退出 |

> 在非 TTY 环境（如管道、CI）下，自动切换为纯文本输出。

---

### 安装

```bash
# 最简用法（安装到默认 Agent claude-code，global 模式）
gkill install <slug>

# 指定命名空间
gkill install <slug> --ns <namespace>

# 指定 Agent
gkill install <slug> --agent cursor

# 安装到项目目录（project 模式）
gkill install <slug> --mode project

# 指定版本
gkill install <slug> --version 2024-01-15T10:00:00Z

# 组合示例
gkill install code-review --ns gyyx --agent claude-code --mode global
```

---

### 更新

```bash
# 检查并更新所有 skill（多选交互）
gkill update

# 更新指定 slug
gkill update <slug>

# 指定 Agent 和模式
gkill update --agent cursor --mode global
```

---

### 删除

```bash
gkill remove <slug>

# 指定 Agent 和模式
gkill remove <slug> --agent windsurf --mode project
```

---

### 发布

```bash
# 发布当前目录下的 skill
gkill publish

# 发布指定目录
gkill publish /path/to/skill-dir

# 指定命名空间和可见性
gkill publish --ns gyyx --visibility PUBLIC

# 可见性选项：PUBLIC | NAMESPACE_ONLY | PRIVATE
```

> `publish` 命令会自动发现目录下的所有 skill（每个含 `SKILL.md` 的子目录），可多选后批量发布。

---

### 从 GitHub 添加

直接从 GitHub 仓库拉取 skill，无需经过 SkillHub：

```bash
# 简写格式
gkill add owner/repo

# 完整 URL
gkill add https://github.com/owner/repo

# 指定 Agent
gkill add owner/repo --agent claude-code --mode project
```

> 支持多选：仓库中有多个 skill 时，会弹出多选列表。

---

## 桌面应用使用

启动后界面分 4 个 Tab：

### 🔍 搜索 Tab

- 在搜索框输入关键词，按 `Enter` 或点击「搜索」
- 结果卡片显示 slug、描述、命名空间、下载量
- 点击「安装」按钮一键安装（使用默认 Agent: `claude-code`，模式: `global`）
- 支持分页翻页

### 📦 已安装 Tab

- 列出本地已安装的全部 skill
- 点击「检查更新」扫描可用更新，展示版本差异
- 逐条点击「更新」或「移除」

### 🚀 发布 Tab

- 填写 skill 压缩包本地路径（`.tar.gz`）
- 选择可见性：公开 / 私有
- 点击「发布」上传到 SkillHub

### ⚙️ 设置 Tab

- **未登录状态**：粘贴 API Token 后点击「保存 Token」
- **已登录状态**：显示当前用户名，可点击「退出登录」

> **获取 Token**：先通过 CLI `gkill login` 登录，Token 会自动保存；桌面应用读取同一配置文件，无需重复输入。

---

## 支持的 Agent

| Agent ID | 显示名 | 全局 skill 目录 |
|----------|--------|----------------|
| `claude-code` | Claude Code | `~/.claude/skills/` |
| `codex` | Codex | `~/.codex/skills/` |
| `cursor` | Cursor | `~/.cursor/skills/` |
| `antigravity` | Antigravity | `~/.gemini/antigravity/skills/` |
| `openclaw` | OpenClaw | `~/.openclaw/skills/` |
| `opencode` | OpenCode | `~/.config/opencode/skills/` |
| `trae` | Trae | `~/.trae/skills/` |
| `windsurf` | Windsurf | `~/.windsurf/skills/` |

> 未指定 `--agent` 时，CLI 会读取环境变量 `GYYXCLI_AGENT`，若也未设置则使用 `claude-code`。

---

## 安装目录说明

| 模式 | 路径 |
|------|------|
| `global`（默认） | Agent 的全局 skill 目录（见上表） |
| `project` | `<当前目录>/<agent.skills_dir>/` |

**示例（claude-code, global）：**

```
~/.claude/skills/
└── code-review/
    ├── SKILL.md
    └── _meta.json      ← 版本信息，由 gkill 自动管理
```

---

## 配置文件

```
~/.config/gkill/config.json
```

```json
{
  "token": "your-bearer-token"
}
```

文件权限为 `0o600`，仅当前用户可读。可通过 `gkill login` / `gkill logout` 管理，无需手动编辑。

---

## 重新安装 gkill

```bash
# 卸载（包名是 gkill）
cargo uninstall gkill

# 重新编译安装
cd gkill
cargo install --path crates/gkill
```
