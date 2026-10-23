# IPC-REPEAT 独立 Verifier 审核

日期：2026-10-22。结论：冻结源码与本轮纯验证 PASS，无 BLOCKER，无需在本次 single 实测前处置的 finding。可由主协调者进入已授权 single；repeat 必须等待本轮 single 正常终态和用户新的人工有效反馈，不能因本报告或旧轮反馈自动推进。

## 审核范围与关键路径

只审核本任务七个冻结源码、冻结契约与关键运行路径，未扩展为旧任务全面审计。源码清单 `build/verification/TAPTAPLOOT-IPC-REPEAT/coder/source-sha256.json` 的七项 SHA256 在审核前后全部相符。

- 阶段运行目录固定为本任务 runtime/single 与 runtime/repeat，分别 CreateNew attempt；真实入口拒绝 evidence 参数，默认纯入口排除真实 runtime。前置拒绝消耗该阶段，不重试/重连。内部 worker 要求本轮票据、程序集 hash、匹配任务/阶段及一次 worker claim。
- launcher 与 repeat worker 都调用同一 LoadSingleGate，要求本轮 single 恰一帧写/drain 成功、正常关闭、监督确认退出且未 kill、热键已注销。人工反馈须绑定同一 RunId 与三份最终文件 hash，且反馈时间在 single 监督结束后、不在未来。工具不会自行写真实反馈。launcher/worker 随后都比对同一目标身份摘要，不自动跟随重启或替换窗口。
- 共用 FrameSchedule 在 single 上限 1、repeat 上限 5；上一帧 drain 未完成时不允许发送下一帧。下一帧至少等前帧 drain 完成后 2 秒，满足发送开始间隔至少 2 秒；晚到只按当前时刻重新计时，不追赶。唯一 WriteAsync 点逐帧发送固定数量 1 的四字节帧。
- 每帧发送前重验保留的目标实例/创建时间、窗口资格及已连接客户端；限定 EXE、已知程序集 hash、唯一 UnityWndClass、后台/可见/非最小化/有效客户区。管道 first-instance/max1/current-user DACL/reject-remote 边界保留。
- 写前原子持久化保守 WriteAttempted，写完成与 drain 开始/成功逐帧留证；父读取允许 Delete 分享。drain 使用已修复的 UTC/小数生产链，按待处理帧 Index 重置计时，10 秒逐帧阈值、60 秒总监督共用生产策略。58 秒 worker 非阻塞令牌不取代父对同步 drain 的限制。
- F8 使用 RegisterHotKey，无 hook；注册失败分支不会创建 worker。注册、轮询和 finally 释放由 launcher 同一同步调用链持有，并校验原生线程身份；注册状态及最终注销结果分开记录，注销不成功不能打开 repeat 门槛。真实注册/送达仍待实测。
- launcher 处理取消/超时/目标失效后写停止标记，并仅终止直接创建且保留的 ownedWorker，Kill(false)，不扫名称/PID、不杀树；两处 WaitForExit 都是 2000ms。worker 在准备/连接/间隔/观察/逐帧前检查停止和目标资格。在途帧不能撤销，回执不把取消、超时或未知阶段说成从未发送。
- 正常 Dispose、确认 worker 退出后的 OS 回收、退出未确认分别记录。未发现键鼠/PostMessage、抢焦点、光标移动、读写游戏进程内存、调试器、游戏文件或产品写入路径。

## 独立执行证据

全新输出目录：`build/verification/TAPTAPLOOT-IPC-REPEAT/verifier/self-test/`。仅运行默认入口：

- 编译成功；51 项核心/调度/阶段门槛纯检查 PASS。
- 21 项生产 JSON/UTC/单调监督链检查 PASS，包含历史七位小数 UTC、另一 UTC、明确偏移、104ms、9.999/10 秒、原子 pending→completed 与下一帧 timer 重置。
- 2 项共用 New-StageAttempt 文件检查 PASS，single/repeat 各自重复拒绝且原文件不变。
- 三个 PowerShell 文件 AST 解析错误为 0。
- 独立重算修复前清单所列旧 IPC-POC、IPC-DRAIN 与产品/配置共 110 文件，全部 hash 不变。证据为 `verifier/checks.json`。

审核结束时真实 runtime 不存在。未执行 RunOnce、真实 F8 注册/发送、worker、目标查询、管道或真实发送；TEMP/TMP/TMPDIR 和全部新增编译/fixture/检查产物在仓库内。

## 尚未验证的真实边界

原生管道连接/逐帧写/drain、F8 注册冲突/送达/取消/注销、真实 worker 结束、single→repeat 真实连续性、游戏效果/计数及完整桌面隔离均没有实际证据。10/60 秒为名义监督阈值，受轮询、Windows 调度与最多 2 秒退出等待影响，不承诺硬实时。成功 drain 仅证明客户端读取传输缓冲，不证明解析或玩法。5 帧验证不构成长时间兼容性结论。以上边界不由纯测试冒充，后续应逐阶段单列实测和人工证据。
