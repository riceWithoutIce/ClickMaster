# TapTapLoot IPC 消费链只读复核

日期：2026-10-22。用户在第二轮IPC无效果后同意继续只读分析，并进一步确认：角色/战斗画面和明确点击计数两项均无变化。本轮未创建或连接管道、未发送新数据、未读取进程内存、未附加调试器或改游戏/产品。

## 方法与证据

- 针对当前唯一TapTapLoot实例安装目录中的主程序集读取PE/IL，SHA-256仍为 `BC5318AC557E4039ADCCAE8D79EAF66F6C0200D18FC043FD2AA632F6513C35B8`。使用元数据解析器，不加载执行游戏程序集。
- 单独诊断目录 `build/verification/TAPTAPLOOT-IPC-CONSUMERS/`：`inspect.ps1`、`result.json`；解析5677个方法体，保留195个与输入、计数、回合或IPC相关的方法明细。原只读结果及两轮运行回执未覆盖。
- 只读检查由app.info定位的现有Player.log末尾最多1500行，实际521行；仅输出异常/IPC标记摘要，无相关命中。`log-markers.ps1` / `log-markers.json`保存脱敏结果，不复制原日志、路径或个人内容。
- 临时编译目录按进程在编译前设为仓库内build/tmp；脚本、摘要和IL证据的盘符路径聚焦扫描通过。没有系统级零外写监控。
- 入口（仓库根）：`& ./build/verification/TAPTAPLOOT-IPC-CONSUMERS/inspect.ps1` 与 `& ./build/verification/TAPTAPLOOT-IPC-CONSUMERS/log-markers.ps1`。这两个入口不运行原POC的RunOnce。

## 数据到事件

```text
TypingFarmerThread
  -> StreamString.ReadString
     -> 两次ReadByte组成大端字节长度
     -> 单次Stream.Read读取负载
     -> 默认UnicodeEncoding解码
  -> 非空字符串 Int32.Parse
  -> _taps += 数量
Ipc.Update
  -> 先处理_buff（若存在）
  -> _taps > 0 时 GlobalKeyHook.Sim(_taps)
  -> _taps = 0
GlobalKeyHook.Sim
  -> OnKeyPressed?.Invoke(数量)
```

复核未发现握手令牌、必须前台、特定按键号或累计数量门槛。原四字节帧仍匹配此安装的静态读取格式；这不证明实际收到完整负载。Stream.Read的返回数量没有校验，且没有应用层确认。

TypingFarmerThread的异常区域包括TimeoutException与System.Exception；后者丢弃异常并等待约1秒再继续，不记录相关日志。故读取/解析异常可能没有日志；本轮没有证据证明它实际发生。不能因为日志无命中就认为解析通过。

## 事件到可见反馈

| 消费者 | 直接IL证据 | 限制 |
| --- | --- | --- |
| ClickCounter | OnEnable订阅，OnDisable取消；OnKeyPressed调用Add(数量)，Clicks直接加数量，再RefreshUI写TMP文本；另累加TTLSteamMultiplayer.TapsToSend | 组件启用状态和实际事件执行未读取；Add中数量>10仅控制弹跳动画，不阻止数量1的计数更新 |
| TickClock | OnEnable订阅；OnKeyPressed将s_tick加1并广播OnTick，不按参数循环 | 一次事件推进一个tick；不能推断发数量N就推进N回合 |
| TurnScheduler | 订阅OnTick；Resolve在IsResolving=false时直接返回；初始化/结束状态设false，主游戏状态设true；只处理活动且启用的角色，也有锁定动作处理 | 角色没动可以受游戏状态影响，但不能单独解释启用计数器不加1 |
| StageManager | OnTick在IsPaused时返回；下一波还需生成状态、敌人数和有效tick等条件 | 单次事件不能保证换波/明显战斗动画 |
| BongoCat.Cat | Tap通知OnTap并调整左右动作/协程 | 当前场景是否有该订阅者未确认，不作为唯一验收指标 |

Sim路径没有直接调用真实键鼠Update尾部的Statistic.AddClick/AchievementManager.AddStatProgress，保持先前“统计不保证等价”的结论。

## 本轮结论与停止边界

