# gkill 的 cargo-dist + GitHub Actions 发布方案

## 目标

为 `gkill` 提供自动化发布能力：

- 推送版本 tag 后，自动构建多平台二进制
- 自动生成安装脚本（shell + powershell）
- 自动创建 GitHub Release 并上传产物
- 仅发布 CLI（`gkill`），不发布 Tauri 桌面应用

## 本次已完成的实现

已在仓库中落地以下改动：

1. `dist` 初始化配置
2. 生成 GitHub Actions 发布流水线
3. 配置仅发布 `gkill`
4. 输出本操作文档

## 关键文件

- `.github/workflows/release.yml`
- `dist-workspace.toml`
- `crates/gkill/Cargo.toml`
- `crates/gkill-app/Cargo.toml`
- `Cargo.toml`（由 `dist init` 增加了 `[profile.dist]`）

## 配置说明

### 1) dist 工作区配置

文件：`dist-workspace.toml`

当前配置要点：

- `cargo-dist-version = "0.31.0"`
- `ci = "github"`
- `installers = ["shell", "powershell"]`
- `targets = ["aarch64-apple-darwin", "x86_64-apple-darwin", "x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc"]`
- `packages = ["gkill"]`
- `install-path = "CARGO_HOME"`
- `pr-run-mode = "plan"`

### 2) 只发布 CLI

文件：`crates/gkill-app/Cargo.toml`

已显式设置：

```toml
[package.metadata.dist]
dist = false
```

这会确保桌面端 `gkill-app` 不进入 dist 发布图。

### 3) 包元数据

`gkill` 和 `gkill-app` 均已增加：

- `description`
- `readme`
- `repository`
- `license`

当前 `repository` 已设置为：

```toml
repository = "https://github.com/Cocola6s6/gkill.git"
```

如需迁移仓库，请同步更新该地址，否则 Release 页面源码链接会不准确。

## 发布流程（开发者）

### 0) 首次准备

确保仓库已推送到 GitHub，且有写 Release 权限。

### 1) 提交改动

```bash
git add Cargo.toml dist-workspace.toml crates/gkill/Cargo.toml crates/gkill-app/Cargo.toml .github/workflows/release.yml docs/cargo-dist-github-actions-guide.md
git commit -m "chore: add cargo-dist github release pipeline"
git push origin <your-branch>
```

### 2) 合并到主分支

将分支合并到 `main`（或你的默认发布分支）。

### 3) 打 tag 触发发布

```bash
git tag v0.1.0
git push origin v0.1.0
```

触发后，GitHub Actions `Release` workflow 会自动：

1. 规划构建矩阵（dist plan/host create）
2. 构建各平台产物
3. 生成全局安装器与校验信息
4. 创建 GitHub Release 并上传所有产物

## 用户下载安装方式

发布成功后，用户可通过以下方式安装：

1. 进入 GitHub Releases 页面下载对应平台压缩包/可执行文件
2. 使用 dist 产出的 shell 安装脚本（macOS/Linux）
3. 使用 dist 产出的 powershell 安装脚本（Windows）

具体安装命令会在对应 Release 页面给出。

## 验证命令

本地可用以下命令验证配置有效性：

```bash
# 查看发布计划
cd gkill
dist plan

# 仅检查 CI 文件是否最新
dist generate --mode=ci --check

# 预览 manifest（JSON）
dist manifest --output-format=json
```

## 常见问题

1. `Github CI support requires you to specify the URL of your repository`
- 原因：crate 缺少 `repository` 字段
- 处理：在可发布 crate 的 `Cargo.toml` 增加 `repository = "https://github.com/Cocola6s6/gkill.git"`

2. 不想发布某个包（如桌面端）
- 在该包 `Cargo.toml` 增加：

```toml
[package.metadata.dist]
dist = false
```

3. 需要增加发布平台
- 修改 `dist-workspace.toml` 的 `targets`
- 然后执行：`dist generate --mode=ci`

## 后续建议

1. 将 `repository` 占位地址替换为真实 GitHub 仓库地址
2. 在 GitHub Releases 首次发布 `v0.1.0`，确认产物命名与安装体验
3. 若要进一步提升用户体验，可追加 `homebrew` installer
