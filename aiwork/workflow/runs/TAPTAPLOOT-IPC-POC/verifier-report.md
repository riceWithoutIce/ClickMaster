# TapTapLoot IPC POC 独立限定验收

日期：2026-10-22。对象：`TAPTAPLOOT-IPC-POC-CODER-S1`。结论：**静态、编译与纯逻辑检查通过，无 findings、无当前阻塞项**。可由主协调者核对快照后执行契约内唯一一次 `-RunOnce`；本报告不是实际管道或游戏兼容通过。

## 快照与范围

- 依据：[任务契约](task.md)、[Coder 交接](coder-handoff.md)。Coder已提供稳定脚本后开始，未向开发者追加或要求修复。
- 审核前后 `Probe.cs` SHA-256 均为 `E76AD24D3B6D7A407A5001B7DF7989804EA6E50DDC18F9F6D950908CBF78986E`。
- 审核前后 `probe.ps1` SHA-256 均为 `DAEAA3C34FE8D844F3E61125B2DA59298414166B82B74466A12C8FEAF93D091A`。
- 被验文件均在 `build/verification/TAPTAPLOOT-IPC-POC/coder/`，Verifier未修改。仅写入 verifier 子目录证据与本报告，不改产品源码、配置、游戏或已有证据。

## 实际检查

| 项目 | 结果与证据 |
| --- | --- |
| 默认纯入口/显式真实开关 | STATIC PASS；默认调用 SelfTest，未调用 RunOnce；纯测试无目标枚举、窗口查询或管道原生调用 |
| 编译、原19项纯测试 | PASS；独立运行默认脚本，结果 `pure-tests-pass-no-native-calls`，WritesAttempted=0 |
| 独立协议解释 | PASS；调用冻结 Frame(1)，按游戏读取方式计算两字节大端长度、默认UnicodeEncoding解码及Int32.Parse，得到长度2、数量1、总帧4字节；无原生调用 |
| 固定唯一尝试 | STATIC PASS；task固定 runtime 路径，CreateNew原子claim，文件保留；已存在时返回拒绝，不创建端点或写入游戏 |
| 已有管道冲突与隔离 | STATIC PASS；CreateNamedPipe first-instance、maxInstances=1、overlapped duplex、拒绝远程；protected DACL仅当前用户，非继承句柄。未实际建立端点验证 |
| 目标与客户端身份 | STATIC PASS；精确EXE名、唯一进程、保留原Process/SafeHandle与创建时间、磁盘程序集hash；唯一UnityWndClass、窗口归属/可见/非最小化/后台/有效客户区检查；连接后两次客户端PID匹配，发前复核原实例和窗口 |
| 单次数量1 | STATIC/PURE PASS；单个WriteAsync调用，Frame只允许1，OneShotBudget原子首取；无重发/补发/Flush/Drain或全局输入 |
| 等待/退出 | STATIC PASS；5秒准备、20秒连接、2秒写入、最多3秒采样、35秒异步总取消预算；finally仅释放自有pipe和查询对象。同步OS/文件系统挂起无独立硬截止，Coder已明确限制 |
| 状态与不确定效果 | STATIC PASS；write-attempted在唯一写前保存，异常/超时保守标记效果未知；write-completed仅传输完成；无应用ack或玩法成功断言 |
| 用户操作/外部进程 | STATIC PASS；无键鼠发送、焦点/鼠标移动、进程终止、注入或游戏文件写入；仅查询前台是否为目标，未记录其他窗口身份/输入 |
| 脱敏与产物边界 | 聚焦扫描PASS；脚本无本机盘符路径/禁止输入与终止API；临时目录在编译前按进程设为项目内；build忽略规则覆盖本轮脚本/证据。非系统级零外写证明 |

执行位置：仓库根目录，既有PowerShell 7/.NET。主要命令：

```powershell
& ./build/verification/TAPTAPLOOT-IPC-POC/coder/probe.ps1 -EvidenceDirectory ./build/verification/TAPTAPLOOT-IPC-POC/verifier/self-test
```

独立协议解释使用同一冻结 `Probe.cs` 的 `Frame(1)`，自写纯解码检查，不运行游戏程序集。编译前TEMP/TMP/TMPDIR均设为项目内 `build/tmp/TAPTAPLOOT-IPC-POC/`。证据为 `build/verification/TAPTAPLOOT-IPC-POC/verifier/self-test/result.json` 及 `verifier/protocol-check.json`。

## 未执行与结束条件

未执行 `-RunOnce`，未实际查询游戏/窗口、建立或连接管道、发送数据、操作GUI、下载/安装、提交/推送、清理或结束任何外部进程。未验证真实ACL/冲突/客户端连接/取消/断连、游戏消费与玩法响应、统计等价或全程焦点/鼠标/工作窗口隔离。纯Gate与one-shot测试不代替这些运行证据。

无 findings，本次限定审核到此结束。主协调者按契约只执行一次真实尝试，任何前置拒绝/冲突/超时均停止且保留首个回执，不因没有玩法变化或终端中断重跑。玩法/计数/统计及用户隔离继续分别人工确认；并发真实输入可能推进游戏，不能仅用画面变化证明IPC因果或精确+1。
