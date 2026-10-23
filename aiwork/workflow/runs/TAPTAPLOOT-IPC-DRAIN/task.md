# IPC 客户端读取确认 POC

日期：2026-10-22。用户对“单次数量1、FlushFileBuffers确认客户端读走、最多10秒、隔离自有进程限制阻塞、人工计数对照、不改游戏、不循环不重试”回复“好的同意”，批准本轮限定开发与一次实测。保留既有IPC-POC两轮证据，不覆盖、删除或复用attempt。

## 目标与证据边界

区分服务器写入完成、客户端读取管道缓冲完成、游戏可见响应三个层级。FlushFileBuffers服务器成功返回是客户端读取缓冲数据的传输层证据，不是解析/事件分发/玩法确认；调用失败/超时不代表从未读取或从未产生作用。数量固定1，不能用数量N替代N回合。

## 范围与角色

- Coder唯一新增脚本写入者：`build/verification/TAPTAPLOOT-IPC-DRAIN/coder/`，独立任务交接/审核放本目录。不得修改生产源码、游戏、既有脚本/attempt/结果。可复用旧稳定脚本的身份/协议校验源码，复制入新目录后明确hash。
- 现有PowerShell7/.NET，无下载/全局安装；子进程、Add-Type前TEMP/TMP/TMPDIR均在仓库内。默认仅编译/纯自检，`-RunOnce`才真实操作。内部worker不得允许无本轮launcher原子attempt/授权令牌而运行真实目标；测试不启动真实worker或管道。
- 主协调者提供契约、核对稳定快照并执行一次真实launcher；Coder/Verifier不得查真实目标、建连真实管道或发送数据。Verifier对冻结脚本独立只读审核，可执行纯自检与假worker监督策略测试，不触及真实游戏/联动管道。findings一次性返回用户，由用户决定修复；不直接要求Coder修复。
- 旧API读取机制分析与IPC协议结论不变：[消费链分析](../../../debug/taptaploot-ipc-consumer-analysis.md)，[旧任务](../TAPTAPLOOT-IPC-POC/task.md)。

## 运行契约

1. 本任务固定runtime目录原子CreateNew声明attempt，前置拒绝也消耗本轮；不允许指定新目录绕过，不自动再执行。launcher只启动一个隐藏自有PowerShell worker，使用直接保存的Process对象持有身份，不按名称/PID扫描并结束外部进程。
2. 唯一精确名称TapTapLoot实例、原创建时间/存活/保留查询句柄、精确EXE与主程序集SHA `BC5318AC557E4039ADCCAE8D79EAF66F6C0200D18FC043FD2AA632F6513C35B8`、唯一UnityWndClass/可见/非最小化/后台/有效客户区校验沿用。新worker独立重新校验，不能信任旧PID/窗口。
3. 固定TypingFarmer通道，first-instance原子独占、max1、当前用户ACL、拒绝远程；冲突/身份不符立即停止，绝不连接/抢占已有服务器，绝不接触buff通道。
4. 至少5秒准备，连接等待最多20秒；客户端PID核验为原游戏实例并在唯一写前重新检查原实例及窗口。单帧 `00 02 31 00`，一次WriteAsync，最多2秒，不补发/重发。
5. 写入完成后持久记录`write-completed`与`drain-started`阶段，随后worker对自己的服务器管道调用原生FlushFileBuffers。launcher从观察到drain-started（包含可比较的UTC时间或单调时钟预算证据）起计时，最多10秒；即使同步调用阻塞也不能让主协调者无限等。成功返回记录`DrainCompleted=true`；native错误单独保留代码。终端/worker中断以持久阶段保守判断，不能冒称未发送。
6. 成功drain后保持连接约3秒给主线程处理/人工观察，有限只读采样目标是否前台。超时或worker挂起，launcher仅终止该次自己创建且持有的worker对象、不杀进程树、不按名字结束任何进程；等待其退出并明确记录OS回收自有资源与正常Dispose的区别。总worker监督预算最多45秒（含准备/连接/写/10秒drain/观察，需合理余量）；编译放在子worker前或通过准备模式避免未记录编译计入读取等待。轮询/退出等待有界，不调用无超时WaitForExit。
7. worker与launcher证据分别命名，阶段写入须足以被监督者稳定读取（原子替换/共享可读快照或可靠marker），避免partial JSON被当成未知并继续无限等。假fixture测试用任务内文件模拟阶段，不创建本次真实命名管道。
8. 全程不发送键鼠/PostMessage，不抢焦点、不移动鼠标、不改游戏、不读其进程内存、不附加调试器、不写其配置/存档。只结束自有诊断worker；结束时释放服务器/查询对象或在强制结束后记录OS回收，不把清理请求当实际完成。
9. 不落真实PID/HWND、窗口标题、用户名、本机绝对路径、原始异常堆栈或截图。输出本任务内脱敏摘要，保留实际错误类型/阶段及数量1证据。游戏原有行为/存档副作用按触发一次玩法可能存在，不伪称没有游戏内部写入。
10. 实测前主协调者提示用户暂停真实键鼠并留意计数/玩法变化，避免把正常输入当IPC效果。用户已授权本次操作，并明确说明回复本身会增加游戏计数，要求直接开始、由其人工观测；因此不再要求输入一个数字基线，也不再等待额外就绪消息。实际发送仍在稳定快照审核通过后预告，留至少5秒准备。人工反馈须区分“观察到变化”与精确+1证据；并发真实输入或无法确认时记不确定。新attempt只有一次，失败/冲突/超时结束，后续另行授权。

## 自检与审核

- 协议与原核心Gate/one-shot有意义纯测试；新增drain成功、drain失败、drain超时/无进度、父监督超时、错误清理结果不报正常关闭等策略检查。真实调用路径须使用与测试一致的策略或明确测试边界，不能另写无效断言凑数。
- launcher源码只允许终止本次直接创建的worker，review冻结snapshot及hash；真实RunOnce仅主协调者执行。
- 不要求真实假管道/目标fixture，不写镜像实现测试。静态/纯测试通过不是系统原生调用或游戏通过。
- 成功/失败均保留attempt、worker阶段、supervisor回执以及人工作用结果。达到本轮边界即停止，不接入产品。

## 技术门槛

Windows本地管道与已验证协议/客户端身份基线成立；FlushFileBuffers为既有Windows API，无新依赖；现有PowerShell/.NET可构建运行；build/verification与build/tmp落点、ignore/脱敏和自有worker边界明确。相对旧POC优势是新增读取层证据。安全/阻塞风险用隔离worker、有界监督和单写限制；实际读取/解析/玩法仍是待验。决策：有限POC已获用户批准，正式集成defer。

本轮已实时核对[Microsoft FlushFileBuffers 文档](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-flushfilebuffers)：对命名管道服务器端，该函数等待客户端读取全部已缓冲数据；非零为成功，零为失败。此语义要求单独限制同步调用阻塞时间，不能把成功返回描述为游戏解析成功。本轮以现有HTTP读取文档文本，无文件下载或浏览器页面/进程新增。
