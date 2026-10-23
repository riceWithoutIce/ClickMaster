# TASK-011 Coder 稳定交接

日期：2026-10-23。生产源码已冻结，rc.7便携候选已生成，交给协调者准入独立只读Verifier。不是正式发布或真实游戏验收。

## 已实现

- rc.7 专用 IPC 重复运行复用一次授权的 worker 与原命名管道连接；单次保留旧路径。命令逐帧放行，每帧保持原实例/唯一窗口/客户端写前核验；断开不重连、不重试、不回退。
- 固定与随机间隔输入1～60000ms，默认2000ms；UI/后端相同边界，不承诺1kHz。重复读取诊断不阻断，短帧不逐阶段刷UI，连接和写入等待留在监督线程。
- 停止/时长在命令准入前再检查；连接等待持续核验目标；已写缓冲帧不能撤回。文档说明目标静态约90ms读前等待、极短间隔背压和游戏停止后尾效应。

## 实际证据与失败保留

- `coder/check.log`、`fmt.log`、`helpers.log`、`typecheck.log`、`frontend.log`：退出0。
- 首轮有效后端：99 passed / 0 failed / 2 ignored，`test-before-final-polling.log`；当时尚未包含最终sequence observer回归，不能替代最终源码通过。
- 测试EXE两次出现0xc0000139，源于嵌入test-only Common Controls manifest后又触发重编译覆盖；`test-first-stale-manifest.log`和协调者工具输出保留。最终测试改为no-run编译、嵌manifest、直接运行产物。
- 最终源码首次直接测试：`test-final.log`，SHA-256 `65EDF59D8CA2205700679F0BB3E12073738172D94AFE6DE845FAF2545036BD25`，退出101，97 passed / 3 failed / 2 ignored。四项持久IPC测试均通过；失败为`performance::tests::budget_lock_and_unknown_records_preserve_data`（旧未知记录预算不足）、`isolated_logger_lifecycle_and_equal_work_pipe_comparison`（lifecycle旧输出）、`portable_unicode_space_move_upgrade_keeps_config_without_cwd_dependency`（旧目录非空）。未删除旧测试产物或修改断言。
- 协调者批准一次全新隔离目录重跑：`final-tests/`，`test.log`退出0，100 passed / 0 failed / 2 ignored。最后no-run编译、嵌test-only manifest、直接运行产物；未再编辑生产源码。旧97/3/2保留，不能称失败未发生；完整参数和哈希见`test-attempts.json`。
- 隔离接收器100ms/1ms各24帧，一worker/一连接，最终平均帧间隔分别107.426ms与4.410ms（约9.309Hz与226.751Hz）；各自worker CPU31.25ms与46.875ms，墙钟2.546s与0.155s。数据在`coder/final-tests/ipc-fixtures/persistent-summary/`和`persistent-fixture-summary.json`；包括启动/监督开销，只是短时非游戏fixture，不是游戏/完整应用性能或1kHz承诺。
- 无读取客户端背压：1024完整帧后约2.005秒native write超时，0重连；有限管道缓冲不是游戏已消费证明。
- 最终`desktop.log`退出0，Windows x64 PE、版本、内嵌图标、未授权worker拒绝、ZIP CRC和解压哈希均通过。Cargo锁文件仅根包版本从rc.6变rc.7，无依赖升级。

## 未运行

真实TapTapLoot、多系统、鼠标loading、普通UI、托盘/F8/焦点、长时间体验。WebView采集生产源码与V008-01/OVERLAY-001未修改。未提交、推送、正式发布、清理历史或操作真实游戏。

## 冻结快照和包

- `build/verification/TASK-011-IPC-REUSE/coder/snapshot.json`：`TASK-011-IPC-REUSE-rc.7-coder-final`，73文件manifest SHA-256 `A3F36AD05CAB3D10FF1BC5DD5744AC8C8B6AC50503A146F7217050470B99F035`；HEAD仅参考，完整dirty源码manifest才是验收快照。
- EXE：`build/verification/TASK-011-IPC-REUSE/coder/final/ClickMaster.exe`，9772032字节，SHA-256 `4A6761F90C0894BABF8B4080B94F77393588CC49337F4FEA89325D4A128A784E`。
- ZIP：`build/staging/TASK-011-IPC-REUSE/ClickMaster-0.1.0-rc.7-windows-x64.zip`，3068834字节，SHA-256 `66D35133DAE371F7C68203D5A1D8F9417552E465B9F1DE2D2705EC0FDC70D934`；NotSigned。
- 18旧候选、4用户日志、652历史文件逐项哈希未变；来源/包已知本机前缀扫描通过。此检查不是穷举隐私或全系统外写监控。
- 所有新输出在本任务verification/staging，统一environment.mjs重定向项目临时/缓存。未清理原日志或历史产物。

## 复核入口

源码变更、manifest、测试与保留证据均在`build/verification/TASK-011-IPC-REUSE/coder/`；`source-changes.json`列17个修改文件，核心为ipc.rs/ipc_worker.rs/ipc_runtime.rs/repeat.rs和UI间隔边界，其余为测试helper/版本/说明。连接等待在独立监督线程；真实cursor loading未自动测，仍需人工。独立复核只核冻结任务与紧邻回归，V008-01继续延期，OVERLAY-001未处理。
