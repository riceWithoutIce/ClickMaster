# IPC-REPEAT Coder 稳定快照交接

日期：2026-10-22。交付独立验证工具，不接入产品。源码冻结于 `build/verification/TAPTAPLOOT-IPC-REPEAT/coder/`；此交接后等待独立 Verifier，只读审核后才由主协调者运行真实入口。

## 入口与阶段

- 默认：`coder/probe.ps1`，只编译、纯策略/时间/文件测试；可用 `-EvidenceDirectory` 指定本任务内、真实 runtime 以外的新证据目录，已有 DLL 拒绝覆盖。
- 真实 single：`coder/probe.ps1 -RunOnce -Stage single`。固定 `runtime/single/attempt.json` 原子声明，单帧数量 1。
- 真实 repeat：`coder/probe.ps1 -RunOnce -Stage repeat`。固定 `runtime/repeat/attempt.json`，不能指定其他运行目录；须先满足本轮 single 的传输、正常关闭、人工反馈和目标连续身份门槛。没有人工反馈时主协调者应等待，不尝试运行 repeat 消耗 attempt。
- 每阶段前置拒绝也消耗对应 attempt；无自动重试/重连。内部 `worker.ps1` 需要 launcher 准备的 DLL hash、环境票据与固定 attempt，另原子声明 worker.claim；缺票据在任何文件/原生操作前拒绝。

## 实现与边界

`Policy.cs` 提供实际共用调度器、阶段门槛、监督策略、DTO、原子阶段写入；`Probe.cs` 持有目标查询 lease、身份/窗口校验、唯一管道和逐帧 worker；`Hotkey.cs` 用 RegisterHotKey/PeekMessage/UnregisterHotKey 实现 F8，无 hook；`SelfTests.cs` 是纯测试；`probe.ps1` 负责固定 attempt、编译、F8生命周期、自有 worker 监督；`worker.ps1` 是受控内部入口；`timing.ps1` 提供实际共用 UTC/逐帧监督/attempt 函数及纯文件回归。

single 上限 1，repeat 上限 5；每帧固定 `00 02 31 00`，上一帧未 drain 不会安排下一帧。采用保守低频：下一帧至少在前一帧 drain 成功后 2 秒，因此发送开始间隔至少 2 秒；延迟后不追赶。每帧写前保存 write-attempted；写成功、drain 开始/成功逐帧保留。发送前再次核对同一客户端和原目标。同步 drain 由父监督 10 秒，每次 WriteAsync 令牌预算 2 秒；阶段从自有 worker 启动前起总监督 60 秒，worker 自身非阻塞操作另有 58 秒令牌。总限优先，不保证发送满 5 次。

F8 注册失败不创建 worker。注册成功后 launcher 绑定并保留只读目标 lease，worker 独立重新绑定并比对身份摘要；launcher 约 25ms 轮询 F8、目标资格与阶段，worker 在准备/连接/间隔/观察及每帧前复核。目标失效/前台、取消、失败或超时停止。只终止本次直接返回并持有的 Process 对象，Kill(false)，不杀树、不按名称/PID结束外部进程，退出等待最多 2000ms。finally 释放热键与只读 lease。正常 Dispose、确认 worker 退出后 OS 回收、退出未确认分别记录。取消可能存在在途帧，不撤回或声称未发送。

原 TypingFarmer 管道名、first-instance/max1/current-user ACL/reject-remote、精确 EXE/程序集 hash、唯一 UnityWndClass/后台/可见/非最小化/有效客户区、创建时间和保留句柄边界沿用。身份摘要包含本任务名、原实例 PID/创建时间/窗口及已知程序集 hash 的 SHA256；仅摘要落盘，不落原值。single→repeat 不跟随重启或新窗口。未增加键鼠、PostMessage、抢焦点、读进程内存、调试器、游戏/产品修改或下载。

## 本轮人工反馈门槛格式

主协调者仅在**本轮 single 正常结束且收到用户新的明确有效反馈后**，写 `runtime/single/human-feedback.json`。此文件不由工具自动生成；旧一轮“成功了”不能填入本轮。

```json
{
  "Task": "TAPTAPLOOT-IPC-REPEAT",
  "SingleRunId": "<本轮 runtime/single/attempt.json 的 RunId>",
  "SingleAttemptSha256": "<本轮 single/attempt.json 的 SHA256>",
  "SingleWorkerSha256": "<本轮 single/worker-result.json 的 SHA256>",
  "SingleSupervisorSha256": "<本轮 single/supervisor-result.json 的 SHA256>",
  "ConfirmedEffective": true,
  "ConfirmedUtc": "<收到本次人工确认后的 UTC ISO8601 时间>"
}
```

