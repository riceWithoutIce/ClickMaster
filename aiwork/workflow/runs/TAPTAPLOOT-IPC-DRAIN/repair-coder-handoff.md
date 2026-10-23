# DRAIN-001 修复 Coder 交接

日期：2026-10-22。本轮仅修复诊断监督的 UTC/小数计时链，稳定源码位于 `build/verification/TAPTAPLOOT-IPC-DRAIN/coder-r1/`。原 `coder/`、`runtime/`、旧验收记录及产品源码保持原状。此交接后冻结新快照，等待独立限定复审。

## 实现

复制原三个源码；`Probe.cs`、`worker.ps1` 与原快照逐字节相同。仅新 `probe.ps1` 改动：

- 原 `DateTime.Parse($snapshot.DrainStartedUtc)` 会先隐式把已反序列化的 UTC DateTime 转成丢失 Kind/小数的字符串。改为按输入实际类型转换：DateTime 保留 Kind 与 ticks 后转 UTC；DateTimeOffset 直接取 UTC；带明确 Z/偏移的字符串按 invariant 解析。无时区日期拒绝，不猜测。
- 实际 launcher 调用共用 `ConvertTo-DrainUtc`、`Get-DrainMonotonicStart`、`Get-DrainMonotonicAge`，从 UTC 观察时刻映射到原单调时钟。相同函数用于纯文件时间回归。
- 初次新增回归发现同一计时链的 PowerShell `Math.Max(0, double)` 会选整数重载并丢失小数，导致 104ms 被取整；同链统一使用 `0.0`，保留小数。10 秒和 45 秒阈值未改。
- 文件读取函数增加仅用于读取的目录参数，生产调用保持默认原 runtime；文件回归调用同一读取/JSON反序列化函数。未增加真实运行目录参数或其他 attempt 能力。

固定原 runtime、原子 CreateNew attempt、单帧数量 1、身份/ACL、10 秒/45 秒监督策略、仅 Kill 自有 Process 对象且不杀树、有限退出等待均保留。原 runtime 已消费，修复入口不能用于再发一轮。

## 已执行与证据

最终默认纯入口：`coder-r1/probe.ps1 -EvidenceDirectory build/verification/TAPTAPLOOT-IPC-DRAIN/coder-r1/self-test-freeze`。

- 编译成功，原 31 项核心纯测试 PASS；原子阶段文件 fixture PASS。
- `self-test-freeze/time-regression.json`：13 项具名时间回归 PASS。通过生产 Save 保存 JSON、生产 Read-WorkerReport 读取、生产时间函数与监督策略，验证历史值 `2026-10-22T04:41:38.9374446Z`、另一七位小数 UTC、显式 `+08:00` 偏移。验证 ticks 不变、104ms 没有变为八小时、9.999 秒继续、达到 10 秒返回 drain-timeout；另检查 DateTimeOffset、UTC DateTime、原始带 Z 字符串。
- 两个 PowerShell AST 解析错误均为 0。定向源码核对原 runtime 和 CreateNew 未变、两处 Kill 仍只引用自有 worker、退出等待仍为 2000ms。
- `preserved-baseline.json` 是修复前的 61 文件 SHA256 清单，涵盖原 coder 全部源码/证据、原 runtime 全部文件、产品 src/src-tauri 与工程配置；`preservation-check.json` 复核 61 文件变更数为 0。

保留 `self-test` 的初次纯回归失败证据与 `self-test-final` 中间成功证据，没有删除或覆盖旧测试产物。最终采用 `self-test-freeze`。初次失败仅为 Math.Max 小数映射断言，未运行任何真实操作。

## SHA256 稳定快照

| 文件（相对 coder-r1） | SHA256 |
| --- | --- |
| `Probe.cs` | `496EC393FEC281004512BD0D44E68CD059E8AC9B49E3F272842838612037956D` |
| `probe.ps1` | `A65288B2B708FB8F32ED62A36F5379E5E43FD832995050A740318D650EDF328B` |
| `worker.ps1` | `4B2C23191F6FDBE110161539ADB4DD1FA01285E2272917CA02ECCEB7A791F130` |

## 验收边界

本轮未运行 RunOnce、真实 worker、管道 API、目标进程/窗口查询、发送或进程终止；未修改旧实测结果、生产代码、游戏或其数据。仅使用已有文件与纯时间/文件/编译测试。TEMP/TMP/TMPDIR 沿用仓库内 `build/tmp/TAPTAPLOOT-IPC-DRAIN`，测试输出位于新快照内。

源修复和纯回归不补写原实测为 drain 成功，不证明原生 FlushFileBuffers、真实监督终止或持续游戏响应。原 25ms 轮询/调度/退出等待的实际时限限制不变。独立 Verifier 应只复审 DRAIN-001 及紧邻回归，用自己的新输出目录，不执行真实入口；发现问题一次性返回用户裁定。
