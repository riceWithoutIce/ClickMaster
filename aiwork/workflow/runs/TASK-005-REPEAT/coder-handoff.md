# TASK-005-REPEAT Coder 交接

日期：2026-10-22。状态：实现、自测、正式模式测试包及清单核验完成，源码冻结，可进入独立 Verifier。无普通 UI、真实游戏查询/管道传输、实际 F8 注册或桌面输入；无提交、推送、发布。

## 实现

- 新增 `src-tauri/src/repeat.rs`：闭区间整数校验、无偏拒绝采样、固定值不使用随机源、单调时间溢出拒绝。两通道默认 2000/2000 毫秒；消息100～60000、IPC2000～60000；前后端拒绝非法值，不夹边。会话草稿不写设置，单次独立于重复参数。
- `input.rs`、`input_runtime.rs`：消息保留原身份配对释放，3秒后首轮，完整释放后才采样下次等待；不补发、不重叠。身份/路由失效及失败停止。计数为完整投递次数，安全上限到达前不再发送。显示准备、操作、等待、停止、终态和下一次等待。
- `ipc_runtime.rs`：每帧仍创建直接拥有的受控同 EXE Child，单帧写入及读取确认后才能等待下一帧；60秒仅为各次传输预算，持续任务无60秒会话上限。保留版本/原实例/后台窗口/真实客户端、当前用户ACL、本地first-instance、一次匿名stdin授权和自有Child停止边界。无数量N批量协议、重试或切通道。
- 新增 `hotkey.rs`：F8 RegisterHotKey、消息循环及注销在同一可靠线程；收到注册成功后才能执行，冲突拒绝开始。MOD_NOREPEAT，不使用 hook。按钮、托盘和 F8 进入统一停止，异常服务退出可检测；准备、单次、重复期间生效，终态释放。
- `lib.rs`：共用单次/重复准入、busy锁定、热键生命周期和统一取消。托盘停止按实际任务启用，tooltip显示单次/重复/停止中/待命及完成次数。托盘更新异步派发，避免持锁同步等待 UI 与退出 join 互相阻塞；收起不停止，实际退出先清理自有任务。
- `src/App.vue`、`input-channel.ts`、`single-input.ts`：区间参数、前端校验、开始/统一停止、状态及次数。原左右键/键盘/坐标草稿、通道选择、单次和配置保持；运行时锁定参数、目标和通道。`scripts/test-target-filter.mjs` 补生产前端策略回归。

## 有界存储与清理取舍

新增 `ipc_storage.rs`。每个活动任务只创建一个独立随机槽位；每轮在直接持有的 Child 已确认退出后，复用此槽位固定的授权/claim/状态文件和 tmp，不按帧新增目录或日志。正常终态移除本轮创建的已知文件及空目录，不保存会话历史（历史上限0）；当前结果保留在内存供 UI 显示。

清理只针对本轮直接创建槽位的已知文件，不扫描删除旧任务目录、用户 data 或未知临时文件。无法确认 worker 退出或出现未知/占用文件时停止且保持锁定，不继续创建后续帧槽位。意外进程崩溃或清理未确认可能留下该次槽位，后续启动不会自动清扫这些残留；它们不能当作持续运行时无限新增历史的理由。

## 验证范围与结果

执行目录均为仓库根，进程环境由 `scripts/environment.mjs` 给出项目内 TEMP/TMP/TMPDIR、Cargo/npm/Node缓存、target及路径remap；Cargo离线锁定，无新增依赖、下载或 .NET 分发。

