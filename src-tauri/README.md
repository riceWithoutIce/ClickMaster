# Tauri / Rust 后端

根级 `src-tauri/` 与前端 `src/` 并列，保持 Tauri CLI 常用发现布局。Cargo/Tauri 配置与锁文件已接入 TASK-001 工程。

| 位置 | 内容 |
| --- | --- |
| [src/](src/README.md) | Rust 入口、窗口/托盘生命周期与偏好保存；输入功能后续实现 |
| [capabilities/](capabilities/README.md) | Tauri 能力与权限声明 |
| [icons/](icons/README.md) | 用户提供的应用图标及可重建平台资源 |

`Cargo.toml` 固定核心依赖，`Cargo.lock` 固定完整依赖图；桌面构建禁止安装器打包。

统一脚本按进程设置 `CARGO_TARGET_DIR` 到仓库内 `build/cargo/`，依赖缓存为 `build/cache/cargo-home/`；不改用户级 Cargo 配置。`gen/` 是生成目录，默认忽略。

生产运行时数据在 EXE 同级 `data/`；开发为 `build/runtime/dev/`，测试仅在 `build/verification/` 下运行。所有持久内容遵循 [脱敏约定](../aiwork/workflow/sanitization.md)。
