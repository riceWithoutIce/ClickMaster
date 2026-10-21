# 前端源码

按已确认的 Tauri 官方常用布局，`src/` 只承载 TypeScript、样式、组件及前端资源；Rust 后端位于根级 [src-tauri/](../src-tauri/README.md)。

前端工程配置、`package.json`、`index.html` 和包管理器锁文件后续放在仓库根目录，从仓库根执行工程命令。组件框架和包管理器尚未确定，本轮不生成空配置或应用代码。

圆角、主题和 UI 动效由前端负责；输入调度、目标身份校验及全局停止由 Rust 后端负责。计划将前端编译输出指向 `build/frontend/`，当前尚未配置或验证。

源码历史通过 Git 追溯，不保留按版本复制的代码、发布包或编译缓存。文件内容遵循 [脱敏约定](../aiwork/workflow/sanitization.md)。