| 检查 | 方法及证据（本任务 coder 目录） | 状态 |
| --- | --- | --- |
| 编译检查 | 项目环境 `cargo check --release --locked --offline --manifest-path src-tauri/Cargo.toml` | PASS |
| 首轮后端 | `node scripts/project.mjs test`，tests-r1 | 58 passed / 1 ignored；test-r1.log |
| 调度补充回归 | 同上，tests-r2 | 61 passed / 1 ignored；test-r2.log |
| 最终后端 | 同上，tests-final | 61 passed / 0 failed / 1 ignored；final-test.log |
| 热键最终紧邻回归 | 离线 `cargo test --release --locked --offline --manifest-path src-tauri/Cargo.toml hotkey::tests` | 2 passed / 0 failed；final-hotkey.log |
| 前端 | `node scripts/project.mjs typecheck`、`test-targets`、`build` | PASS；typecheck.log、test-targets.log、build.log |
| 正式模式单 EXE | `node scripts/project.mjs desktop`（--no-bundle） | PASS；final-desktop.log |
| 最终包图标 | `scripts/verify-pe-icon.mjs` 对照源 ICO | PASS；final-icon.log |
| 未授权 worker 入口 | 最终包仅使用两组 `--ipc-worker` 非授权参数启动 | 均退出64，无stdout/stderr，无data生成；worker-entry-checks.json |
| 旧证据保留 | 开发前216文件逐项hash复核 | 全部未变；old-evidence-preservation.json |
| 已知本机前缀 | 当前源码文本、最终二进制UTF-8/UTF-16扫描 | 0匹配；sanitization.json；不声称穷尽隐私检查或OS零外写 |

ignored 项是由父测试以独立子进程调用的 fixture 入口，非略过的验收要求。原消息配对释放/路由、IPC帧/身份/超时、配置未知字段保留和托盘策略紧邻回归保留。新验证覆盖：默认/边界/非法区间、固定值无需随机源、拒绝值及均匀分布、完成后等待/延迟不补发、单次互斥、准备/按下后/等待停止及身份/部分投递终止、实际完成时钟和安全上限、持续调度超过60秒、fake热键成功/冲突/dispatch/终态释放、20轮槽位复用及不触及原数据。原非游戏管道fixture继续验证单帧/first-instance、连接超时、自有Child与无关Child隔离、阻塞drain时停止。

本轮没有测试失败；初始编译的未用项诊断在最终编译前移除。最终常规回归后仅补充热键线程实际存活判断，并以最终热键紧邻回归核对；早期结果不代替最终清单。

## 人工和独立验收边界

未启动独立 Verifier。待冻结后按实际源码/配置/锁文件与最终包清单审核，不用HEAD代替dirty快照。真实游戏持续响应、物理F8/冲突、主界面和托盘停止、目标退出/程序实际退出/托盘继续、随机节奏、完整工作输入/鼠标/焦点隔离、单次与计算器内容路由由用户按 human-test.md 验收；fake服务和非游戏管道不能替代这些证据。投递/读取次数不代表游戏计数或玩法响应。

原 TASK-004包和旧 IPC证据开发前已记录 `old-evidence-baseline.json`；最终216文件逐一复核均未变。保留所有无关dirty文件及用户数据。不包含覆盖层修复、其他游戏适配、参数持久化或自定义热键。

## 稳定快照

快照 `TASK-005-REPEAT-coder-final`；`snapshot.json` 记录实际57份源码、配置、锁文件清单和最终包。HEAD `d3d8f58ba8490212a16f33faecb4b5533d79bd5d` 仅为参考，不能代替本次dirty快照。

- 清单：`build/verification/TASK-005-REPEAT/coder/source-manifest.json`，SHA-256 `D1CA33BC0A2AB47F2C0FCBBF4390E29F79B7CEA1FB66BF4E1EDC3A5B7005753D`。
- 测试包：`build/verification/TASK-005-REPEAT/coder/final/ClickMaster.exe`，9,423,360字节，SHA-256 `97A7B2FC9F844E6F7BA64824FE7DCCED7209EFDB6647A89B0DF3C21655E8676D`。final目录仅此文件，未生成运行data。
- 清单、复制、worker拒绝、图标及已知前缀扫描由 `build/verification/TASK-005-REPEAT/coder/finalize.mjs` 执行并退出0。此辅助脚本会写Coder输出，Verifier只读清单和重新计算hash，不重跑该脚本。
- Coder交接后不再修改被验源码，Verifier开始及结束须复核57文件与包hash。测试日志/文档位于各自职责目录，不以Git HEAD覆盖归属性。
