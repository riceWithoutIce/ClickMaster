# TASK-003-ROUTING 独立限定验收

日期：2026-10-22。验收对象：`TASK-003-ROUTING-CODER-S1`。

结论：静态及独立相称检查通过，无 findings、无当前阻塞项；可由主协调者按契约执行唯一一轮主程序公开 UI 自动验收。真实 UIA 可操作性、内容窗口实际接收和计算器响应未由 Verifier 测试，本报告不作为这些项目通过证据。

## 冻结对象

- 17项清单路径相对仓库根：`build/verification/TASK-003-ROUTING/coder/source-manifest.sha256`。
- 清单 SHA-256：`4CF3C1A51A1F778DC0D88CA08B69C4CE1F52553825B37618C1C97C7E815456C7`；逐项核对和检查结束时再次核对均零差异。
- 最终 EXE：`build/verification/TASK-003-ROUTING/coder/final/ClickMaster.exe`，9,251,840 bytes，SHA-256 `1A445CC5827E45DFBD85E2C917AB31309A020A41B260B1C5B5CAD01ADD1A75FC`，与 Coder 交接一致。
- 独立检查输出：`build/verification/TASK-003-ROUTING/verifier/`；源码哈希核对记录为 `source-hashes.json`。

## 独立执行的检查

工作目录均为仓库根。

| 检查 | 结果 | 证据与限制 |
| --- | --- | --- |
| `./scripts/npm.ps1 run typecheck` | PASS | 无真实窗口、服务器或 GUI |
| `./scripts/npm.ps1 run test:targets` | PASS | 筛选、位置、单次输入及 receiver-routing 生产纯辅助逻辑 |
| targets/routing/input 纯模块 Rust harness | 24/24 PASS | 复制原生产模块与原测试，仅去除 serde 导入、序列化派生/属性后用 rustc 编译；未链接或调用 Win32/UIA |
| 自动化助手独立副本默认模式 | 编译 PASS | `verifier/automation/run.ps1`；运行 `20261022T031710812Z-672c9dae384946699e34e03000d78f14`，明确 `DesktopQueries=false`、`InputPosted=false` |
| manifest / EXE 身份 | PASS | 如上 hash、大小及17项逐项核对 |
| 新文本脱敏辅助扫描 | 未发现候选 | 本机绝对路径、用户目录、典型凭据赋值模式；扫描不能证明所有敏感内容均不存在 |

纯模块输出位于 `verifier/pure/test-output.txt`。首轮 harness 复制时漏去一个独立 `derive(Serialize)` 导致 harness 编译失败；只修正验收副本的去序列化处理后24项通过，未更改产品文件或逻辑。

另曾启动完整后端检查，明确 `CLICKMASTER_TEST_ROOT=build/verification/TASK-003-ROUTING/verifier/tests` 后运行 `./scripts/npm.ps1 run test:backend`；工具连接中断后未保留可消费的原会话终态。因此完整后端独立结果为**证据不可确认**，即使目录中已有测试夹具也不记为 PASS；未为补证重跑。相称的独立24项纯检查已完成。完整40项 Rust、生产前端、Windows release 与 PE 图标 PASS 继续归于 Coder 交接证据，Verifier 未独立重跑 release 构建。

## 核心审查

- 绑定根身份与输入接收身份分离；普通窗口自身、明确宿主唯一 CoreWindow 自动、多候选待显式 token 选择，前端不可传任意 HWND/PID 决定投递。宿主结构查询失败不静默回退。
- 接收目标包含 PID、进程创建时间、线程及父子/根关系复验；失效后清空接收目标、停止新输入，不自动发现替代窗口。绑定或路由 epoch 变化使已有任务快照失效；显示刷新不重选内容。
- 切换绑定/接收者先取消旧任务；输入 worker、命令和取消沿用 input→targets 锁序。原按下成功后的释放仍复验原快照，失效/释放失败显示 partial，不转发替代窗口。overlay 停止不在持 targets 锁时等待。
- 键盘使用接收窗口线程布局；左右键先校验绑定客户区位置，在同一 PMv2 上下文由 ClientToScreen/ScreenToClient 转换，再校验接收客户区和16位范围。映射失败/越界不截断、不夹边、不移动真实鼠标。纯测试覆盖映射偏移、负值、越界、范围边界和失败。
- UI 分别呈现绑定与实际接收对象，路由未就绪/失效时单次禁用；自动/手动/自身/待选状态明确。任务 receipt 保留本次实际接收摘要，投递完成仍不宣称玩法响应。既定单次倒计时、取消、失败/partial 和不自动重试语义保持。
- 自动验收助手只经本轮 owned EXE 的 UIA 子树进行公开选择/Invoke；计算器仅结果读取，没有 PostMessage/全局输入、激活或计算器 Invoke/SetValue 路径。基线只接受0或1；控件不支持/不唯一则在投递前停止，Single Invoke 一次后不重试。
- 助手清理仅使用本次 Start 返回的 owned Process；watchdog 仅终止 owned worker，不按进程名杀外部实例。超时可能留下本轮产品、输入结果未知，说明文件已明确，不作为误停外部进程风险。
- 编译/验收受管理输出与 TEMP/TMP/TMPDIR 均在项目工作目录；报告仅固定码/归一化显示值/布尔值，不保存原始 UIA 树、真实标识、路径、截图或计算历史。没有系统级外写监控，不宣称全系统零外写。

## Findings 与停止点

无 findings，当前限定审核结束，不追加全审或给 Coder 派修复。Verifier 未枚举真实窗口、调用真实 UIA、启动产品/浏览器、投递输入、修改被验源码或提交/推送。

主协调者可执行契约内唯一真实公开 UIA 验收；无法操作 WebView 控件时按约定记为未完成，不用旁路发送工具替代产品通过。实际左右键、游戏兼容性和全程输入隔离保持未验证，OVERLAY-001 继续延期。
