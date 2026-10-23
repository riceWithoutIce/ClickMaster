# TASK-012 UI 用户化整理：独立只读 Verifier 报告

日期：2026-10-23

## 审核边界

本次只读审核针对 Coder 交接的 `0.1.0-rc.8` 稳定候选，检查 `src/App.vue`、`src/style.css`、任务验收文件、候选元数据和校验值。没有修改产品代码，没有提交或推送，也没有运行长时间构建、真实游戏或人工 UI 操作。

## AC 结果

| 编号 | 结论 | 静态证据 |
| --- | --- | --- |
| AC-01 | PASS（静态） | `src/App.vue` 仅渲染固定窄图标侧栏；导航按钮没有展开/收起控件，`src/style.css` 固定侧栏宽度并提供窄屏宽度。 |
| AC-02 | PASS（静态） | `src/App.vue` 的 `.runbar` 集中显示状态、进度、已投递/已发送次数及“执行一次 / 开始 / 停止”操作；`.runbar` 位于主内容底部。 |
| AC-03 | PASS（静态） | `statusTone`/`statusLabel` 覆盖待命、准备中、运行中、停止中和错误/失败；`.status-running` 使用绿色文字与指示点，状态同时显示文字，不依赖颜色单独传达。 |
| AC-04 | PASS（静态） | 目标页常驻内容为筛选、隐藏/后台开关、窗口选择、刷新和绑定状态；PID 与接收窗口进入“窗口详情与接收设置”折叠区。候选列表默认仅显示标题与 EXE。 |
| AC-05 | PASS（静态） | “诊断信息”折叠区保留性能日志入口、错误/失败信息、IPC 连接/单帧写入/读取诊断、接收与按下/释放回执、执行摘要和发送计数说明。 |
| AC-06 | PASS（有限静态证据） | Coder 交接记录 `npm run typecheck`、`npm run build`、`npm run build:desktop` 通过；本次未重复构建，也未运行后端测试或运行时验收。未观察到 UI 源码中对 IPC、worker、输入语义或性能逻辑的直接改动。 |

## 候选追溯核验

- `build/staging/TASK-012-UI-USER-FACING/source.json` 声明版本 `0.1.0-rc.8`、dirty snapshot、89 个源码文件。
- `build/verification/TASK-012-UI-USER-FACING/coder/source-manifest.json` 的文件数、字节数和 SHA-256 逐项核对通过；manifest SHA-256 为 `99E0E5A8CD610FDD90605A6F5BF0ABE3F54EEB35A93D2B0465C0EF2FBB79A61C`。
- 候选 EXE 为 `9,772,544` 字节，SHA-256 为 `7EB0E7661BF4700C622416A7FE0F813717DD978CCB571862C9262DB74E4BE61E`；候选 ZIP 为 `3,221,700` 字节，SHA-256 为 `44E1CC66818311653E5469A9D2857ED894E1C482AEC9675367FBA29047AC88B8`。两者与 `source.json`、`SHA256SUMS.txt` 一致。
- ZIP 内容为 `ClickMaster.exe`、`README.md`、`CHANGELOG.md` 与 `docs/` 下两份说明，未发现候选外的构建输出被打入包内。

## Findings

### P2 — 版本说明中的“当前版本”仍为 rc.6

`version/README.md` 的版本说明仍写“当前 `0.1.0-rc.6`”，而 `version/VERSION`、package 元数据、候选来源清单和候选包均为 `0.1.0-rc.8`。这不会影响程序启动或本轮 UI 行为，但会误导版本追踪和后续发布准备。建议在正式发布或下一次版本元数据整理时同步为当前候选版本；本次 Verifier 不直接修复。

没有发现 BLOCKER 或 P1。未发现需要阻断本轮人工体验的静态问题。

## 未运行 / 不作通过声明

- 未进行人工 UI 视觉与交互验收，因此布局密度、颜色可读性、窄窗口适配仍待用户体验反馈。
- 未运行真实 TapTapLoot、计算器或其他目标窗口；不宣称输入响应、IPC 兼容性或目标失效保护通过。
- 未运行托盘、F8、关闭流程、长时间运行和性能采集；不宣称性能或资源占用通过。
- 未运行后端 Rust 测试；AC-06 的后端部分仍属于未运行证据。

结论：静态 UI 验收可进入用户人工体验，候选仍是未正式发布的 `rc.8` 本地包。
