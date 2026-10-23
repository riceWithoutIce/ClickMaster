# TASK-011 独立限定验收报告

日期：2026-10-23。Verifier：task011_verifier。结论：冻结范围源码、后端/helper、非游戏持久管道 fixture 与候选检查通过，**无新增实质 finding**。真实 TapTapLoot、普通 UI、鼠标 loading、F8、托盘与焦点均 NOT RUN，不能据此宣称游戏兼容或完整 UI 体验已通过。

## 来源和方法

对象为 `TASK-011-IPC-REUSE-rc.7-coder-final`，73文件dirty快照。manifest SHA256 `A3F36AD05CAB3D10FF1BC5DD5744AC8C8B6AC50503A146F7217050470B99F035`；EXE `4A6761F90C0894BABF8B4080B94F77393588CC49337F4FEA89325D4A128A784E`；ZIP `66D35133DAE371F7C68203D5A1D8F9417552E465B9F1DE2D2705EC0FDC70D934`。源码和包在准入与限定审查结束时一致。

独立输出在 `build/verification/TASK-011-IPC-REUSE/verifier/`。`check.mjs backend` 通过统一 environment.mjs 运行 Cargo release/offline/locked/no-run，退出0；复制库测试EXE到Verifier自有bin，仅为该副本嵌Common Controls v6清单，退出0；随后直接运行该副本，**100 passed / 0 failed / 2 ignored，退出0**。两个ignored为驱动测试所调用的私有子进程入口。未再重编译覆盖已嵌清单EXE，没有修改生产文件或Coder证据。`check.mjs helpers` 的前端helper与typecheck均退出0。

`package-check.mjs`、`zip-check.mjs`、`dependency-check.mjs`、`sanitization-check.mjs`、`final-evidence.mjs`最终均退出0。独立脚本首次编写出现fixture目录误指、正则解析和PowerShell语法错误；这属于Verifier辅助程序错误，随后只修正Verifier自有脚本，未重跑后端、未变更生产断言。一次含条件删除的归档展开命令被策略拒绝且未执行，后改为只读列举ZIP。原始工具输出保留于会话。生产首次失败没有被这些辅助检查替代；Coder的97/3/2及loader失败仍按原交接保留。

## 限定条目

| 条目 | 结果与边界 |
| --- | --- |
| AC-01/02 持久连接与失败 | 重复路线在监督线程保留单个PersistentSession/Child/pipe，单次仍用原路径。每条命令逐帧放行；生产sequence observer等待当前序号，旧完成回执不会重复计数或隐藏worker退出。断开与背压fixture通过，不重连、不重试、不回退。fixture直接复用生产send_persistent_frame与wait，不冒充完整Tauri授权/游戏路径。 |
| AC-03 间隔 | UI number最小值、TS helper与Rust Request统一1～60000ms；固定与随机范围、非法值、空值、反序、非整数、NaN/Infinity、加法溢出及有界拒绝采样测试通过。默认仍2000ms，文档明确完成后额外等待、非1kHz承诺。消息模式紧邻回归通过。 |
| AC-04 等量证据 | 独立100ms/1ms各24帧、单worker/单连接，平均间隔与CPU/墙钟从Verifier自产JSON复算，见下表与final-evidence.json。没有每帧重新建连。短时非游戏结果不代表真实目标消费速度或长期资源峰值。 |
| AC-05 停止与界面边界 | 连接/写入等待在ipc-supervisor及受控child，未放到Tauri UI线程。连接等待检查取消/期限/原目标，发命令前再检查；完成写计数在清理后收集，清理失败锁定。持久会话未套单次60秒预算，父进程死亡监视保留。自有Child句柄清理与不误停无关子进程fixture通过；实际按钮/F8/退出/tray/loading仍由用户验收。 |
| AC-06 保护与诊断 | 授权使用私有stdin长度前缀、token/EXE hash、原父进程身份和claimed记录；worker创建时支持版本/原目标Lease验证，逐帧保留唯一实例/窗口与实际pipe客户端写前核验。读取确认继续只诊断，不阻断。performance生产模块未改；既有collector开关/生命周期/状态分组与IPC单次紧邻测试通过。持久worker跨采集epoch的阶段覆盖限制不能当作完整逐帧性能归因；资源采样仍独立。 |
| AC-07 冻结与包 | 73文件、EXE/ZIP哈希核对一致。ZIP仅5条目：EXE、README、USAGE、LIMITATIONS、CHANGELOG，无data/log/cache/debug。Cargo/前端锁与manifest按rc.7回替rc.6版本即重建旧哈希，无依赖升级；18旧候选、652历史项、4用户原日志逐项哈希一致。已知本机前缀扫描通过，不声称穷尽所有隐私或全系统零外写。PE/图标/版本/未授权worker拒绝沿Coder已声明证据；本轮不启动普通候选。 |

## 独立 fixture 复算

| 配置 | 帧 / worker / 连接 | 平均帧间隔 | 派生频率 | 总墙钟 | worker CPU |
| --- | --- | --- | --- | --- | --- |
| 100ms | 24 / 1 / 1 | 104.719ms | 9.549Hz | 2.484029s | 0.031250s |
| 1ms | 24 / 1 / 1 | 3.485ms | 286.955Hz | 0.125914s | 0.015625s |

均值由23个完成帧间隔求和除23，频率取均值倒数；总墙钟包含启动与清理，CPU为自有child原句柄计量，不把墙钟等待解释为CPU。该测试接收器立即消费，未包含真实游戏约90ms读前睡眠，也不含完整应用或UI调度成本。此前同步消息中未复算的估算值以本表和final-evidence.json为准。Coder独立测量107.426ms/4.410ms仅为另一轮样本，不能混作本次数据。

Verifier无读取背压fixture写入1024个完整帧后，约2.013秒报告native write timeout，读取0帧、重连0次；断开测试失败写入、不追加第二次写。有限内核缓冲不等于目标已消费，停止后已写帧不能撤回；真实消费者静态90ms读取等待及尾效应已在候选文档披露，不恢复客户端读取确认阻断。

## 未运行与停止

真实游戏同连接连续帧、实际计数/消费、1ms长时间积压、鼠标loading、UI响应、F8、托盘/恢复、焦点与完整退出体验均NOT RUN。长会话不误套60秒为源码/调度helper证据，本轮未实际执行超过60秒游戏会话。没有游戏固定管道操作、普通UI/浏览器启动、全局输入、提交推送或正式发布。

V008-01未修未处置，OVERLAY-001继续延期。本轮无新增BLOCKER/FOLLOW-UP，不向Coder追加工作；限定检查完成并停止。rc.7可作为本地人工候选，不能标为真实游戏或正式发布通过。
