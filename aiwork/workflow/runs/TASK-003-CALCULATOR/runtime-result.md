# 自动单次验证运行记录

日期：2026-10-22。快照：`TASK-003-CALCULATOR-CODER-S1`。执行者：主协调者；独立 Verifier 无 findings 后执行一次，未重跑。

## 命令与原始脱敏回执

工作目录：仓库根。

```powershell
& ./build/verification/TASK-003-CALCULATOR/coder/run.ps1 -ExecuteOnce
```

运行编号：`20261022T022424156Z-7267f3cfadb44f2c94f7b834f3fb2aa0`。

回执：`build/verification/TASK-003-CALCULATOR/coder/runs/20261022T022424156Z-7267f3cfadb44f2c94f7b834f3fb2aa0/receipt.sanitized.json`。

| 字段 | 实际结果 |
| --- | --- |
| Outcome / Phase | not-sent / stopped |
| Reason | eligible-content-not-unique |
| EligibleCandidates | 0 |
| Down / Up | not-attempted / not-attempted |
| Baseline / FinalDisplay | unreadable / unreadable |
| ObservationCount | 0 |
| Response | inconclusive |
| ForegroundSame / CursorSame | null / null（未进入该阶段） |

启动器退出码为0表示诊断流程正常结束，不能解读为输入执行成功或计算器响应通过。原因码涵盖0或多个候选，本轮计数明确为0，不能称为多个候选冲突。Baseline 的 unreadable 为尚未读取的初始值，本次没有执行 UIA 结果控件读取，不能将其记为真实 UIA 读取失败。

## 结论与限制

- 本轮自动运行正常停止在筛选门槛，无真实输入投递，无重试、激活、鼠标移动、应用启动或产品代码修改。
- 自动化脚本已具备定位、门槛、单次发送和前后结果判定路径；本机实际仅验证到定位拒绝。真实内容窗口数字键响应仍待验证，不能声称自动化端到端通过。
- 当前回执不能区分：内容候选是否已经变化、进程名未匹配，或可见性/最小化/cloaked等筛选条件未满足。前次只读结构快照不代表本轮仍然相同。尤其不根据0个候选判定计算器后台输入不兼容。
- 下一步应先只读细分筛选原因（按框架、内容类、进程白名单、可见性及DWM查询返回结果分别计数，不保存真实标识），再决定是否调整诊断工具。当前不放宽白名单、不删除身份门槛、不自动重跑输入。
- 原TASK-003 50文件快照在本轮检查中零差异；没有提交、推送、发布或历史清理。OVERLAY-001保持延期。

## 用户重新打开计算器后的只读预检（2026-10-22）

用户说明旧计算器已经关闭，随后重新打开并通知继续。本阶段按已说明的范围只读检查新窗口，不沿用旧句柄、不发送输入。协调者新增独立 `build/verification/TASK-003-CALCULATOR/filter-diagnostic/read-filters.ps1`，只调用窗口/进程/DWM读取接口、输出脱敏别名，不更改冻结的Coder四文件或产品源码。

- 筛选诊断运行：`20261022T023640395Z-1d44db58ef0a418199d865b683bd1a30`，结果在上述诊断目录 `runs/<run-id>/filters.sanitized.json`。
- 两个匹配框架中，F1及其唯一CoreWindow C1通过可见性、非最小化、DWM cloaked=0、正客户区、计算器进程白名单、身份可读与父子关系检查；合格内容目标数为1。
- F2的DWM调用成功，但cloaked=2，按原规则被排除。此次没有修改筛选门槛，不能将F2直接视为失效窗口。
- 随后默认只读入口运行：`20261022T023650414Z-1def81e2d48e471d9735ef0d62808e66`，回执在 `build/verification/TASK-003-CALCULATOR/coder/runs/<run-id>/receipt.sanitized.json`。
- 默认预检得到 `EligibleCandidates=1`、`Baseline=0`、`Reason=preflight-ready-no-input`、`Phase=complete`。Down/Up均为not-attempted；没有结果观察或前后台/鼠标对照。

结论：新开计算器的内容定位及UIA基线读取已实际通过，未修改脚本门槛，也没有复用旧实例。此前0候选发生在不同窗口生命周期，不能根据这次成功反推上次唯一根因。单次内容窗口消息实际响应仍待真实投递验证，本阶段未执行输入、不作为响应通过证据。

## 用户明确开始后的单次实际投递（2026-10-22）

用户在新窗口预检通过后明确“好的 那我们开始测试”，授权本次新单轮执行。主协调者重新核对冻结的四文件清单，零差异，使用原S1脚本执行一次显式 `-ExecuteOnce`；未修改门槛、未沿用旧句柄、未重试。

运行编号：`20261022T023934831Z-de9588a2a150457e962a4923e65b5570`。

脱敏回执：`build/verification/TASK-003-CALCULATOR/coder/runs/20261022T023934831Z-de9588a2a150457e962a4923e65b5570/receipt.sanitized.json`。

| 字段 | 实际结果 |
| --- | --- |
| EligibleCandidates | 1 |
| Baseline / FinalDisplay | 0 / 1 |
| Down / Up | posted / posted |
| DownError / UpError | 0 / 0 |
| Outcome / Phase | posted / complete |
| Reason / Response | both-messages-posted / observed-0-to-1 |
| ObservationCount | 8 |
| ForegroundSame / CursorSame | true / true |

本轮目标身份/父子关系、后台和0基线门槛通过，向唯一计算器CoreWindow内容子窗口投递一次主键区1的按下/释放，自动观察到0→1。本机当前计算器内容子窗口的这次后台数字键路径取得实际响应证据；不再仅依据API返回值判断响应。

前台/鼠标前后采样相同不证明全程未变或其他窗口零输入；没有输入接收侧监控，不能排除并发人工输入1的影响。没有取得人工独立观察或普适因果保证。此前顶层投递与本轮测试属于不同时间/实例，支持优先处理内容接收窗口选择，但不构成其他因素均受控的唯一根因证明。

本次测试至此停止，不追加按键、重复执行、鼠标或其他目标。ClickMaster产品仍未接入内容子窗口路由；TapTapLoot与其他程序兼容性未验证，OVERLAY-001继续延期。没有产品源码/版本修改、提交、推送或发布。
