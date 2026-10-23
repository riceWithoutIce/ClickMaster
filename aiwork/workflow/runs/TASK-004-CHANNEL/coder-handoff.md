# TASK-004-CHANNEL Coder 稳定交接

日期：2026-10-22。角色：任务 Coder。状态：实现、自测、本地测试包完成，源码冻结；尚未启动独立 Verifier。无提交、推送或正式发布。

## 稳定快照

- 标识：`TASK-004-CHANNEL-coder-final`。源码/配置/锁文件实际内容清单：`build/verification/TASK-004-CHANNEL/coder/source-manifest.json`，54 文件。
- 清单 SHA-256：`0AA24949D0915A816E9197BAD8F9D00AB1F44C2FBD6CDE8DE4CA8865A99C04C3`。
- HEAD 仅作参考：`d3d8f58ba8490212a16f33faecb4b5533d79bd5d`。本仓库大量既有未提交/未跟踪内容保留，不能使用 HEAD 代替上述实际快照。
- 本地测试包：`build/verification/TASK-004-CHANNEL/coder/final/ClickMaster.exe`，9,364,992 字节，SHA-256 `AE4E289C0DB31D0C5C35DD41421800A1B4EC5DBF092EC79EF354703375E4FCD1`。目录仅包含此 EXE，不含用户 data，也不含 .NET sidecar。
- 机器可读摘要：`build/verification/TASK-004-CHANNEL/coder/snapshot.json`。提交交接后不再修改被验源码；独立检查输出应落到 verifier 目录。

## 实现

- 新增 `src-tauri/src/channels.rs`、`ipc.rs`、`ipc_native.rs`、`ipc_worker.rs`、`ipc_runtime.rs`：会话通道、固定单帧、原生身份与传输、受控同 EXE worker、单调监督分别拆分。
- `lib.rs` 接入通道命令、共同忙碌门槛、IPC 单次准备/停止/退出；IPC 不要求消息路由就绪。窗口消息原配对释放及托盘规则保留。`input.rs` 统一递增结果版本与通道/传输回执；`input_runtime.rs` 仅调度消息任务。`targets.rs` 仅增加身份 DTO 序列化，以匿名 stdin 传给自有 worker，不落盘原始身份。
- `Cargo.toml` 仅启用现有 windows-sys 0.61.2 的安全/管道/IO/进程枚举功能；未更改依赖版本，无下载、安装或 .NET 依赖。
- `src/App.vue`、`single-input.ts`、新增 `input-channel.ts`：顶部下拉、IPC 原因和专用说明、隐藏键鼠/坐标与路由控件、会话消息草稿保留、忙碌锁定、实际通道及分项连接/写/读取状态。`scripts/test-target-filter.mjs` 增加生产通道策略纯回归。
- 默认每次启动窗口消息；不新增配置键。IPC 选中后失配仍保留 IPC，禁止发送，不回退。兼容核验只读；显式单次才创建游戏管道。仅发送既有四字节一帧，未开发重复、F8、其他游戏适配或覆盖层修复。
- 原实例持有进程句柄、创建时间/EXE/版本 SHA、唯一 Unity 主窗口/客户区、后台非最小化及实际客户端 PID 核验。当前用户 DACL、本地客户端、first-instance、max 1。worker 启动秘密通过匿名 stdin，校验父进程实例/相同 EXE、包哈希、一次 claim；入口失败不初始化 UI。
- 3 秒准备，连接20秒、写2秒、读取10秒、总60秒；单调时间，无墙钟换算。阻塞 FlushFileBuffers 留在自有 worker，停止只操作直接保留的 Child 句柄，不按名称/PID/树终止外部进程。无法确认清理保持锁定；父进程退出后 worker 自行终止。取消不能撤回已发帧。

## 自测证据

工作目录均为仓库根。统一 `scripts/environment.mjs` 配置项目内 TEMP/TMP/TMPDIR、Cargo/npm/Node 缓存和 Rust remap；测试根显式为本任务 `coder/tests-r1` 或 `coder/tests-r2`。Cargo 设置 `CARGO_NET_OFFLINE=true`，锁文件锁定。只运行独立非游戏随机管道 fixture，不查询或连接实际游戏。

