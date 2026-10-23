# Coder 稳定快照交接

日期：2026-10-22。范围仅为本轮独立读取确认诊断，不接入产品；本文件交接后冻结下面三个源码文件，供独立 Verifier 只读审核。

## 交付

- `build/verification/TAPTAPLOOT-IPC-DRAIN/coder/Probe.cs`：复用旧诊断身份、窗口、单写和管道 ACL/独占保护；唯一一次数量 1 帧写后持久记录阶段，调用原生 `FlushFileBuffers`。成功后观察约 3 秒，不把读取成功描述为玩法成功。
- `build/verification/TAPTAPLOOT-IPC-DRAIN/coder/probe.ps1`：默认仅编译/纯自检。`-RunOnce` 原子声明固定 runtime attempt，编译后只创建一个隐藏自有 worker；使用其直接返回的 Process 对象监督并仅终止该对象，不结束进程树或扫描进程后结束。正常 Dispose、确认 worker 退出后的 OS 回收、退出未确认分别记录。异常路径保留 worker 阶段，缺失阶段时明确写入未知，不能推断未发送。
- `build/verification/TAPTAPLOOT-IPC-DRAIN/coder/worker.ps1`：只加载本轮准备的 DLL；校验 launcher attempt、DLL hash、内存环境令牌，另有原子 worker claim。不给缺失令牌的入口进行真实操作。

所有临时环境变量、编译产物、文件 fixture 和证据均放在仓库的本任务目录或 `build/tmp/TAPTAPLOOT-IPC-DRAIN`。本轮未改生产源码、游戏、旧诊断或旧 attempt。

## 已执行验证

执行默认入口并指定 `coder/self-test-freeze`：编译成功，31 项纯自检 PASS。涵盖固定四字节帧、拒绝其他数量、原核心 Gate、one-shot、10 秒 drain 与 45 秒总监督策略、drain 已完成不误杀、正常/强制/未确认清理区分、native 失败/发送不确定性、无令牌拒绝且无 I/O。

同一默认入口另外执行文件 fixture，使用生产 `Save` 的原子替换写入 pending/completed JSON 并读取确认；`self-test-freeze/stage-fixture.json` 为 PASS。此 fixture 为文件测试，没有启动 worker 或创建任何管道。

两个 PowerShell 文件 AST 解析错误均为 0。定向源码检查确认仅一处 `WriteAsync`、一处 native drain；所有 `WaitForExit` 显式为 2000ms；两处 Kill 均引用本次直接保存的自有 worker 且参数 false。未使用键鼠注入或抢焦点 API。

保留了较早的 `self-test`、`self-test-final` 中间证据，未覆盖删除。最终证据是 `self-test-freeze`。本任务 `runtime` 尚不存在，因此没有真实 attempt。

## 未执行与实际限制

没有执行 `-RunOnce`、启动 live worker、查真实游戏或窗口、创建/连接命名管道、调用 native FlushFileBuffers、发送帧或终止任何进程。纯策略与文件测试通过不构成原生 API、实际监督终止或游戏响应证据。

监督采用约 25ms 轮询。10 秒 drain 与 45 秒总预算为名义监督阈值，实际结束受调度、退出等待最多 2 秒等影响，不承诺实时或系统挂死时限。drain 开始时持久化 UTC，launcher 首次读到时折算到单调时钟，减少发现阶段延迟。阶段异常时仍受总预算限制。

真实入口仅由主协调者在审核后执行一次；前置失败也消耗固定 attempt，禁止自动重试。正常退出时看到 drain 成功只代表管道客户端读走服务器缓冲，解析、事件处理与玩法效果须另记人工观察。目标前台采样不是完整输入隔离证明。

## SHA256 冻结

| 文件（相对 coder 目录） | SHA256 |
| --- | --- |
| `Probe.cs` | `496EC393FEC281004512BD0D44E68CD059E8AC9B49E3F272842838612037956D` |
| `probe.ps1` | `950629BBA571355D244F0786900B4E5BA405782EABF653F892CA8AD2E8A49CA6` |
| `worker.ps1` | `4B2C23191F6FDBE110161539ADB4DD1FA01285E2272917CA02ECCEB7A791F130` |

建议 Verifier 使用新的本任务内 evidence 子目录执行默认纯自检，避免覆盖既有 DLL；不执行真实入口。若发现问题，一次性汇总交用户决策，不直接修改本快照。
