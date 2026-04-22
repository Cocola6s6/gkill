# Phase 2：Tauri 桌面应用（gkill-app）

## 目标
基于 Tauri v2 + Sycamore（Rust WASM）构建桌面图形应用，复用 Phase 1 的 gkill-core 核心库。

## 技术栈
| 层 | 技术 |
| --- | --- |
| 应用框架 | Tauri v2 |
| 前端语言 | Rust |
| 前端框架 | Sycamore（响应式，编译为 WASM） |
| 前端构建 | Trunk |
| 后端逻辑 | 复用 `gkill-core` |

## Workspace 扩展
```
gkill/
├── Cargo.toml
├── crates/
│   ├── gkill-core/
│   ├── gkill-cli/    # 阶段一：CLI 入口
│   └── gkill-app/        # 新增：Tauri 应用
```

## 任务拆分

### 2.1 Tauri + Sycamore 工程初始化
- 添加 `gkill-app` crate
- 配置 Tauri v2 + Trunk 构建链
- 配置 Sycamore 前端骨架

### 2.2 Tauri Commands 层
将 gkill-core 暴露为 Tauri invoke 命令：
```rust
search_skills(query, page) -> SkillPage
install_skill(slug, namespace, agent, mode) -> ()
list_installed(agent, mode) -> Vec<InstalledSkill>
remove_skill(slug, agent, mode) -> ()
publish_skill(path, namespace, visibility) -> ()
login(token) -> ()
logout() -> ()
whoami() -> UserInfo
```

### 2.3 搜索浏览页
- 搜索框 + 结果列表
- 分页
- 点击安装（调 install_skill command）

### 2.4 已安装管理页
- 按 Agent 展示已安装 skill 列表
- 检查更新 / 更新按钮
- 删除按钮

### 2.5 发布页
- 拖拽或选择目录
- 选择 namespace / visibility
- 上传进度展示

### 2.6 登录态管理
- 登录表单（输入 token）
- 显示当前用户（whoami）
- 登出

### 2.7 打包发布
- macOS（.dmg / .app）
- Windows（.msi / .exe）
- Linux（.AppImage / .deb）

## 前置条件
- Phase 1 完成，gkill-core API 稳定

## 验收标准
- [ ] 桌面应用可启动
- [ ] 搜索 + 安装流程可用
- [ ] 已安装列表可管理（更新 / 删除）
- [ ] 发布功能可用
- [ ] 登录态正常
- [ ] 三平台打包成功
