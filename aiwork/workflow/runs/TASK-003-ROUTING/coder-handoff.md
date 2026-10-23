# TASK-003-ROUTING Coder 交接

日期：2026-10-22。快照：`TASK-003-ROUTING-CODER-S1`。Coder 已停止修改，等待独立 Verifier；未查询真实窗口、UIA、运行新EXE或投递输入。

## 实现

- 新增 `src-tauri/src/routing.rs`：绑定根与接收窗口分离，普通自身、唯一内容自动、多候选token选择、父子/身份/epoch失效停止；加入纯fake覆盖。
- 新增 `src-tauri/src/routing_native.rs`：只读识别 `ApplicationFrameWindow` 下可见非cloaked `Windows.UI.Core.CoreWindow`，不按名称追踪替代窗口。
- `input.rs`/`input_native.rs`：请求快照同时携带接收身份；键盘使用接收线程；鼠标从绑定客户区在同一PMv2上下文转换到接收客户区，映射失败、接收越界或16位溢出拒绝；receipt保留接收摘要。
- `targets.rs`/`lib.rs`：绑定时建立路由、路由候选确认IPC、周期和阶段复验、失效时禁用输入；绑定/接收切换沿用input→targets锁序并取消旧任务。
- `src/receiver-routing.ts`/`App.vue`/`style.css`：显示绑定窗口与实际接收窗口，自动/手动/自身/待选/失效状态，歧义选择及未就绪禁用单次。
- `build/verification/TASK-003-ROUTING/coder/automation/`：只公开UIA控件的主程序计算器验收助手。默认只编译；`-ExecuteOnce` 仅由主协调者授权后使用，不提供计算器旁路消息/全局输入。

## 检查

- 仓库根执行 `./scripts/npm.ps1 run typecheck`：PASS。
- `./scripts/npm.ps1 run test:targets`：PASS，含接收路由UI辅助检查。
- 设置 `CLICKMASTER_TEST_ROOT=build/verification/TASK-003-ROUTING/coder/tests-final` 后执行 `./scripts/npm.ps1 run test:backend`：40 Rust tests PASS。
- `./scripts/npm.ps1 run build:desktop`：Windows x64 release PASS，PE图标核验PASS。
- 最终入口：`build/verification/TASK-003-ROUTING/coder/final/ClickMaster.exe`，9,251,840 bytes，SHA-256 `1A445CC5827E45DFBD85E2C917AB31309A020A41B260B1C5B5CAD01ADD1A75FC`。
- 稳定源码清单：`build/verification/TASK-003-ROUTING/coder/source-manifest.sha256`，SHA-256 `4CF3C1A51A1F778DC0D88CA08B69C4CE1F52553825B37618C1C97C7E815456C7`。
- 自动化助手默认 `run.ps1` 编译通过，记录 `DesktopQueries=false`、`InputPosted=false`；未启动程序、调用UIA或投递输入。

## 边界

无版本/锁/图标/托盘/overlay修改；旧TASK-003 final/data与历史产物保留。真实计算器、鼠标、游戏、焦点与隔离仍待主协调者在Verifier通过后按契约单次验收；未提交、推送或发布。所有新产物位于项目内并使用脱敏回执。
