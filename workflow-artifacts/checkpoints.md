# Checkpoints

## Phase 1 — Rust CLI（gkill）✅

**完成时间**: 2026-04-22  
**状态**: complete

### 交付物

| 文件 | 描述 |
| --- | --- |
| `gkill/Cargo.toml` | Workspace 根配置，统一管理依赖版本 |
| `crates/gkill-core/` | 核心库：types / config / http / api / agents / install / update / publish / add |
| `crates/gkill-cli/` | CLI 入口：main.rs + 9 个命令模块 |

### 已实现命令

| 命令 | 功能 |
| --- | --- |
| `login` | 保存 Bearer Token 到 `~/.config/gkill/config.json`（0o600） |
| `logout` | 清除本地 Token |
| `whoami` | 调用 `/api/v1/auth/me` 显示用户信息 |
| `install` | 下载并解压 skill ZIP 到 Agent 目录，写入 `_meta.json` |
| `remove` | 删除 Agent 目录下的 skill |
| `update` | 扫描已安装 skill，对比远端 `published_at`，批量更新 |
| `publish` | 打包当前目录 skill(s) 为 ZIP，上传到 SkillHub |
| `add` | 从 GitHub 仓库直接拉取 skill（支持简写 `owner/repo` 和完整 URL） |
| `search` | ratatui TUI 搜索（↑↓导航，←→翻页，Enter安装，q退出）；非 TTY 时输出纯文本 |

### 测试结果

```
running 8 tests
test add::tests::test_parse_github_source_git_url ... ok
test add::tests::test_parse_github_source_shorthand ... ok
test install::tests::test_safe_path ... ok
test add::tests::test_parse_github_source_url ... ok
test add::tests::test_parse_github_source_invalid ... ok
test update::tests::test_is_newer ... ok
test publish::tests::test_discover_skills_root ... ok
test publish::tests::test_discover_skills_subdirs ... ok

test result: ok. 8 passed; 0 failed
```

### 关键技术决策

- **reqwest rustls-tls**：不依赖系统 OpenSSL，跨平台兼容性好
- **ratatui + crossterm**：纯 Rust TUI，search 命令状态机（Loading→Displaying）
- **ZIP path safety**：`is_safe_path` 防路径穿越攻击
- **版本比较**：ISO 8601 字符串字典序比较（`published_at` 字段）
- **Token 存储**：`~/.config/gkill/config.json`，权限 0o600

### 下一阶段

Phase 2 — Tauri + Sycamore 桌面应用（`gkill-app`）

---

## Phase 2 — Tauri 桌面应用（gkill-app）✅

**完成时间**: 2026-04-22  
**状态**: complete

### 交付物

| 文件 | 描述 |
| --- | --- |
| `crates/gkill-app/Cargo.toml` | Tauri v2 backend crate（workspace member） |
| `crates/gkill-app/build.rs` | `tauri_build::build()` |
| `crates/gkill-app/tauri.conf.json` | Tauri 配置：窗口、frontendDist、bundle |
| `crates/gkill-app/src/lib.rs` | Tauri Builder，注册 12 个命令 |
| `crates/gkill-app/src/main.rs` | 入口，调用 `gkill_app::run()` |
| `crates/gkill-app/src/commands/mod.rs` | 共享类型（AuthStatus、InstalledInfo、AgentInfo）、`client()` helper |
| `crates/gkill-app/src/commands/auth.rs` | login / logout / get_auth_status / whoami |
| `crates/gkill-app/src/commands/skills.rs` | list_agents / search_skills / install_skill / list_installed / remove_skill / find_updates / update_skill |
| `crates/gkill-app/src/commands/publish.rs` | publish_skill |
| `crates/gkill-app/ui/Cargo.toml` | Sycamore 0.9 UI crate（独立 workspace，WASM 目标） |
| `crates/gkill-app/ui/Trunk.toml` | trunk 构建配置 |
| `crates/gkill-app/ui/index.html` | Tailwind CDN + Trunk link |
| `crates/gkill-app/ui/src/state.rs` | 镜像类型 + AppCtx 全局信号 |
| `crates/gkill-app/ui/src/api.rs` | wasm-bindgen Tauri invoke shim + 类型化 async API |
| `crates/gkill-app/ui/src/pages/search.rs` | 搜索 + 分页 + 安装 |
| `crates/gkill-app/ui/src/pages/installed.rs` | 已安装列表 + 更新/移除 |
| `crates/gkill-app/ui/src/pages/publish.rs` | Skill 发布表单 |
| `crates/gkill-app/ui/src/pages/settings.rs` | Token 登录/退出 |
| `crates/gkill-app/ui/src/main.rs` | App 组件 + Tab 导航 + Toast |

### Tauri 命令清单（共 12 个）

| 命令 | 类型 | 说明 |
| --- | --- | --- |
| `login` | sync | 写入 Bearer Token |
| `logout` | sync | 清除 Token |
| `get_auth_status` | sync | 返回登录状态 + registry |
| `whoami` | async | 调用 /api/v1/auth/me |
| `list_agents` | sync | 列出支持的 Agent 列表 |
| `search_skills` | async | 分页搜索 skill |
| `install_skill` | async | 下载并安装 skill |
| `list_installed` | sync | 列出本地已安装 skill |
| `remove_skill` | sync | 移除已安装 skill |
| `find_updates` | async | 检查可用更新 |
| `update_skill` | async | 更新单个 skill |
| `publish_skill` | async | 发布 skill 压缩包 |

### 构建验证

```
cargo build -p gkill-app  → Finished ✅
trunk build (UI)            → ✅ success
```

### 关键技术决策

- **Sycamore 0.9 Signal API**: `get()` 返回 `T`（需 `Copy`），非 Copy 类型用 `get_clone()`；`Indexed` 要求 item 实现 `PartialEq`
- **UI crate 隔离**: `ui/Cargo.toml` 含独立 `[workspace]`，不加入父 workspace（WASM 目标与 native 不兼容）
- **Tauri invoke shim**: 通过 `wasm_bindgen(inline_js)` 调用 `window.__TAURI_INTERNALS__.invoke`
- **token 登录**: 与 CLI 统一，通过 token 认证，而非用户名/密码
- **agent/mode 默认值**: UI api 层默认使用 `claude-code` + `global`，简化页面调用

### 下一阶段

Phase 3（如有）— CI/CD 发布打包 / Tauri bundle 多平台分发