1. 用户已观察计数和战斗均无变化；不能再仅以动画不明显、单次数量小或角色状态解释为成功。
2. 在订阅组件正常启用、无中途异常且事件被实际触发的条件下，数量1应足以使ClickCounter增加1；未发现必须放大数量才能计数的静态依据。
3. 两轮已有回执证明目标客户端身份及服务器单次写入完成；不证明游戏读完字节、Int32.Parse完成、Ipc.Update执行或OnKeyPressed分发成功。
4. 目前没有足够证据给出唯一根因。读取/解析问题、实例/组件运行状态、跨线程数据交接、Update早期异常或订阅者异常均只能视作未定位环节，不把理论竞态当成已复现缺陷。
5. 本轮只读调查完成并停止；不追加输入、不加大数量、不调用其他通道、不改游戏或接入产品。

若继续验证，建议先另行冻结一次“消费证据”POC：仍只发数量1，但分开记录服务器写入完成、客户端读走全部帧的传输层证据、用户计数前后值；可使用隔离自有worker对管道drain设硬期限，禁止无限阻塞或重发。drain完成仍不能代替游戏解析/事件确认，失败也可能已读部分数据。当前仅建议，未实现/运行；具体连接保持时长、worker结束方式和人工观察基线需在新授权前对齐。

## 后续读取确认POC与人工结果

上述建议随后经用户授权进入独立[TAPTAPLOOT-IPC-DRAIN](../workflow/runs/TAPTAPLOOT-IPC-DRAIN/acceptance.md)。单次数量1写入完成，目标客户端身份通过；用户反馈“成功了”，记本轮人工可见响应有效，不改写前两轮无响应的历史。

原生drain未取得完成回执：监督工具因JSON时间二次转换错误，在drain开始后约104ms提前结束自有worker，原 `drain-timeout` 不能作为真实等待10秒证据。此结果既不证明游戏未读取，也不代替解析/事件确认；计时缺陷DRAIN-001待用户决定后续修复。无新重试、数量放大或产品集成；精确+1、计数/战斗分别变化、持续有效和完整输入隔离仍未确认。

随后用户批准计时修复，[独立限定复审](../workflow/runs/TAPTAPLOOT-IPC-DRAIN/repair-verifier-report.md)通过，DRAIN-001源码问题关闭。修复轮仅做生产JSON时间链纯回归，没有新实测；上述读取/持续/隔离证据边界保持。

再后用户授权[IPC-REPEAT分段验证](../workflow/runs/TAPTAPLOOT-IPC-REPEAT/acceptance.md)，新single已获得目标客户端身份、单帧写与原生drain完成及正常释放证据，故“客户端读取传输缓冲”这一证据缺口已在该新尝试取得；不改写旧attempt。用户未看清本次效果并明确要求暂停，未执行repeat、未记本次玩法通过。解析/事件确认、持续响应和完整桌面隔离仍未验。

## 独立EXE人工验证结果

2026-10-22，用户改用[独立IPC测试EXE](../workflow/runs/TAPTAPLOOT-IPC-EXE/acceptance.md)自主测试后反馈：“测试通过 单次测试、低频测试都有响应”。该EXE的单次与低频可见游戏响应记为人工PASS，支持在本次游戏环境继续评估IPC路线；此前失败、提前结束及暂停记录保持原状。

本反馈不提供游戏内部解析/事件执行跟踪，也不单独证明精确增加量、全部5帧间隔、统计/成就等价、长期稳定或完整桌面隔离。停止/F8/关闭与目标退出尚未单独确认；未追加实测或接入主程序。

## 主程序集成人工结果

2026-10-22，[TASK-004-CHANNEL](../workflow/runs/TASK-004-CHANNEL/acceptance.md)将协议移植到Rust并接入主程序通道选择，构建、相称回归及独立审核通过后交付用户。用户按本轮IPC单次测试上下文反馈“测试成功”，记主程序IPC单次可见游戏响应人工PASS，区别于旧独立工具结果。停止/退出/目标失效、完整桌面隔离和消息回归未单独确认；本次只记录反馈，没有新增发送或内部事件跟踪，不扩大为统计等价、持续或任意版本兼容。