| 检查 | 命令/方法 | 结果与证据（coder 目录下） |
| --- | --- | --- |
| Rust 检查 | 项目环境 `cargo check --release --locked --offline --manifest-path src-tauri/Cargo.toml` | PASS；首次发现同步权限常量名称错误后修正，未产生运行副作用 |
| 后端首轮 | `node scripts/project.mjs test` | 50 passed / 1 ignored；`test-r1.log` 保留 |
| 最终后端 | 同上，新测试根 tests-r2 | 52 passed / 0 failed / 1 ignored；`test-r2.log`。ignored 为由父测试单独启动的 fixture 入口，非未执行验收项 |
| 前端类型 | `node scripts/project.mjs typecheck` | PASS，`final-typecheck.log`（退出0，无诊断） |
| UI 策略回归 | `node scripts/project.mjs test-targets` | PASS，`final-test-targets.log`。IPC 不受消息路由误阻塞、版本/绑定 epoch 失配禁用、不回退、传输结果措辞；保留原筛选/坐标/消息/路由回归 |
| 离线正式构建 | `node scripts/project.mjs desktop` | PASS，`desktop.log`。Tauri --no-bundle，单 EXE，无依赖下载 |
| 原生 EXE 图标 | `scripts/verify-pe-icon.mjs` 解析最终 EXE 资源 | PASS，`final-icon.log`；资源与现有指定 ICO 一致 |
| worker 拒绝 | 最终 EXE 仅以 --ipc-worker 无授权/多余参数运行 | 均退出64，无普通 UI、无 stdout/stderr、无 data 创建；`worker-entry-checks.json` |
| 脱敏 | 源码/配置文本、最终 EXE UTF-8/UTF-16 已知项目/用户前缀扫描 | 0 命中，`sanitization.json`；不声称全系统零外写或穷举所有敏感数据 |
| 旧证据保持 | 开发前188文件 hash，交接前逐一重算 | 全部不变；`old-ipc-baseline.json`、`old-ipc-preservation.json` |

独立 fixture 覆盖：一次授权/错误秘密拒绝、随机非游戏管道 first-instance 排他和四字节单次传输、连接有限超时、只终止自有 Child 且另一独立 Child 保持运行、真实阻塞 drain 被自有 worker 停止。纯回归覆盖通道默认/不回退/忙碌、帧协议、各阶段单调边界、原进程创建时间和名称门槛；原消息/路由/配置未知字段保留/托盘纯测试均通过。具体真实目标枚举/版本/窗口查询没有在本轮开发中对游戏执行。

## 待验与限制

- 未打开普通 UI；新 Rust 路径尚无真实游戏/桌面证据。主程序单次玩法响应、停止/关闭/目标退出/托盘、工作窗口输入/真实鼠标/焦点隔离及计算器紧邻回归按 human-test.md 由用户验收。旧独立 C# 工具人工通过不能替代本包。
- IPC 连接/写入/读取是传输证据，不代表玩法、统计或成就等价。通道特定单次仅适配冻结 Assembly-CSharp SHA；其他 EXE 保留消息通道。
- 不新增重复调度、全局热键或通道跨重启记忆；OVERLAY-001 继续延期。本包仍依赖既有 WebView2 运行前提，不是正式 release。
- 首次 Rust 编译检查发生过常量命名诊断，已修正并通过最终测试/构建；没有测试失败或游戏尝试被隐藏。首轮测试后仅补契约必要回归及消息 runtime 通道隔离，最终快照以清单为准。
- 校验入口辅助脚本 `build/verification/TASK-004-CHANNEL/coder/finalize.mjs` 会复制已有构建、运行拒绝入口并写 coder 证据；Verifier 不需运行它，可直接只读重算清单和 EXE hash，独立输出写 verifier/。不要运行普通 EXE 或实际游戏传输替代用户验收。
