# IPC 读取确认验收记录

日期：2026-10-22。状态：唯一一次实测已结束；用户反馈“成功了”。人工可见响应确认有效，客户端读取确认因监督计时缺陷未取得。

用户已批准[本轮契约](task.md)，并说明发送文字本身也会增加游戏计数，因此不提供数字基线，要求由主协调者开始后人工观测。此变更不是取消观察，而是不再要求输入一个会随操作变化的数字；实际效果与精确因果仍分别记录。

| 项目 | 状态 |
| --- | --- |
| Coder稳定快照/编译纯测试 | PASS：编译、31项纯测试及原子文件fixture；快照hash一致 |
| 独立Verifier限定审核 | 静态/纯测试PASS，无finding；实际计时缺陷随后在实测暴露 |
| 主协调者真实连接/客户端身份 | PASS：目标客户端身份通过 |
| 单次写入数量1 | PASS：WritesAttempted=1，WriteCompleted=true；未重试 |
| FlushFileBuffers读取确认 | 未取得；DrainCompleted=false，监督提前结束，不能解释为客户端未读取 |
| 自有worker/服务器资源释放 | 已确认自有worker退出，由OS回收其句柄；不是正常Dispose完成 |
| 用户人工计数/玩法变化 | 用户回复“成功了”，可见响应人工确认；未分别说明计数/战斗、未取得精确+1对照 |
| 焦点/其他工作窗口影响 | 仍待确认；观察采样未执行，0次前台样本不代表全程未前台 |

主协调者保存的26个产品文件SHA-256结束复核全部一致，见 `build/verification/TAPTAPLOOT-IPC-DRAIN/coordinator/product-after.json`。未修改产品或游戏、未发送键鼠或窗口消息、未提交推送发布。旧IPC-POC两轮attempt/结果保留不动。

## 本轮唯一实测

仓库根执行一次 `& ./build/verification/TAPTAPLOOT-IPC-DRAIN/coder/probe.ps1 -RunOnce`。发送前提示用户暂停真实键鼠；准备阶段至少5秒。北京时间12:41:38.937左右记录写入完成及drain开始；12:41:39.041监督回执结束。原始回执保留在本任务 `runtime/worker-result.json` 与 `runtime/supervisor-result.json`，不因后续诊断或人工成功反馈改写。

监督回执原Outcome为 `drain-timeout`，但该标签不是有效的10秒读取超时证据：drain开始至监督结束仅约104ms，worker监督总用时约5.754秒（含准备）。主协调者只读复现：当前PowerShell的ConvertFrom-Json将时间自动转为Utc DateTime，再传入DateTime.Parse时隐式转字符串丢失时区与小数；二次ToUniversalTime使时间提前约8小时，触发立即超时。记录在 `coordinator/timestamp-diagnosis.json`；仅使用已有文件与纯时间计算，无新增目标查询或发送。

## 本轮发现与停止

`DRAIN-001`：诊断监督的UTC时间处理错误，正常读取确认流程不可用；阻塞该工具后续复用及“真实等待10秒后超时”的证据声明，不推翻用户观察到响应。冻结源码与回执保留，本轮不自动修复、不重跑。此前独立纯测试仅覆盖监督策略数值，未覆盖JSON日期反序列化到生产计时链，不能作为该链正确的证据。

用户反馈仅记人工可见响应，不扩大为完整玩法/统计等价、持续运行、全程输入隔离或正式集成通过。固定attempt已消耗，本轮到此停止。若后续继续，先由用户决定计时修复及新一轮验证/产品集成范围。

## 后续限定修复完成

用户同意继续“先修计时，再对齐低频验证”，按[修复契约](repair-task.md)完成新 `coder-r1/` 快照。原快照/runtime不改写，产品不集成。[Coder交接](repair-coder-handoff.md)与[独立限定复审](repair-verifier-report.md)确认DRAIN-001源码修复关闭，无新增finding。

31项核心测试、13项生产JSON时间链回归及原子文件fixture均通过，PowerShell AST零错误。实际生产时间链保留UTC和小数精度，历史104ms不再映射为8小时，9.999秒继续、10秒才判超时；相邻Math.Max使用double重载，避免小数取整。61个旧证据与产品文件hash复核未变，主协调者再次核对三个新源码hash一致。

本修复轮无真实入口/worker/管道/目标查询或发送，旧单次读取确认仍为未取得，不因修复补记PASS。旧attempt继续阻止再执行；新实测尚未授权。[下一轮建议](next-validation-proposal.md)待对齐，不自动开始。
