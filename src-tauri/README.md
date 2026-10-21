# Tauri / Rust 后端

根级 `src-tauri/` 与前端 `src/` 并列，保持 Tauri CLI 常用发现布局。当前只建立目录说明，未初始化 Cargo 或 Tauri 工程。

| 位置 | 内容 |
| --- | --- |
| [src/](src/README.md) | 后续 Rust 入口、绑定、Windows 消息、调度和停止实现 |
| [capabilities/](capabilities/README.md) | 后续 Tauri 能力与权限声明 |
| [icons/](icons/README.md) | 后续应用图标源资产 |

`Cargo.toml`、`Cargo.lock`、`build.rs`、`tauri.conf.json` 及 Rust 入口待工程初始化时生成；不以空文件冒充有效工程。

后续构建按进程设置 `CARGO_TARGET_DIR` 到仓库内 `build/cargo/`，前端资源路径与 `build/frontend/` 对齐；不改用户级 Cargo 配置。当前未接通构建，`gen/` 为未来生成目录，默认忽略。

运行时配置、日志和 WebView 用户数据不写入源码或 release 目录，具体数据位置仍待设计。所有持久内容遵循 [脱敏约定](../aiwork/workflow/sanitization.md)。
