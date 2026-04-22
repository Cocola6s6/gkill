# Phase 1：Rust CLI（gkill）

## 目标
用 Rust 重写 gyskill CLI 的全部功能，提供单二进制分发，无需 Node.js 运行时。

## 技术栈
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

## Cargo Workspace 结构
```
gkill/
├── Cargo.toml              # workspace
├── crates/
│   ├── gkill-core/       # 共享核心：API 客户端、安装逻辑、配置管理
│   └── gkill-cli/    # CLI 入口
```

## 任务拆分

### 1.1 工程骨架
- 初始化 Cargo workspace
- 创建 `gkill-core` 和 `gkill-cli` crate
- 配置各 crate 的 Cargo.toml 依赖

### 1.2 配置 & HTTP 模块（gkill-core）
- `config` 模块：token 读写 `~/.config/gkill/config.json`，文件权限 0o600
- `http` 模块：封装 reqwest client，统一添加 `Authorization: Bearer <token>`，超时 15s，自定义 `HttpError`

### 1.3 认证命令
- `login [--token]`：交互或直接保存 token
- `logout`：清除 config.json
- `whoami [--registry]`：GET `/api/v1/whoami`

### 1.4 install 命令
- GET `/api/v1/skills/{ns}/{slug}` → 取 `publishedVersion.version`
- GET 版本详情 → `parsedMetadataJson` + `publishedAt`
- GET `.../download` → ZIP 字节流
- 清空目标目录，解压 ZIP，路径安全校验（禁止 `..`）
- 写入 `_meta.json`
- 未传 `--agent` / `--mode` 时 dialoguer 交互选择

### 1.5 remove & update 命令
- `remove`：`fs::remove_dir_all(skill_dir)`
- `update`：扫描 `_meta.json` → 对比 `publishedAt` 字符串字典序 → dialoguer 多选 → 批量 install

### 1.6 publish 命令
- 递归查找 `SKILL.md`，支持单个或批量
- `zip` crate 打包，路径安全校验
- POST multipart `/api/v1/skills/{ns}/publish`
- 未传 `--visibility` 时交互选择

### 1.7 search 命令
- GET `/api/web/skills?q=&sort=&page=&size=`
- TTY 环境：ratatui TUI 列表，↑↓ 导航，←→ 翻页，Enter 触发 install
- 非 TTY：纯文本输出

### 1.8 add 命令（GitHub）
- 解析 `owner/repo` 或 `https://github.com/owner/repo`
- 下载 `https://github.com/{owner}/{repo}/archive/HEAD.zip`
- 扫描 `SKILL.md`（根目录 or `skills/*/SKILL.md`）
- dialoguer 多选 → 直接复制文件到 Agent 目录（不经 Registry）

### 1.9 发布打包
- `cargo build --release`
- 提供 macOS（arm64 + x86_64）、Linux（x86_64）、Windows（x86_64）二进制

## Agent 目录映射
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

## 验收标准
- [ ] `gkill login/logout/whoami` 正常工作
- [ ] `gkill install <slug>` 能下载并解压到正确 Agent 目录
- [ ] `gkill update` 能检测并批量更新
- [ ] `gkill remove <slug>` 能删除
- [ ] `gkill publish` 能打包上传
- [ ] `gkill search` 有 TUI 交互
- [ ] `gkill add` 能从 GitHub 安装
- [ ] 单二进制，无依赖
