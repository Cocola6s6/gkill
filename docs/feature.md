# 功能与修复清单

该文件是持续维护“性能优化 + Bug 修复 + 新功能”的唯一记录入口。
发现新问题或完成一项改动后，请及时更新本文件。

## 状态说明

- `TODO`：未开始
- `DOING`：进行中
- `DONE`：已完成并已验证

## 当前事项

### [DONE] 请求中保持 UI 可交互（不整页锁死）

- 类型：优化 + 交互问题修复
- 日期：2026-04-23
- 范围：
  - 搜索页在请求中保留已有结果，不再整页替换为 loading。
  - 用局部状态提示替代全页 loading（按钮和结果区显示“加载中”状态）。
  - 确保首次自动搜索只触发一次，避免输入时 effect 反复触发请求。
  - 发布页命名空间仅加载一次，避免重复渲染导致重复请求。
- 涉及文件：
  - `crates/gkill-app/ui/src/pages/search.rs`
  - `crates/gkill-app/ui/src/pages/publish.rs`
- 验证结果：
  - `crates/gkill-app/ui` 下 `cargo check` 通过。
  - 工作区 `cargo check -p gkill-app` 仍依赖已存在的 `ui/dist` 构建产物。

### [DONE] 修复页面切换后异步回写导致的 `signal was disposed` 崩溃

- 类型：bug修复
- 日期：2026-04-23
- 背景：
  - 页面切换后，异步请求返回仍写入已销毁的组件 signal，触发 `signal was disposed` panic。
- 范围：
  - 为 `search / publish / installed / settings` 页面增加组件生命周期保护。
  - 组件卸载后阻断本地 signal 的异步回写。
- 涉及文件：
  - `crates/gkill-app/ui/src/pages/search.rs`
  - `crates/gkill-app/ui/src/pages/publish.rs`
  - `crates/gkill-app/ui/src/pages/installed.rs`
  - `crates/gkill-app/ui/src/pages/settings.rs`
- 验证结果：
  - `crates/gkill-app/ui` 下 `cargo check` 通过。

## 新增事项模板

每次新增优化或修复时，按以下模板补充：

```md
### [TODO] <事项标题>

- 类型：优化 | bug修复 | 功能
- 日期：YYYY-MM-DD
- 背景：
- 目标范围：
- 实施计划：
- 验证方式：
- 涉及文件：
- 备注：
```