repeat 的 launcher 与 worker 均调用同一 LoadSingleGate：核对任务/阶段/run、单帧写与 drain 成功、正常关闭、监督退出且未 kill、热键注销、反馈 run 和三个最终文件 hash、反馈时间不早于 single 监督结束且不在未来；再比对当前目标摘要。通过仅代表允许运行本次 repeat，不把人工观察当精确 +1 或完整隔离证据。

## 已执行验证

最终默认入口指定 `coder/self-test-freeze`，编译成功，74 项具名纯检查全部 PASS：

- `result.json`：51 项。实际共用 FrameSchedule 的 single/5帧上限、未 drain 不安排、2秒边界、不追赶、取消（含在途）/失效后不安排；共用 10/60 秒监督与 F8启动门槛；真实 LoadSingleGate 的纯文件 fixture；旧反馈/hash/时间/身份/仅写成功/kill 不通过门槛等。
- `time-regression.json`：21 项。实际 Policy.Save、共享 Delete 读取/JSON日期、Get-SupervisionDecision 和 UTC函数完整链；历史 UTC 七位小数、另一小数 UTC、明确偏移；104ms精度、9.999/10秒边界、原子 pending→completed、下一帧重新计时。保留 DateTime Kind/ticks，Math.Max 明确用 0.0。
- `attempt-regression.json`：2 项。调用与 launcher 相同的 New-StageAttempt，在纯 fixture 中验证 single/repeat 分别 CreateNew、重复拒绝且不覆盖原文件。

三个 PowerShell AST 解析错误均为 0，见 `parser-check.json`。定向源码检查确认唯一 WriteAsync 调用在受共用调度器限制的循环内、两处 Kill 仅引用自有 worker、两处 WaitForExit 均为 2000ms，无键鼠/抢焦点 API。源码扫描没有本机绝对路径、用户名等内容。

`preserved-baseline.json` 和 `preservation-check.json` 覆盖旧 IPC-POC/IPC-DRAIN（包括修复/审核/实测证据）及产品/配置，共 110 个文件前后 hash 不变。先前 `self-test`、`self-test-chain`、`self-test-final` 均保留，最终证据用 `self-test-freeze`。

本轮真实 `runtime/` 尚不存在。默认没有查询真实游戏/窗口、注册/发送 F8、启动 worker、建立管道或发送帧。TEMP/TMP/TMPDIR 位于本任务 `coder/tmp`，所有编译/测试/fixture/清单均在仓库内。

## SHA256 冻结

清单另存 `coder/source-sha256.json`。

| 文件（相对 coder） | SHA256 |
| --- | --- |
| `Policy.cs` | `8DA0BACD618ACF691C5CD827569C1F48AB5B482CE120D6E73BCB4CAB91921135` |
| `Probe.cs` | `5E29713561A44ED7605DACE994135CB41A988EA54FA830DBCEAC4BC3FFBFC6FF` |
| `Hotkey.cs` | `5EE9B844CDACDB11CE2C7031B1D13A008E1D7F7133FD5CF71103DC582A655AAB` |
| `SelfTests.cs` | `723EA5C041A6CCE6BFC9E8C1095378CB98765B952177E9C0B704FB5506F13B33` |
| `probe.ps1` | `D26064CA2CFA768DB2D6FFADCD04FFDDC6CC98D3F4AC1A5696BACC033C8F5E54` |
| `worker.ps1` | `B16488699638E37F9C3D4C6A32A90F4636E297176583BF1CD75B6C81DF170227` |
| `timing.ps1` | `9C2665014D2DACF9D56D38536B742CB832CFE5936ABECA8A7585D38EA54C2040` |

## 尚未验证

原生连接/写/drain、真实 F8 注册冲突/送达/取消/注销、真实 worker 结束、游戏响应/计数、全程桌面隔离与长期稳定均未执行，不能由纯测试代替。轮询、Windows 调度和退出等待使 10/60 秒是名义监督阈值，不承诺硬实时；采样无法证明瞬时状态或完整输入隔离。read/drain 成功不代表解析/玩法成功。真实操作须由主协调者在独立审核后逐阶段执行，single 后等待本次人工反馈，不自动进入 repeat；不为凑足次数增加 attempt。
