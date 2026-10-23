# TASK-002 R2 独立限定复审

日期：2026-10-21。对象：`TASK-002-R2-CODER-S1`。仅复审工具型顶层窗口排除 finding 及紧邻范围/身份保护。

## 结论

**原规则 finding 的代码修复、纯策略测试及紧邻前端检查通过，未发现 BLOCKER 或 FOLLOW-UP findings。** 真实目标出现及绑定仍须用户人工复验；本轮没有重新诊断、读取或操作真实目标。

## 快照与范围

- [42 文件清单](r2-source-manifest.sha256) SHA-256：`11B9A4A63A66BB311FB99443CA5F43BDEC36C2F9C390CA6FD9713EC620FCFADE`。
- 开始/结束原始 42 文件均 0 mismatch，结束时隔离副本亦 42 文件 0 mismatch。
- 对 R1 恰好 4 文件变化：`src-tauri/src/windows_query.rs`、`src-tauri/src/README.md`、根 `README.md` 和 `src/App.vue`。其中 App.vue 仅补充“含工具型窗口”说明，未更改前端行为。
- Windows 查询生产逻辑唯一变化为工具条件从 `!window.tool` 改为 `(include_hidden || !window.tool)`；默认范围仍排除工具窗口，明确开启扩展范围后才允许。
- owner/APPWINDOW、NOACTIVATE、自身/系统排除及其 Win32 读取接线未变；身份状态、绑定前/周期验证、隐藏保持、筛选、托盘、配置、锁文件、图标资源均保持 R1 哈希。
- 最终入口：`build/verification/TASK-002/coder/r2/final/ClickMaster.exe`，9,063,424 bytes；SHA-256 `F724722928E50E16846D375AAAAA66027EEA9A0C04CA069ED379BFF3764FF0AC`，与交接一致；核验时同级无 `data/`。

## 实际独立验证

隔离根 `build/verification/TASK-002/verifier/r2/source/` 复制冻结清单及项目现有前端依赖，不安装依赖或复制 Rust 缓存。通过副本包装入口执行以下两项，均退出 0：

```powershell
./scripts/npm.ps1 run typecheck
./scripts/npm.ps1 run build
```

前端构建 Vite 7.3.1、18 模块，约 620 ms，JS 86.68 KB、CSS 7.49 KB；没有启动浏览器或服务器。

从冻结 `windows_query.rs` 按明确边界原样提取 `#[cfg(test)]` 测试模块、`WindowProperties`、`eligible` 和 `display_status`，仅补无外部依赖的 `WindowStatus` 枚举，以 `rustc --test` 独立编译运行。提取不含 Win32 adapter/import/candidate/inspect，因此不触发真实窗口查询；使用生产规则及原测试断言，未重写或替换规则。纯策略源/二进制为本轮 verifier/r2 下 `pure-window-rules.rs` / `.exe`，TEMP/TMP/TMPDIR 同在此目录内，使用路径重映射。

3 项测试均通过，0 failed：

| 原测试 | 独立结果 |
| --- | --- |
| 隐藏/无标题范围与一般辅助排除 | PASS |
| 扩展范围允许 tool，但 self/owned无APPWINDOW/noactivate/system 与 tool 组合仍排除；owned+APPWINDOW工具窗口仅扩展允许 | PASS |
| 可见/最小化/隐藏状态区分 | PASS |

上述测试同时确认默认范围仍拒绝 tool，扩展范围允许隐藏、空标题工具型窗口。源码对照确认不增加名称特判、子窗口、message-only 或全进程枚举，也不弱化可信 token/PID/创建时间/线程身份校验。

[Coder 交接](r2-coder-handoff.md) 中完整 22 项 Rust 测试、生产桌面构建与 PE 图标校验属 Coder 实跑证据；本次未重复 Rust release、完整 22 项测试或 PE 校验，不声称这些是 Verifier 独立执行。前轮夹具、final 和运行数据未改写。

## Findings、人工门槛与停止

本轮未发现 findings，不向 Coder 追加修复；限定审核到此结束。用户仅需从本轮最终入口复验：开启扩展范围后示例目标能出现、筛选和绑定；默认范围继续排除工具型窗口。纯策略通过及上轮诊断不能替代新 EXE 的真实绑定证据，也不证明后台输入或游戏响应。

未启动应用/浏览器/GUI夹具/开发服务器，未操作桌面、查询真实窗口、清系统缓存或终止外部进程。所有新增产物/缓存均在 verifier/r2 项目内；报告仅记录相对路径和脱敏摘要，不保存真实窗口信息或原始日志。未做系统级外写监控或完整二进制审计，不扩大已有证据承诺。
