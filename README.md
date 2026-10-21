# ClickMaster

轻量级 Windows 后台重复输入工具，技术方向为 Tauri 2 + Rust + TypeScript。支持选择应用窗口，以客户区坐标发送鼠标左键、右键或单个键盘按键消息。

当前只有需求、技术选型和目录基线，尚未实现或发布；目标应用的后台输入兼容性尚未验证。

| 目录 | 职责 |
| --- | --- |
| [src/](src/README.md) | 前端源码预留，工程配置后续位于仓库根 |
| [src-tauri/](src-tauri/README.md) | Rust/Tauri 后端、能力声明与图标预留 |
| [build/](build/README.md) | 编译缓存、前端产物、安装包中间结果和待发布暂存 |
| [version/](version/README.md) | 版本号、变更说明及可追溯发布索引，不存二进制 |
| [release/](release/README.md) | 当前正式版、有限历史归档及单个候选版 |
| [scripts/](scripts/README.md) | 后续构建、版本同步与发布管理脚本；当前没有脚本实现 |
| [aiwork/](aiwork/README.md) | 需求、路线、设计、验证及调试证据 |

目录与保留规则见 [项目目录和版本管理约定](aiwork/project-layout.md)。

开发协作流程见 [workflow](aiwork/workflow/README.md)，包含任务对齐、Coder 开发、Verifier 独立验收及收尾交付。

所有项目文件、文档和证据遵循 [脱敏约定](aiwork/workflow/sanitization.md)，使用相对路径或占位符，不保存本机路径或敏感内容。

Git 仓库：[riceWithoutIce/ClickMaster](https://github.com/riceWithoutIce/ClickMaster)。源码使用根级 `src/` + `src-tauri/` 布局；当前只有目录骨架和文档，尚无可运行的 Tauri 工程。
