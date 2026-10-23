# ClickMaster 工作记录

本目录保存 Windows 后台连点工具的需求、路线、开发日志和 AI 调试记录。工程、界面与绑定已实现；单次后台消息输入已完成构建和独立非 UI 复审，实际游戏响应待用户验收。重复与全局热键尚未开发，实验覆盖层问题保持延期。

## 文档入口

| 文件 | 用途 |
| --- | --- |
| [requirements.md](requirements.md) | 第一版冻结需求、排除项与待定参数 |
| [roadmap.md](roadmap.md) | 开发阶段、交付物与完成条件 |
| [workflow/README.md](workflow/README.md) | 主协调者、Coder、Verifier 流程、职责边界与交接模板 |
| [technology-selection.md](technology-selection.md) | 参考产品技术调查、候选比较与待定选型 |
| [ui-technology-selection.md](ui-technology-selection.md) | 圆角与动效要求下的 Tauri、WPF 等候选比较 |
| [frontend-selection.md](frontend-selection.md) | 已确认的 Vue 3、Vite、npm 组合、候选取舍与证据边界 |
| [runtime-distribution.md](runtime-distribution.md) | Windows、x64、WebView2 与免安装便携发布约定 |
| [local-data.md](local-data.md) | EXE 同级数据、开发隔离、升级与最小化补充约定 |
| [project-layout.md](project-layout.md) | 源码、构建、版本、发布目录及有限归档规则 |
| [devlog.md](devlog.md) | 按日期追加的开发记录 |
| [validation.md](validation.md) | 后台兼容性验证步骤与验收证据 |
| [debug/README.md](debug/README.md) | AI 调试记录规则 |
| [debug/_template.md](debug/_template.md) | 单次问题调查模板 |

## 当前任务对齐

[TASK-006-UI-CLEANUP：界面整理与首版发布准备](workflow/runs/TASK-006-UI-CLEANUP/task.md)已建档，当前仅对齐范围与发布决策。前置TASK-005重复/F8/统一停止已完成开发与独立审核，用户确认本轮人工验收通过，详见[验收回执](workflow/runs/TASK-005-REPEAT/acceptance.md)。下列历史条目保留当时状态；本任务尚未启动代码调整、构建或发布。

[TASK-003-ROUTING：内容子窗口输入路由](workflow/runs/TASK-003-ROUTING/task.md) 已实现并完成构建/独立限定复审，无findings：唯一明确CoreWindow自动识别、歧义选择、普通自身路径及左右键/键盘、鼠标绑定客户区坐标转换、接收身份失效停止。[本轮自动实测](workflow/runs/TASK-003-ROUTING/runtime-result.md)在计算器目标定位未唯一时停止，产品未启动/未发送；新EXE已交付，主程序实际响应待验，不以独立工具旧0→1结果替代。

用户随后自行使用主程序并截图确认“成功了”：自动识别CalculatorApp.exe / CoreWindow、单次数字1已投递且计算器显示1。[该项人工确认通过](workflow/runs/TASK-003-ROUTING/acceptance.md)，计算器键盘原问题关闭；自动化未执行到发送的历史结果保留，实际左右键/游戏和全程隔离仍待验。

[TASK-003：单次后台输入](workflow/runs/TASK-003/task.md) 已实现固定3秒倒计时/约50毫秒配对、左键/右键/单键与客户区位置；构建、相关测试及 [独立非 UI 复审](workflow/runs/TASK-003/verifier-report.md) 通过，无 findings。实际投递/玩法响应及隔离待用户使用 [本轮入口](workflow/runs/TASK-003/human-acceptance.md) 人工验收；重复/全局热键未实现，预览问题延期。

[TASK-001：Tauri 工程初始化与界面骨架](workflow/runs/TASK-001/task.md) 已完成 [Coder 交接](workflow/runs/TASK-001/coder-handoff.md) 与 [独立非 UI 验收](workflow/runs/TASK-001/verifier-report.md)，未发现 findings；[人工窗口与托盘验收](workflow/runs/TASK-001/human-acceptance.md) 已通过，剩余证据边界单独保留。

[TASK-001-UI：侧栏二级页面与应用图标](workflow/runs/TASK-001-UI/task.md) 已实现并完成 [独立非 UI 验收](workflow/runs/TASK-001-UI/verifier-report.md)，无 findings；本轮应用启动和 [桌面验收](workflow/runs/TASK-001-UI/human-acceptance.md) 继续由用户负责。

用户随后报告 EXE/标题栏仍为旧图标；[R1](workflow/runs/TASK-001-UI/r1-task.md) 修复并通过独立复审后，用户最终确认“验收通过”，本轮收尾。关闭按钮的托盘/退出二级选择已记录至需求与 roadmap，按用户要求并入下一任务，不单独修复。

[TASK-002：目标窗口选择与绑定、关闭行为选择](workflow/runs/TASK-002/task.md) 已实现并完成 [独立非 UI 验收](workflow/runs/TASK-002/verifier-report.md)，无 findings；[桌面验收](workflow/runs/TASK-002/human-acceptance.md) 待用户完成，后台输入尚未实现。

用户反馈后台窗口无法选中后，[R1](workflow/runs/TASK-002/r1-task.md) 已实现后台范围与即时筛选、绑定后隐藏保持；构建及 [独立限定复审](workflow/runs/TASK-002/r1-verifier-report.md) 通过，无 findings。真实后台目标和 UI 仍待用户使用 R1 新入口验收，不代表后台输入兼容。

R1 人工反馈仍未出现示例目标，经授权只读诊断确认工具样式被排除；[R2](workflow/runs/TASK-002/r2-task.md) 最小修复及 [限定复审](workflow/runs/TASK-002/r2-verifier-report.md) 已通过，无 findings。用户已确认可筛选、选择并绑定示例目标，原问题关闭；[人工记录](workflow/runs/TASK-002/human-acceptance.md) 保留其他未反馈场景待验，后台输入尚未实现。

## 使用约定

当前技术验证：[TASK-003-OVERLAY](workflow/runs/TASK-003-OVERLAY/task.md) 的实现/构建及独立非 UI 复审通过，但用户反馈没有可见图形，视觉 POC 未通过。源码复核未定位确定根因，按用户成本边界将 [OVERLAY-001](debug/overlay-visible-without-pixels.md) 延至主功能完成后处理；先关闭实验性预览，不自动换型，后台输入未实现。

- 需求以 `requirements.md` 为准；变更应记录原因和用户确认，避免实现过程默默扩展范围。
- 路线中的完成状态必须有对应记录或证据。编译成功、消息投递成功和游戏实际响应分别记录。
- 开发日志按时间追加，调试调查按问题单独建文件。
- 文件、文档及证据遵循 [脱敏约定](workflow/sanitization.md)：路径使用仓库相对路径或占位符，命令注明相对执行位置，不保存本机路径、用户标识、凭据或无关个人内容。
- 工程和骨架证据不代表后台输入已实现或目标兼容性已通过；未验证能力保持待验证。
