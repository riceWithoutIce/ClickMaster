# TapTapLoot IPC POC Coder 交接

日期：2026-10-22。快照：`TAPTAPLOOT-IPC-POC-CODER-S1`。状态：**READY_FOR_REVIEW**，脚本冻结。任务边界见 [task.md](task.md)。

## 改动与快照

本轮 Coder 为唯一诊断脚本写入者，生产源码、原分析脚本/结果、游戏文件均未修改。

| 文件（仓库相对路径） | SHA-256 |
| --- | --- |
| `build/verification/TAPTAPLOOT-IPC-POC/coder/Probe.cs` | `E76AD24D3B6D7A407A5001B7DF7989804EA6E50DDC18F9F6D950908CBF78986E` |
| `build/verification/TAPTAPLOOT-IPC-POC/coder/probe.ps1` | `DAEAA3C34FE8D844F3E61125B2DA59298414166B82B74466A12C8FEAF93D091A` |

辅助纯测试结果：`build/verification/TAPTAPLOOT-IPC-POC/coder/self-test/result.json`。本交接是说明文档，不纳入脚本冻结 hash。项目存在先前 dirty/untracked 状态，本轮不整理、不提交，不把 HEAD 当成此脚本快照。

## 实现

- 默认 PowerShell 入口仅 Add-Type 编译和调用 SelfTest；真实模式必须显式 `-RunOnce`。纯自检不调用任何目标枚举、窗口查询、命名管道或原生 API。
- 真实模式固定在任务目录 `runtime/attempt.json` 使用 CreateNew 原子声明一次尝试；失败/拒绝也保留，不删除或覆盖，不允许调用者选另一落点绕过。再次执行返回 refused，不创建管道或写游戏。
- 精确进程名唯一，持有原 Process/SafeHandle，创建时间/存活检查；QueryFullProcessImageName 校验精确 EXE，磁盘主程序集 hash 固定匹配分析版本；唯一 UnityWndClass、归属、客户区、可见/非最小化/后台在准备后、连接后、发送前再次检查。
- 5秒准备后使用原生 CreateNamedPipe：`FILE_FLAG_FIRST_PIPE_INSTANCE | FILE_FLAG_OVERLAPPED | PIPE_ACCESS_DUPLEX`、`PIPE_REJECT_REMOTE_CLIENTS`、最大1实例。受保护 DACL 仅当前用户 SID，非继承句柄；固定 TypingFarmer 通道，不枚举或连接已有端点、不接触 buff 通道。冲突/权限错误直接终止。
- 连接预算20秒；GetNamedPipeClientProcessId 两次检查连接方为同一目标 PID，配合保留进程对象/创建时间。单次预算防止第二次写；一次 WriteAsync 传入四字节 `00 02 31 00`，预算2秒。无重试、补发、Flush/WaitForPipeDrain 或其他输入。
- 首次写调用前先保存 `write-attempted` 保守回执；超时/异常如已进入发送阶段标为效果未知，不能因无完成回执重跑。成功只称 transport write completed，不称游戏已消费/玩法成功。
- 成功后最多30次、每次100ms采样目标是否前台，观察退出即停。仅保存计数和阶段，不输出其他前台窗口信息或用户按键/鼠标。
- 异常路径释放本诊断服务器和自己的 Process 查询句柄；不操作外部进程生命周期，不退出游戏。最终保留 result.json；不自动清理证据。
- nominal async 总预算35秒涵盖5秒准备、20秒连接、2秒写入和约3秒观察。现有 OS 同步查询/文件系统访问及最终落盘没有独立硬终止保障，不宣称 OS hang 情况也可精确35秒结束；未使用无限异步等待或 drain。

## 已执行与未执行

执行位置：仓库根目录，现有 PowerShell 7。命令：

```powershell
& ./build/verification/TAPTAPLOOT-IPC-POC/coder/probe.ps1
```

结果：C# 编译 PASS，19项纯自检 PASS，包括精确协议帧；数量 -1/0/2/max拒绝；有效闸门；目标缺失/多目标、客户端身份不符、实例变化、隐藏/最小化/前台、窗口缺失/歧义、程序集不符拒绝；一次发送预算首取允许、第二/第三次拒绝。结果明确 `pure-tests-pass-no-native-calls`、WritesAttempted=0。

- 未运行 `-RunOnce`；未调用真实进程/窗口查询、建立/连接管道、发送输入或启动 UI。没有实际管道冲突、ACL、连接身份、超时/断连、游戏响应或全程隔离证据。
- 没有下载、安装、修改依赖/产品配置、提交、推送或发布。
- 编译前脚本按进程设置 TEMP/TMP/TMPDIR 为 `build/tmp/TAPTAPLOOT-IPC-POC/`；结果位于 coder/self-test。运行时脚本按自身路径解析项目根，不持久保存机器路径。没有系统级零外写监控。
- 对 coder 目录文本进行本机盘符路径与禁止输入 API 名称聚焦扫描，无命中；rg无命中返回1，不是编译/自检失败。扫描不能证明穷尽脱敏。

## Verifier 与主协调者入口

Verifier只执行默认纯模式，可指定任务内自己的证据目录：

```powershell
& ./build/verification/TAPTAPLOOT-IPC-POC/coder/probe.ps1 -EvidenceDirectory ./build/verification/TAPTAPLOOT-IPC-POC/verifier/self-test
```

只有主协调者核对本快照/独立审核通过后可执行一次：

```powershell
& ./build/verification/TAPTAPLOOT-IPC-POC/coder/probe.ps1 -RunOnce
```

该入口会占用本轮唯一实际尝试，即使前置拒绝或连接失败也不重跑。`runtime/attempt.json` 或 `runtime/result.json` 已存在时不得删掉重试；是否执行过以 attempt 回执为主，不因终端中断当作没发送。遇 setup/receipt错误也先查项目内阶段，不自动再次调用。

实际发送后由用户单独确认玩法/显示计数/统计、副作用，不能由传输回执替代。真实输入也可能推进游戏，非静止基线下的观察不证明精确+1或排除人工输入。当前仅诊断工具，不是产品集成许可。
