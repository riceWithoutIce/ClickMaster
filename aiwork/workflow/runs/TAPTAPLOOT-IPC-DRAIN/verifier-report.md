# 独立 Verifier 审核

日期：2026-10-22。审核对象为 Coder 交接中冻结的三个源码；审核前后 SHA256 均与交接一致。结论：本轮限定诊断的静态/纯自检验收 PASS，无 BLOCKER，无需在本轮实测前修改的 finding。允许主协调者进入已授权的唯一一次实测；这不是实际 IPC 或游戏兼容性通过。

## 核对结果

- 固定 runtime 的 `attempt.json` 使用 CreateNew；前置失败也保留声明，不可由自定义 evidence 目录绕过。内部 worker 须匹配一次令牌与准备程序集 hash，并独立 CreateNew worker claim/attempt。
- 目标限定唯一精确名称实例、EXE 文件名及主程序集 hash，保留原实例查询对象并校验创建时间；检查唯一 Unity 窗口可见、非最小化、后台和有效客户区。连接后两次核验客户端身份，唯一写前重新核验窗口和实例。
- 管道使用 first-instance、最大实例 1、当前用户专用 DACL、拒绝远程；不连接或抢占已有服务端。唯一发送点为数量 1 的 `00 02 31 00` 四字节帧；单写预算、写前持久阶段与无重试路径成立。
- 原生 FlushFileBuffers 只操作自己的服务端句柄。写完成、drain 开始与完成分别留证；成功只表示传输层读走，失败/超时均不被解释为从未生效。
- launcher 只监督直接创建并持有的 worker Process 对象。两处 Kill 参数均为 false；无按进程名/PID寻找后终止、无杀树。所有 WaitForExit 限定为 2000ms。生产监督调用与纯测试使用同一策略，10 秒 drain / 45 秒总阈值与调度限制已披露。
- 阶段通过临时文件后原子替换，父读取允许 Delete 分享；看不到阶段仍受总监督阈值约束。正常 Dispose、确认 worker 退出后的 OS 回收与退出未确认分别记录。异常路径保留发送不确定性。
- 未发现全局输入、PostMessage、抢焦点、光标移动、进程内存/调试器或游戏文件写入调用。诊断只创建本任务文件和自有管道/worker，不接入产品。

## 独立执行证据

使用全新 `build/verification/TAPTAPLOOT-IPC-DRAIN/verifier/self-test/` 执行默认入口；编译 PASS，31 项纯测试 PASS。原子文件阶段 fixture PASS，两个 PowerShell 文件 AST 解析错误均为 0。TEMP/TMP/TMPDIR 在运行前及脚本中均指向仓库内任务临时目录。

审核结束时本任务 runtime 不存在。没有执行 RunOnce、真实 worker、真实目标查询、命名管道建连/写入、原生 drain 或进程终止。

## 证据限制

纯策略与文件 fixture 不能证明原生管道调用、实际 worker 强制终止、完整桌面输入隔离或游戏响应。10 秒和 45 秒是名义监督阈值，受系统调度及最多 2 秒退出等待影响；该限制已在 Coder 交接中明确，不新增理论加固要求。实际 drain/正常清理需读取本轮实测回执；玩法变化由用户人工观测，不能把前台采样或传输成功当作 gameplay PASS。

## 冻结快照

| 文件（相对本任务 coder 目录） | 审核前后 SHA256 |
| --- | --- |
| `Probe.cs` | `496EC393FEC281004512BD0D44E68CD059E8AC9B49E3F272842838612037956D` |
| `probe.ps1` | `950629BBA571355D244F0786900B4E5BA405782EABF653F892CA8AD2E8A49CA6` |
| `worker.ps1` | `4B2C23191F6FDBE110161539ADB4DD1FA01285E2272917CA02ECCEB7A791F130` |
