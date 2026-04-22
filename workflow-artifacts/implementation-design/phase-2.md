# Phase 2 实现设计：Tauri 桌面应用（gkill-app）

## 技术选型

| 层 | 选型 | 版本 | 理由 |
|----|------|------|------|
| 应用框架 | Tauri v2 | 2.x | 轻量桌面容器，Rust 原生，3平台打包 |
| 前端框架 | Sycamore | 0.9.2 | 纯 Rust WASM，响应式信号，无 JS 依赖 |
| 前端构建 | Trunk | latest | 官方 Sycamore + WASM 构建工具 |
| 前端样式 | Tailwind CSS via CDN | 3.x | 不引入 Node 构建链，CDN 内联即可 |
| Tauri-WASM 桥 | wasm-bindgen + js-sys | 0.2 | `tauri::invoke()` 由 `@tauri-apps/api` JS 层提供，WASM 用 `wasm-bindgen` 调用 |
| 核心逻辑 | gkill-core | 复用 Phase 1 | 零重复代码 |

### 注意事项
- **不** 将 UI crate 加入主 workspace（WASM target 与 native target 混用会造成 cargo build 困扰）
- UI crate 放在 `crates/gkill-app/ui/`，有独立 `Cargo.toml`，由 Trunk 单独构建
- Tauri 后端 crate（`crates/gkill-app/`）加入主 workspace

---

## 工程结构

```
gkill/
├── Cargo.toml                        # workspace（新增 gkill-app）
└── crates/
    ├── gkill-core/                 # 复用，不改动
    ├── gkill-cli/                  # 复用，不改动
    └── gkill-app/                  # Tauri 后端 crate（native）
        ├── Cargo.toml
        ├── build.rs                  # tauri-build
        ├── tauri.conf.json           # Tauri 配置（窗口、命令白名单）
        ├── src/
        │   ├── main.rs               # App 入口 + Tauri 命令注册
        │   └── commands/
        │       ├── mod.rs
        │       ├── skills.rs         # search/install/remove/update
        │       ├── publish.rs        # publish
        │       └── auth.rs           # login/logout/whoami
        └── ui/                       # Sycamore WASM crate（独立 Cargo）
            ├── Cargo.toml
            ├── Trunk.toml
            ├── index.html
            └── src/
                ├── main.rs           # Sycamore app 入口
                ├── api.rs            # wasm-bindgen invoke 封装
                ├── state.rs          # 全局 context：auth / registry
                └── pages/
                    ├── search.rs     # 搜索 + 安装页
                    ├── installed.rs  # 已安装管理页
                    ├── publish.rs    # 发布页
                    └── settings.rs  # 登录 / 登出 / 配置页
```

---

## Tauri Commands 接口设计

所有命令在 `src-tauri/src/commands/` 实现，通过 `tauri::Builder::invoke_handler` 注册。

### auth.rs

```rust
#[tauri::command]
fn login(token: String) -> Result<(), String>

#[tauri::command]
fn logout() -> Result<(), String>

#[tauri::command]
async fn whoami(registry: Option<String>) -> Result<serde_json::Value, String>

#[tauri::command]
fn get_auth_status() -> AuthStatus  // { logged_in: bool, registry: String }
```

### skills.rs

```rust
#[tauri::command]
async fn search_skills(query: String, sort: String, page: u32, size: u32, registry: Option<String>)
    -> Result<SkillPage, String>

#[tauri::command]
async fn install_skill(slug: String, namespace: String, agent: String, mode: String, version: Option<String>)
    -> Result<(), String>

#[tauri::command]
fn list_installed(agent: String, mode: String) -> Result<Vec<InstalledInfo>, String>

#[tauri::command]
fn remove_skill(slug: String, agent: String, mode: String) -> Result<(), String>

#[tauri::command]
async fn find_updates(agent: String, mode: String, registry: Option<String>)
    -> Result<Vec<UpdateCandidate>, String>

#[tauri::command]
async fn update_skill(slug: String, namespace: String, agent: String, mode: String)
    -> Result<(), String>
```

### publish.rs

```rust
#[tauri::command]
async fn publish_skill(path: String, namespace: String, visibility: String)
    -> Result<(), String>

#[tauri::command]
fn list_agents() -> Vec<AgentInfo>  // { id, display_name }
```

