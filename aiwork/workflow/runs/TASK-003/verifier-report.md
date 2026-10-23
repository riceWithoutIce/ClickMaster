# TASK-003 独立限定复审

日期：2026-10-21。对象：`TASK-003-CODER-S1`。依据：[任务契约](task.md)、[Coder 交接](coder-handoff.md)、[50 文件清单](source-manifest.sha256)。范围仅单次后台输入、紧邻绑定转换/退出与前端状态；不展开原生预览问题或无关全面审计。

## 结论与快照

**本轮源码审查及独立非 UI 检查通过，未发现 BLOCKER 或 FOLLOW-UP findings。** 这不构成真实 Win32 输入或游戏兼容性验收。用户正在使用电脑，实际应用、消息接收、隔离及焦点由用户人工验证。

- 清单 SHA-256：`3bcc7e67a1282933f6610a204b26a588f1d5d104cb67979aa88557a9016a73ee`。原始 50 文件开始/结束均 0 mismatch，独立副本结束亦 50 文件 0 mismatch。
- 最终入口：`build/verification/TASK-003/coder/final/ClickMaster.exe`；9,222,144 bytes；SHA-256：`3bcf9f5f12ed69dd208e9be35445c0ce3640da1fdda1a9635853153b7f33e8b9`，与交接一致，核验时同级无 `data/`。
- 本次输出：`build/verification/TASK-003/verifier/`，包含 `run.mjs`、脱敏日志、`evidence.json`、纯测试源及二进制、隔离源码及前端输出。未覆盖旧任务、Coder 产物或历史报告。

## 独立实跑证据

在清单文件的独立副本运行，复用现有 `node_modules` 只读用途的目录联接，未安装或升级依赖。TEMP/TMP/TMPDIR、Node/npm/Cargo 缓存及编译路径显式设为本次 verifier 目录；前端构建输出在副本内。没有服务器或浏览器。

| 检查 | 结果 |
| --- | --- |
| vue-tsc `--noEmit` | PASS，退出 0 |
| 现有前端 helper 测试 | PASS，筛选/坐标转换/42 项单键选项与默认空格/倒计时取整/partial 及错误码文案 |
| Vite 生产前端构建 | PASS，20 模块，JS 91.04 KB / CSS 7.81 KB |
| 独立 `input.rs` 纯模块 harness | PASS，原有 7 项测试全部通过，0 failed |
| 50 文件与最终 EXE 核验 | PASS，原始及副本无 mismatch，大小及哈希符合交接 |

纯模块 harness 保留冻结 `input.rs` 的生产逻辑和全部原测试断言，仅移除 serde import、derive 中的 Serialize/Deserialize 与 serde rename 属性；从冻结 `targets.rs` 原样提取 Identity 定义补入。使用 `rustc --test --edition=2021` 和仓库路径重映射编译；不链接原生适配层，不创建或查询窗口。它验证 fake adapter/人工时间下的逻辑，不验证 serde IPC、真实键盘布局映射、原生 API 或线程锁竞争。

7 项测试覆盖左右键消息/坐标/有符号 16 位边界；键白名单、扫描码及 extended/release 位；3 秒截止、50 ms 释放与拒绝重复排队；快照不跟随草稿；取消/退出语义、重绑/身份变化、准备后及释放前再验证；最小化拒绝、按下失败和释放 partial、保留错误码；终态不重复投递。

完整 34 项 Rust 测试、Windows x64 release 桌面构建与 PE 图标核验属于 Coder 实跑证据，本次未重复完整 Rust/release/PE 检查；不将其计作 Verifier 独立运行结果。

## 源码核对

| 路径 | 结论与实际边界 |
| --- | --- |
| 消息构造 | STATIC / PURE PASS：左右键分别为 DOWN/UP 对及正确按钮 wParam，释放复用原坐标；键盘 WM_KEYDOWN/UP，repeat=1，扫描码低 8 位及 E0 extended，释放 previous/transition 位正确。严格 42 项字符串白名单，IPC 不接受原始 VK、消息或目标句柄决定投递。 |
| 原生适配 | STATIC PASS：只向可信绑定 HWND 调用 PostMessageW；GetKeyboardLayout 使用目标线程，MapVirtualKeyExW 使用 VK_TO_VSC_EX；客户区读取临时设置并恢复线程 DPI 上下文，拒绝最小化、不可读/非正客户区或非法鼠标坐标。不要求前台/可见，不调用 SendInput、设备键盘状态或真实鼠标/目标激活移动接口。实际 API 行为未运行。 |
| 单任务与时间 | STATIC / PURE PASS：后端单一 Engine/Job，固定 3000 ms 截止和约 50 ms 释放，重复请求拒绝、不排队；单个 worker 约 10 ms 节拍，切页或收起不重置任务。实际调度与目标处理耗时不是精确 50 ms 承诺。 |
| 可信身份与阶段 | STATIC / PURE PASS：后端捕获绑定 HWND/PID/进程创建时间/线程及 epoch，开始请求核 expected epoch；按下前、准备后实际按下前、释放前核原身份及当前绑定。失效不跟随替代实例，同目标重绑也改变 epoch。既有快速同进程同线程 HWND 复用限制保留，不宣称消除所有竞态。 |
| 取消、解绑与换绑 | STATIC / PURE PASS：倒计时取消不发送；按下后取消在原身份/epoch仍有效时立即尝试一次配对释放，否则 partial，不向替代窗口补发。锁序 input→targets；转换先取消/配对，释放 targets 后等待 overlay stop，仍持 input 阻止新任务，随后取 targets 执行转换；未见持 targets 等待 overlay 回复的锁环。真实线程交错未实跑。 |
| 退出 | STATIC PASS：真实退出回调先置 stopping，拒绝新增任务并取消/配对；取出 worker 后释放其 mutex 再 shutdown/join；worker 也观察停止状态。无需等待完整 3 秒或完整 50 ms，仅原生查询/非阻塞投递及 worker 节拍；不把静态接线或 fake cancel 测试当成真实 Tauri 退出时序实证。 |
| 结果与前端 | STATIC / HELPER PASS：down/up 独立 receipt，区分 rejected/failed/partial/delivered，保留 Win32 错误码且不自动重试；成功只称已投递、目标响应待人工确认。事件+轮询、单调 revision、防重入及执行期间草稿禁用，常驻取消入口与结果；重复开始/全局热键仍禁用。真实控件/托盘/关闭选择待人工验收。 |

## 人工门槛与停止

- **HUMAN PENDING**：三种输入分别倒计时后切到其他工作窗口，观察实际目标响应、其他窗口无额外输入、真实鼠标不移动和目标不抢焦点；PostMessage 成功不能代替玩法响应。
- **HUMAN PENDING**：实际取消、同目标重绑/换绑/解绑、退出、目标退出/最小化、无效坐标及错误展示；收起/切页后的计时和状态恢复。先单次，不进入重复执行。
- **OVERLAY-001 保持延期**：用户此前已报告实际预览不显示，本轮没有修复、运行重测或把旧视觉失败改记为通过。单次输入不依赖原生预览。

全程未启动应用、GUI/隐藏窗口夹具、浏览器、开发服务器或桌面自动化，未执行真实 PostMessage/SendInput、窗口枚举/查询、目标诊断或外部进程操作。只写本次 verifier 证据与本报告，生产源码只读；未提交、推送、发布、下载工具或清理历史。日志与报告使用相对引用/占位，未保存真实标题/PID/HWND/完整进程路径；没有系统级写入监控，不宣称整个系统零外写。

本轮无 findings，不向 Coder 追加工作；约定源码/非 UI 复审到此结束，人工门槛由用户完成。
