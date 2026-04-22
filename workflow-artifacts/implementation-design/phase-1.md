# Phase 1 实现设计：gkill Rust CLI

## 1. 整体结构

```
gkill/
├── Cargo.toml                    # workspace
└── crates/
    ├── gkill-core/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── config.rs         # token 读写、registry 配置
    │       ├── http.rs           # reqwest client 封装、HttpError
    │       ├── api.rs            # SkillHub API 调用层
    │       ├── agents.rs         # Agent 目录映射
    │       ├── install.rs        # install / remove 核心逻辑
    │       ├── update.rs         # update 核心逻辑
    │       ├── publish.rs        # ZIP 打包 + 上传
    │       ├── add.rs            # GitHub skill 发现与安装
    │       └── types.rs          # 公共数据结构
    └── gkill-cli/
        ├── Cargo.toml
        └── src/
            ├── main.rs           # clap 命令树入口
            └── commands/
                ├── login.rs
                ├── logout.rs
                ├── whoami.rs
                ├── install.rs
                ├── remove.rs
                ├── update.rs
                ├── publish.rs
                ├── search.rs     # ratatui TUI
                └── add.rs
```

---

## 2. gkill-core 设计

### 2.1 types.rs — 公共数据结构

```rust
// API 响应
pub struct SkillDetail {
    pub published_version: Option<VersionRef>,
    pub headline_version: Option<VersionRef>,
}
pub struct VersionRef { pub version: String }

pub struct VersionDetail {
    pub published_at: String,           // ISO 8601
    pub parsed_metadata_json: Option<serde_json::Value>,
}

pub struct SkillPage {
    pub items: Vec<SkillItem>,
    pub total: u64,
    pub size: u32,
}
pub struct SkillItem {
    pub slug: String,
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    pub downloads: u64,
}

// 本地元数据
pub struct SkillMeta {
    pub version: String,
    pub name: String,
    pub namespace: String,
    pub published_at: String,
}

// Agent 配置
pub struct AgentConfig {
    pub name: &'static str,
    pub skills_dir: &'static str,       // 项目相对路径
    pub global_skills_dir: String,      // 绝对路径（运行时计算）
}
```

### 2.2 config.rs — 配置管理

```rust
// 路径：~/.config/gkill/config.json
// 格式：{ "token": "..." }

pub fn config_path() -> PathBuf
pub fn read_token() -> anyhow::Result<Option<String>>
pub fn write_token(token: &str) -> anyhow::Result<()>   // 0o600
pub fn clear_token() -> anyhow::Result<()>
pub const DEFAULT_REGISTRY: &str = "https://skills.gydev.cn";
```

### 2.3 http.rs — HTTP 客户端

```rust
pub struct Client {
    inner: reqwest::Client,         // timeout: 15s
    token: Option<String>,
    registry: String,
}

impl Client {
    pub fn new(registry: &str, token: Option<String>) -> Self
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T>
    pub async fn get_bytes(&self, path: &str) -> anyhow::Result<Bytes>
    pub async fn post_multipart(&self, path: &str, form: Form) -> anyhow::Result<serde_json::Value>
}

// 非 2xx → HttpError { status, message }
pub struct HttpError { pub status: u16, pub message: String }
```

### 2.4 api.rs — API 调用层

```rust
pub async fn get_skill(c: &Client, ns: &str, slug: &str) -> anyhow::Result<SkillDetail>
pub async fn get_version(c: &Client, ns: &str, slug: &str, ver: &str) -> anyhow::Result<VersionDetail>
pub async fn download_skill(c: &Client, ns: &str, slug: &str, ver: &str) -> anyhow::Result<Bytes>
pub async fn search_skills(c: &Client, q: &str, sort: &str, page: u32, size: u32) -> anyhow::Result<SkillPage>
pub async fn whoami(c: &Client) -> anyhow::Result<serde_json::Value>
pub async fn publish(c: &Client, ns: &str, visibility: &str, archive: Bytes, slug: &str) -> anyhow::Result<()>
```

### 2.5 agents.rs — Agent 目录

```rust
pub fn all_agents() -> Vec<AgentConfig>
pub fn get_agent(name: &str) -> anyhow::Result<AgentConfig>
// 计算 global_skills_dir 时读取环境变量（CLAUDE_CONFIG_DIR、CODEX_HOME 等）
// openclaw 按顺序检查 ~/.openclaw、~/.clawdbot、~/.moltbot
```

### 2.6 install.rs — install / remove

