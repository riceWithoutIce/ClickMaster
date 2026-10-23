# TASK-003-CALCULATOR 独立限定验收

日期：2026-10-22。验收对象：`TASK-003-CALCULATOR-CODER-S1`。结论：静态与纯逻辑检查通过，无 findings、无当前阻塞项；可按契约由主协调者执行唯一一轮真实测试。本报告不证明目标定位、UIA 读取或后台输入已在真实计算器上成功。

## 冻结对象与独立证据

- 四文件清单 SHA-256：`4853AD7FB1C26D1FDB8E91C3B627A42A317DB62CFE8C5D8E993A6C4D7E4290E3`，独立核对与 Coder 交接一致。
- 验收副本：`build/verification/TASK-003-CALCULATOR/verifier/`；`CalculatorProbe.cs`、`run.ps1`、`worker.ps1`、`README.md` 四项与原文件 SHA-256 零差异，见副本 `copy-hashes.json`。
- 仓库根独立运行 `& ./build/verification/TASK-003-CALCULATOR/verifier/run.ps1 -SelfTest`，退出码 0。运行编号 `20261022T022245075Z-c84fdfa8d6ea466db6999f2c853e8106`：编译与 20 项纯/fake 断言通过，回执明确 `DesktopQueries=false`、`InputPosted=false`。
- 所有本轮受管理输出留在验收副本及其 `runs/` 内，编译 worker 启动前设置项目内 TEMP/TMP/TMPDIR；未下载工具或依赖。没有系统级文件监控，不能推断整个系统零外写。

## 审查范围与结果

- 候选范围限定计算器标题的 ApplicationFrameHost 框架及计算器进程白名单的 CoreWindow，要求可见、非最小化、非 cloaked 和唯一内容候选；身份捕获/复验覆盖 HWND、PID、线程、进程创建时间及父子/根关系。候选变化不会转向替代窗口。
- UIA 仅定位内容树内唯一 `CalculatorResults`，读取值/归属，严格要求基线为可识别的 0；不能读取、多个候选、前台计算器或身份变化均拒绝投递。没有 Invoke、SetValue、SetFocus、激活、清空或全局设备输入路径。
- 显式 `-ExecuteOnce` 才进入输入；默认预检不调用配对。固定主键区 1、目标线程扫描码、一次 WM_KEYDOWN/WM_KEYUP、约 50ms，释放通过 finally 对仍有效原目标尝试一次。按下失败不释放，身份无法验证不向替代目标补发；纯/fake 断言验证了相关分支。
- 观察循环仅只读，次数和时间有界，API 完整投递与 0→1 观察分开报告；无变化/异常/其他值不冒充响应成功。前后台/鼠标相同仅为瞬时比较，文档明确不能证明全程隔离或排除并发人工输入。
- 启动器唯一 `.Kill()` 对象为本次 `.Start()` 得到的自有 worker，不通过目标进程名/PID 杀进程；25秒 watchdog 和18秒发送期限不授权重跑。强制终止时不能保证释放、最后阶段可能未知，README 已明确限制。
- 回执仅序列化固定码、归一化 0/1/other/unreadable、计数、错误码与比较布尔值，不落盘原始标题、实际标识、UIA 树、历史、截图或原始异常。对四文件的本机路径/用户目录/典型凭据赋值文本检查无命中；扫描是辅助证据。

## 未执行与停止点

Verifier 未枚举真实窗口、调用真实 UIA、投递消息、启动 GUI/浏览器或运行产品构建；未修改 Coder 源码、未向 Coder 派修复、未提交/推送/清理历史。真实 DWM 可用性、进程名、UIA 控件/显示标签均为运行前提，实际不满足时应按既有门槛记录拒绝，不放宽条件或自动重跑。

本轮无 findings，当前审核结束。后续真实运行成功、未响应、预检拒绝或超时均应独立记录；不能将本报告的静态/纯测试通过替代实际计算器或游戏兼容性证据。
