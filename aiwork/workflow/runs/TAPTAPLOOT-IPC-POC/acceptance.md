# TapTapLoot IPC POC 验收状态

日期：2026-10-22。当前：诊断稳定快照、独立限定审核通过；首轮及用户重新授权的第二轮单次写入均完成，资源已释放。用户反馈第二轮游戏没有响应，玩法兼容未通过；停止本轮POC，不自动重发或接入产品。

| 证据项 | 状态 |
| --- | --- |
| 协议静态核对 | 已核对：数量字符串1，单帧 `00 02 31 00` |
| 诊断实现与纯测试 | Coder 编译及19项纯自检 PASS，稳定脚本见交接 |
| 稳定快照独立审核 | Verifier 编译/19项/独立协议解码 PASS，无findings，前后hash一致 |
| 真实目标唯一/客户端身份 | 实测校验通过，客户端为原目标实例 |
| 实际单次写入 | WritesAttempted=1、WriteCompleted=true；只证明传输写入完成，无应用ack |
| 自有资源释放 | PipeDisposed=true、ProcessDisposed=true；仅释放测试自有管道/查询对象 |
| 游戏玩法/计数/统计响应 | 第二轮用户反馈“游戏没有响应”，玩法兼容未通过；具体计数/统计未单独确认 |
| 工作窗口/焦点/真实鼠标隔离 | 未确认，采样不能替代全程证明 |

脚本与实测阶段按[任务契约](task.md)推进。主协调者已在 `build/verification/TAPTAPLOOT-IPC-POC/coordinator/product-before.json` 保存26个产品源码/资源文件的仓库相对路径与 SHA-256，供结束时核对；未覆盖既有 dirty 内容。

## 唯一实测结果

- 冻结快照：`TAPTAPLOOT-IPC-POC-CODER-S1`，执行前核对两脚本hash与[交接](coder-handoff.md)/[独立审核](verifier-report.md)一致。
- 2026-10-22 04:19:34.8668030Z 至 04:19:43.3584099Z（本地12:19:34至12:19:43），约8.49秒。仓库根执行 `& ./build/verification/TAPTAPLOOT-IPC-POC/coder/probe.ps1 -RunOnce`，退出码0。
- 阶段：preflight → 5秒准备 → listening → client-verified → write-attempted → write-completed → closed。`Outcome=transport-write-completed-response-unverified`，Error=null，NativeError=0。
- 连接与目标检查通过，仅写入一次帧 `00 02 31 00`；没有重复、补发、窗口消息或系统键鼠输入，没有产品/游戏文件修改。
- 写入后30次、约100ms间隔采样，目标前台采样数0。不等同全程焦点未变，也没有观察工作窗口控件或记录鼠标位置/用户按键。
- 回执：`build/verification/TAPTAPLOOT-IPC-POC/runtime/attempt.json`、`runtime/result.json`。本轮真实尝试已使用，不删attempt重试，不因无视觉效果重跑。
- 后置复核26个产品文件 SHA-256 全部保持一致；实际回执盘符路径聚焦扫描通过。没有系统级零外写证据。
- 尚待用户确认：本地12:19:40左右游戏有无一次明确响应，工作软件有无额外输入、鼠标/焦点是否异常。真实操作也能推进游戏，无法判明则记“响应不确定”，不能用传输成功补成玩法通过。

## 用户重新授权第二次单次观察

- 用户明确表示首次未注意并要求“再测试一下”，授权一次新的单次尝试；不是自动重试。首轮 `runtime/attempt.json` 与结果原样保留，未删除或覆盖。
- 复用独立审核通过的冻结脚本，执行前两脚本hash核对一致，并确认首轮自有资源释放回执。主协调者设置项目内TEMP/TMP/TMPDIR，直接编译同一Probe.cs、仅调用一次RunOnce，使用新的 `runtime-r2/`。该目录含本次授权摘要、独立attempt和result；仅针对本次用户新授权使用，不改变原脚本固定首轮入口或防重跑策略。
- 时间：2026-10-22 04:21:51.3302896Z 至04:21:59.6429591Z（本地12:21:51至12:21:59）。约12:21:56完成单次写入。
- 结果：client-verified → write-attempted → write-completed → closed；WritesAttempted=1、WriteCompleted=true、ClientIdentityVerified=true、PipeDisposed=true、ProcessDisposed=true，Error=null。30次采样目标前台数0。
- 仅数量1，无循环、补发或系统输入。游戏响应与用户隔离仍等待本轮观察，传输回执不替代玩法通过。第二轮attempt已使用，不自动再试。

## 第二轮人工反馈与停止

- 用户随后明确反馈“游戏没有响应”。记录第二轮单次IPC未取得可观察玩法响应；原GameResponse字段为运行时未观察的原始回执，不回写为自动检测失败。
- 两轮客户端身份及传输完成证据保留；写入没有游戏消费确认，不能证明字符串已解析或事件已触发，亦不能确定具体统计/回合差异。
- 工作窗口额外输入、鼠标和焦点没有本轮单独反馈；30次无目标前台采样不能补成完整隔离通过。
- 达到本轮停止条件，不重复同帧、不增大数量、不尝试其他联动通道、不改协议碰运气、不接入ClickMaster，不升级全局输入或注入。若继续调查，先对齐只读消费链分析和可观察效果；未取得新证据前不进行集成。

## 后续只读消费链复核

用户同意只读复核，并明确说明战斗画面与点击计数均无变化。确认ClickCounter收到事件时直接加数量1并更新文本，不存在必须大于10才加计数的门槛（大于10仅弹跳动画）；角色/关卡动作另受调度和暂停等限制。读取线程吞掉部分异常，无应用ack，现有Player.log相关标记无命中不能证明解析成功。见[消费链分析](../../../debug/taptaploot-ipc-consumer-analysis.md)。本次没有第三次发送；根因未唯一定位，保持POC未通过。
