# gkill

## 项目目标

本项目的目标是实践并验证通过 `spec-to-delivery-workflow` skill，完成从需求到交付的全自动开发流程。

核心关注点：

- 按阶段推进需求拆解、设计、实现、测试与验收
- 在真实代码库中沉淀可复用的 workflow artifacts
- 以 `gkill` 作为可运行产物验证端到端流程

## 目录说明

- `gkill/`：Rust 客户端项目（CLI + Desktop）
- `workflow-artifacts/`：流程产物（spec / design / progress / checkpoints）
- `skill客户端需求文档.md`：需求基线文档

## 发布策略

当前发布策略：

- GitHub Release 仅发布 CLI（`gkill`）
- Desktop 应用 `gkill-app` 不参与 dist 发布

对应配置位于：`gkill/dist-workspace.toml`

