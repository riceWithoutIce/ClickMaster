# TASK-003-ROUTING 限定自动实测结果

日期：2026-10-22。快照：`TASK-003-ROUTING-CODER-S1`。执行角色为主协调者，独立Verifier无findings后执行一次，未重试。

## 入口与证据

仓库根执行：

```powershell
& ./build/verification/TASK-003-ROUTING/coder/automation/run.ps1 -ExecuteOnce
```

运行编号：`20261022T032357871Z-2a33922bd85a4be4a7ce728207c0e869`。

脱敏回执：`build/verification/TASK-003-ROUTING/coder/automation/runs/20261022T032357871Z-2a33922bd85a4be4a7ce728207c0e869/result.sanitized.json`。

| 字段 | 实际值及解释 |
| --- | --- |
| Outcome / Phase | not-sent / stopped |
| Reason | calculator-content-not-unique |
| Baseline / Expected / FinalDisplay | unreadable（未进入结果读取） |
| SingleInvokeAttempted | false，未触发产品单次入口 |
| ReceiverCoreWindowVisibleInUi | false，未进入产品界面检查，非实际界面失败证据 |
| ProductReceiptDelivered | false，未发送，不是投递失败 |
| ForegroundSame | null，未进入采样阶段 |

原因码只表示合格计算器内容目标没有得到唯一结果，回执未保存具体计数；不能断言是零候选还是多个，也不能根据前次其他生命周期的成功推断当前状态。helper在FindCalculator阶段停止，未启动新ClickMaster、未操作公开UI、未投递任何输入，不存在本轮owned产品残留。启动器退出码0只说明诊断结束，不是产品响应通过。

## 交付结论

- 产品接收路由实现、构建及独立限定非UI复审已完成；本轮17文件清单及EXE hash在执行前与交接一致。
- 主程序公开UI自动验收没有开始，计算器实际响应待验证。不得用旧独立CoreWindow工具0→1结果代替主程序通过，也不能把本次定位拒绝当作产品路由失败。
- 已保留本轮产物，按契约一次停止，不调整门槛、不追加输入、不再自动查询/重跑。后续实测由用户保持计算器可用后单独开始，或用户用新EXE人工验收。
- 左右键真实响应、游戏兼容、全程焦点/鼠标/其他窗口隔离仍未验证。OVERLAY-001延期，不提交/推送/发布。

## 后续用户人工结果（2026-10-22）

用户随后自行使用主程序并反馈“成功了”，截图可见自动内容接收CalculatorApp.exe / CoreWindow、键盘1两阶段已投递、计算器显示1。该项独立记录为用户人工确认通过，见[人工验收补充](acceptance.md)；不改写上述自动尝试结果，不声称自动化取得响应证据。原计算器数字键未响应在本次人工路径关闭，其他未反馈验收仍待验。没有追加运行或代码修改。