### 共用数据结构（commands/mod.rs）

```rust
pub struct AuthStatus { pub logged_in: bool, pub registry: String }
pub struct InstalledInfo { pub slug: String, pub namespace: String, pub version: String, pub published_at: String }
pub struct AgentInfo { pub id: String, pub display_name: String }
// SkillPage, UpdateCandidate 复用 gkill-core::types
```

---

## Sycamore 前端设计

### 全局状态（state.rs）

```rust
pub struct AppCtx {
    pub page: Signal<Page>,        // Search | Installed | Publish | Settings
    pub auth: Signal<AuthStatus>,  // logged_in, registry
    pub agent: Signal<String>,     // 当前选中 Agent id
    pub mode: Signal<String>,      // "global" | "project"
}
```

### invoke 封装（api.rs）

```rust
// wasm-bindgen 调用 window.__TAURI_INTERNALS__.invoke(cmd, args)
pub async fn invoke<T: DeserializeOwned>(cmd: &str, args: JsValue) -> Result<T, String>
```

### 页面说明

| 页面 | 组件 | 核心交互 |
|------|------|----------|
| Search | `SearchBar` + `SkillCard` + `Pagination` | 输入关键词 → invoke search_skills → 渲染列表；Install 按钮 → invoke install_skill → toast |
| Installed | `AgentSelector` + `SkillRow` | 切换 Agent/mode → invoke list_installed；Update/Remove 按钮 |
| Publish | `DirPicker` + `MetaForm` | 选目录 → invoke publish_skill → 进度 |
| Settings | `LoginForm` + `UserInfo` | Token 输入 → invoke login；显示 whoami；Logout |

### 路由方式
SPA 内部状态切换（无 URL router），顶部 Tab 导航：搜索 / 已安装 / 发布 / 设置

---

## 测试用例

| ID | 类型 | 前置条件 | 步骤 | 预期 |
|----|------|----------|------|------|
| T2-1 | 单元 | — | `list_agents()` 调用 | 返回 8 个 Agent，含 claude-code |
| T2-2 | 单元 | Token 已保存 | `get_auth_status()` | logged_in=true |
| T2-3 | 单元 | 无 Token | `get_auth_status()` | logged_in=false |
| T2-4 | 单元 | 目录含 SKILL.md | `list_installed(agent, "global")` | 返回正确列表 |
| T2-5 | 编译 | — | `cargo build -p gkill-app` | 0 错误 0 警告 |
| T2-6 | 编译 | — | `trunk build`（ui/） | 0 错误，生成 dist/ |
| T2-7 | 集成 | Tauri dev 环境 | `cargo tauri dev` 启动 | 窗口正常打开，搜索页显示 |

---

## 实现落点（按顺序）

1. **workspace Cargo.toml** — 新增 `crates/gkill-app` member
2. **gkill-app/Cargo.toml** — 依赖 tauri 2, tauri-build, gkill-core
3. **gkill-app/build.rs** — `tauri_build::build()`
4. **gkill-app/tauri.conf.json** — 窗口配置，指向 `ui/dist`
5. **commands/auth.rs / skills.rs / publish.rs** — 所有 Tauri 命令实现
6. **src/main.rs** — 注册全部命令，Builder::invoke_handler
7. **ui/Cargo.toml** — sycamore, wasm-bindgen, js-sys, serde-wasm-bindgen
8. **ui/Trunk.toml + index.html** — Trunk 构建配置
9. **ui/src/api.rs** — invoke 封装
10. **ui/src/state.rs** — AppCtx context
11. **ui/src/pages/** — 4 个页面组件
12. **ui/src/main.rs** — provide_context + Router render

---

## 关键约束

- Tauri commands 的 `Result<T, String>` 中 `String` 是 JS 侧可见错误消息，不暴露内部堆栈
- `list_installed` / `remove_skill` / `login` / `logout` 是同步命令（无 async），其他为 async
- UI 编译目标为 `wasm32-unknown-unknown`，不在主 workspace 中
- Tailwind 样式仅用 CDN Play（避免引入 Node/npm 构建链）
