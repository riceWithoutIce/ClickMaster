# DRAIN-001 独立限定复审

日期：2026-10-22。结论：DRAIN-001 源修复及紧邻纯回归 PASS，原 finding 关闭；无新增 BLOCKER 或需用户处置的 finding。按约定停止本次审核，不扩展全面审计，不授权下一轮真实发送。

## 复审范围与结论

仅比较 `coder/` 与冻结 `coder-r1/`，复审 JSON 日期反序列化 → 明确 UTC → 单调起点/年龄 → 监督阈值链，以及同链 double 精度、旧证据和单 attempt 保留。两个未修改源码 hash 与原快照完全相同；新 launcher 变化仅为日期处理函数、同链调用、文件读取目录参数及纯时间 fixture。

实际生产路径不再把 ConvertFrom-Json 已生成的 DateTime 隐式转为丢失时区/小数的字符串后重新解析。DateTime 按 Kind 转 UTC、DateTimeOffset 取 UTC、明确 Z/偏移字符串按 invariant 解析，未带时区的值拒绝。生产监督调用与时间 fixture 共用这组函数。两处 Math.Max 使用 `0.0`，独立回归实证 104ms 保留，避免整数重载取整。

真实旧 worker 回执中的 DrainStartedUtc 为 `2026-10-22T04:41:38.9374446Z`，与回归历史值一致。fixture 确实经过 C# 生产 Save 写 JSON、生产 Read-WorkerReport 文件读回/反序列化，再进入生产 UTC/单调处理链；不是只测试另写的镜像算法。

## 独立证据

全新输出 `build/verification/TAPTAPLOOT-IPC-DRAIN/verifier-r1/self-test/`：

- 编译 PASS，原 31 项核心纯测试 PASS，原子阶段文件 fixture PASS。
- 13 项时间回归 PASS：历史七位小数 UTC、另一小数 UTC、显式 +08:00 均保留 UTC ticks；104ms 映射无八小时偏差；9.999 秒继续，10 秒触发 drain-timeout；DateTimeOffset、UTC DateTime 和原始带 Z 字符串均保留 ticks。
- 两个 PowerShell AST 解析错误均为 0。
- 独立重新计算修复前清单所列 61 文件的 SHA256，变更数为 0，涵盖原冻结源码/证据、runtime 和产品文件。结果在 `verifier-r1/preservation-check.json`。
- 审核前后三个新源码 hash 均匹配下表。原固定 runtime/attempt 仍存在；源码未增加可选择的真实运行目录，CreateNew attempt 保护保留。没有通过执行 RunOnce 来测试已消费保护。

| 文件（相对 coder-r1） | 审核前后 SHA256 |
| --- | --- |
| `Probe.cs` | `496EC393FEC281004512BD0D44E68CD059E8AC9B49E3F272842838612037956D` |
| `probe.ps1` | `A65288B2B708FB8F32ED62A36F5379E5E43FD832995050A740318D650EDF328B` |
| `worker.ps1` | `4B2C23191F6FDBE110161539ADB4DD1FA01285E2272917CA02ECCEB7A791F130` |

## 边界

本轮仅文件、编译和纯时间/策略验证；TEMP/TMP/TMPDIR 与所有新增证据在仓库内。未运行 RunOnce、真实 worker/管道、目标进程或窗口查询、发送、终止进程；未改旧证据、产品或游戏。该源修复不会把旧实测补写为 drain 成功，也不证明实际原生读取、强制终止、持续游戏效果或完整输入隔离。下一次验证范围仍须另行对齐。