```rust
// install 主流程
pub async fn install_skill(
    client: &Client,
    slug: &str,
    namespace: &str,
    agent: &AgentConfig,
    mode: &str,                     // "global" | "project"
    version: Option<&str>,
) -> anyhow::Result<()>

// 解压 ZIP，校验路径安全
fn extract_zip(data: &[u8], target_dir: &Path) -> anyhow::Result<()>
fn is_safe_path(path: &str) -> bool     // 禁止 ".." 和空段

// remove
pub fn remove_skill(slug: &str, agent: &AgentConfig, mode: &str) -> anyhow::Result<()>

// _meta.json 读写
pub fn write_meta(dir: &Path, meta: &SkillMeta) -> anyhow::Result<()>
pub fn read_meta(dir: &Path) -> anyhow::Result<Option<SkillMeta>>
```

### 2.7 update.rs

```rust
pub struct UpdateCandidate {
    pub slug: String,
    pub namespace: String,
    pub local_published_at: String,
    pub remote_published_at: String,
}

pub async fn find_updates(
    client: &Client,
    agent: &AgentConfig,
    mode: &str,
) -> anyhow::Result<Vec<UpdateCandidate>>

// ISO 8601 字典序比较
fn is_newer(remote: &str, local: &str) -> bool
```

### 2.8 publish.rs

```rust
pub fn zip_skill_dir(skill_dir: &Path) -> anyhow::Result<Bytes>

pub async fn publish_skill(
    client: &Client,
    skill_dir: &Path,
    namespace: &str,
    visibility: &str,
) -> anyhow::Result<()>

// 递归发现 SKILL.md
pub fn discover_skills(root: &Path) -> Vec<PathBuf>
```

### 2.9 add.rs

```rust
pub fn parse_github_source(src: &str) -> anyhow::Result<(String, String)>  // (owner, repo)

pub async fn fetch_github_zip(owner: &str, repo: &str) -> anyhow::Result<Bytes>

pub fn discover_github_skills(zip_data: &[u8]) -> anyhow::Result<Vec<GitHubSkill>>
pub struct GitHubSkill { pub name: String, pub description: String, pub files: Vec<(String, Vec<u8>)> }

pub fn install_github_skill(skill: &GitHubSkill, agent: &AgentConfig, mode: &str) -> anyhow::Result<()>
```

---

## 3. gkill-cli 设计

### 3.1 main.rs — clap 命令树

```rust
#[derive(Parser)]
#[command(name = "gkill", version)]
enum Cli {
    Install(InstallArgs),
    Update(UpdateArgs),
    Search(SearchArgs),
    Publish(PublishArgs),
    Add(AddArgs),
    Remove(RemoveArgs),
    Login(LoginArgs),
    Logout,
    Whoami(WhoamiArgs),
}
```

### 3.2 公共参数约定

- `--registry <url>`：默认读 `DEFAULT_REGISTRY`
- `--agent <name>`：未传则 dialoguer::Select 交互选择
- `--mode global|project`：未传则 dialoguer::Select 交互选择
- spinner：每个网络操作使用 `indicatif::ProgressBar` 显示

### 3.3 search TUI（ratatui）

状态机：
```
Loading → Displaying → Installing
              ↑↓ ↑↓ (navigate)
              ←→    (page)
             Enter  (install)
              q/Esc (quit)
```

布局：
```
┌─ gkill search ──────────────────────────────────┐
│ > [query]                              Page 1 / 5 │
├───────────────────────────────────────────────────┤
│ ▶ skill-name        namespace    ↓1234  ★ 4.5     │
│   skill-name-2      global       ↓567              │
│   ...                                              │
├───────────────────────────────────────────────────┤
│ ↑↓ navigate  ←→ page  Enter install  q quit       │
└───────────────────────────────────────────────────┘
```

---

## 4. 依赖版本（Cargo.toml）

```toml
[workspace.dependencies]
clap       = { version = "4", features = ["derive"] }
reqwest    = { version = "0.12", features = ["multipart", "json"] }
tokio      = { version = "1", features = ["full"] }
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
zip        = "2"
ratatui    = "0.29"
crossterm  = "0.28"
dialoguer  = "0.11"
indicatif  = "0.17"
dirs       = "5"
anyhow     = "1"
thiserror  = "2"
bytes      = "1"
```

---

## 5. 错误处理策略

- 所有 `gkill-core` 函数返回 `anyhow::Result<T>`
- CLI 层统一捕获：非 0 退出码 + 红色错误信息
- `HttpError` 实现 `std::error::Error`，403 单独提示权限问题
- 用户取消交互（Ctrl+C）→ 正常退出 0

---

## 6. 测试策略

| 层 | 测试方式 |
| --- | --- |
| config.rs | 单元测试（tmpdir） |
| install.rs（extract_zip / is_safe_path） | 单元测试 |
| update.rs（is_newer） | 单元测试（纯函数） |
| publish.rs（zip_skill_dir / discover_skills） | 单元测试（tmpdir） |
| add.rs（parse_github_source / discover_github_skills） | 单元测试 |
| API 调用 | 集成测试（mockito 或手动对 skills.gydev.cn） |
