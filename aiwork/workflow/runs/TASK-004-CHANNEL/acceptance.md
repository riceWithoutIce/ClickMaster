# TASK-004-CHANNEL 验收记录

日期：2026-10-22。Coder已稳定交接，独立Verifier PASS，无BLOCKER；本地包交付后用户反馈“测试成功”，主程序IPC单次游戏响应人工通过。主协调者未打开普通UI或连接游戏；其他待验项目不视为通过。

| 项目 | 状态 |
| --- | --- |
| 通道选择及设置联动 | 实现已冻结，类型/辅助策略检查通过；UI待用户 |
| 默认值、忙碌锁定和既有配置兼容 | Coder及独立Verifier相称检查通过 |
| Rust IPC协议、实例/版本/客户端核验 | 纯回归/非游戏fixture及独立源码审核通过；用户已确认本次主程序IPC响应，不扩大为全部拒绝场景通过 |
| 停止、超时、退出及自有worker清理 | 独立含阻塞drain/owned Child fixture通过；真实桌面行为待用户 |
| 原消息模式和内容路由回归 | 独立核心/辅助回归通过；实际计算器回归待用户 |
| 构建、本地测试包及图标/脱敏 | Coder离线正式模式构建、图标及已知路径扫描通过 |
| 稳定快照及独立Verifier | PASS，无BLOCKER；审核前后54文件及包hash一致 |
| 主程序IPC单次游戏响应 | 用户反馈“测试成功”，人工确认通过 |
| 工作窗口输入、鼠标、焦点隔离 | 待用户 |
| 旧IPC证据保留 | Coder188文件、主协调者159文件清单复核不变 |

范围见[任务契约](task.md)，交互见[示意与建议](proposal.md)。本轮不包含重复调度、全局停止热键、覆盖层修复、提交推送或正式发布。交付前不提供未冻结的EXE入口作为验收对象。

## Coder稳定快照

见[交接](coder-handoff.md)。`TASK-004-CHANNEL-coder-final`：54文件清单SHA256 `0AA24949D0915A816E9197BAD8F9D00AB1F44C2FBD6CDE8DE4CA8865A99C04C3`；最终EXE 9,364,992字节，SHA256 `AE4E289C0DB31D0C5C35DD41421800A1B4EC5DBF092EC79EF354703375E4FCD1`。源码清单位于本任务build下coder/source-manifest.json，主协调者核对结果位于coordinator/handoff-check.json。

Coder最终52 passed / 1 ignored（父测试调用的fixture入口），类型/辅助策略/正式构建/PE图标通过。仅无授权worker入口拒绝和非游戏fixture运行过，未启动普通UI或真实游戏操作；不将独立C#工具人工成功转记本包通过。

## 独立审核与本地交付

[Verifier报告](verifier-report.md)：PASS，无BLOCKER，无需当前处置的新增finding，审核停止。独立类型/生产helper/52项后端回归及PE图标检查通过；1个ignored为父测试调用的fixture入口。188份Coder旧证据和159份主协调者子集清单均未变。实际游戏响应、桌面停止/退出/托盘及隔离保持待用户，不以非游戏fixture替代。

入口：`build/verification/TASK-004-CHANNEL/coder/final/ClickMaster.exe`。仅此EXE，沿用既有WebView2运行前提，无.NET依赖；运行后数据在EXE同级data，不是正式release。用户按[人工步骤](human-test.md)自行操作：关闭旧ClickMaster，打开本包，绑定TapTapLoot，输入方式选专用IPC，再点击单次测试；3秒准备后观察游戏。不要同时使用旧独立IPC测试器，不使用本轮未开放的重复/F8能力。读取完成与玩法响应分开反馈。

主协调者最终复核54源码、包及159历史清单一致；未自动启动普通UI、查询或连接实际游戏，无额外发送、提交、推送、发布或旧产物清理。

## 用户人工确认

2026-10-22，用户在收到本轮新EXE及“绑定TapTapLoot → 选择专用IPC → 单次测试”的步骤后反馈“测试成功”。据上下文记录主Tauri程序专用IPC单次可见游戏响应人工PASS；这与旧独立C#测试器证据分开，不重写旧失败和暂停记录。

用户没有逐项反馈停止、退出、目标失效、托盘、计算器消息回归或完整工作窗口输入/鼠标/焦点隔离，这些仍待验。不据此证明精确增加量、统计/成就等价、长期稳定、其他游戏或版本兼容；用户实际运行文件路径/hash未另行核验。本轮仅更新记录，不追加发送、构建或代码修改，不开始重复与热键开发。
