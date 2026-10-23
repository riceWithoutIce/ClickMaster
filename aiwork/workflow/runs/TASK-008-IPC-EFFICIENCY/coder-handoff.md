# TASK-008 Coder交接

日期：2026-10-22。Coder：task008_coder。范围依据task.md、implementation-plan.md及baseline-report.md，用户授权独立非游戏测量，真实游戏后续人工验收。

## 稳定快照与产物

- 快照标识：TASK-008-IPC-EFFICIENCY-rc.4-coder-final。生产源码、配置与自有版本元数据在最终自测后冻结；没有commit/push或正式晋级。
- 权威来源与EXE/ZIP哈希：`build/verification/TASK-008-IPC-EFFICIENCY/coder/snapshot.json`、`source-manifest.json`、`source-changes.json`；Git HEAD仅作参考，不把已有dirty工程等同提交。
- 独立EXE：`build/verification/TASK-008-IPC-EFFICIENCY/coder/final/ClickMaster.exe`。
- 便携候选：`build/staging/TASK-008-IPC-EFFICIENCY/ClickMaster-0.1.0-rc.4-windows-x64.zip`，配套`source.json`和`SHA256SUMS.txt`。
- 本任务不覆盖rc.1～rc.3或历史证据；历史652项与9个候选EXE/ZIP/manifest的核对结果见`preservation.json`、`candidate-preservation.json`。用户data与已有dirty状态保留，包括任务前已删除的build/frontend/.gitkeep。

## 改动与边界

- `ipc_native.rs`：快速核验只用持有的原进程与直接HWND/PID/TID/可见/非最小化/客户区；完整发送核验保留全进程唯一实例与唯一Unity窗口。专用IPC不查询焦点。系统失败保存固定阶段/原因及可用数值错误码。
- `ipc_worker.rs`：每帧仍为独立worker，原父子EXE哈希、游戏程序集哈希、一次授权、私有stdin来源/父实例、用户DACL及first-instance全部保留。连接后实际客户端检查为快速检查；唯一一次完整核验在写前guard，随后再次核对实际客户端，再写唯一4字节帧。无重试/补发/重叠。
- `ipc_runtime.rs`、`ipc.rs`：10ms监督与50ms等待仅做快速目标检查；闭合枚举阶段/原因构成脱敏报告，保留首次失败。最终/收尾报告不抹掉已知错误，完整写入后失败仍计数。读取观察仍200ms且只作诊断；取消/限时、原目标保护与清理锁定保留。
- `channels.rs`、`lib.rs`：普通兼容状态按绑定epoch复用；显式选通道及开始强制重验。缓存只供UI展示，不能授权worker写入；worker每帧仍校验程序集。
- 前端类型/诊断标签、帮助说明及rc.4版本元数据同步；100～60000ms、默认2000/2000不变。消息通道、依赖、布局、预览和配置持久化不变。

## 已执行自测

所有命令从仓库根通过`coder/check.mjs`执行，使用项目环境包装：TEMP/缓存/target/test root在当前项目内、Cargo offline/locked；日志替换项目绝对前缀。早期compile1/focused2日志单独保留，不覆盖。

| 证据 | 结果与范围 |
| --- | --- |
| fmt-final.log | cargo fmt通过 |
| typecheck-final.log | Vue/TS类型检查通过，版本rc.4一致 |
| backend-final.log | 82通过、0失败、1子进程fixture按设计忽略；release后端单元与独立管道fixture |
| frontend-final.log | 筛选、路由、通道、间隔、时钟与IPC诊断映射通过；未知标签不回显原文本 |
| desktop-final.log / final-icon.log | release桌面无安装器构建及实际PE图标检查；不启动普通UI |
| pe-imports.json / pe-version-signature.json | x64、完整rc.4版本字符串/数值资源、实际签名状态 |
| worker-entry.json | 无授权worker入口退出64、无输出且不创建data；不启动主UI |
| zip-validation.json / sanitization.json | ZIP五个预期条目、CRC及解压字节一致；源码及EXE已知本机前缀检查 |

新增测试执行生产helper/状态报告路径：快速窗口身份/最小化策略、完整guard顺序与一次枚举/短路、绑定epoch缓存与强制重验、写前拒绝、原生写入失败阶段/错误码穿过worker退出、首次错误不被后报覆盖、完整写入后的状态错误保留失败并计数。已有单帧精确协议、DACL first-instance、私有父绑定一次授权、未读/慢读/终态竞态、停止/限时/目标失效后计数及仅清理自有Child等回归继续通过。

## 证据限制与后续入口

- 非游戏fixture不操作真实游戏/普通工作窗口、不触发真实F8。策略通过与原生独立管道通过不代表真实游戏前台交互、最小化/退出、停止和长期资源验收。上述人工项目均NOT RUN。
- 现有“读取失败”fixture使用注入Err(109)验证诊断语义，不能表述为实际FlushFileBuffers原生失败实测。写失败分支则使用真实断开管道，校验实际数值错误码。
- task008_baseline独立维护受控A/B。Coder不宣称完整Tauri/WebView2/真实游戏CPU降幅；其完整报告及相同发送量比较由协调者归档。逐帧worker/哈希剩余开销明确保留，不扩大为任务级worker。
- 脱敏扫描只核对已知本机前缀，不能推导全系统零外写、未知敏感信息穷尽扫描或OS/Runtime网络行为已验证。报告无原PID/HWND/路径/token；枚举只允许固定词汇。
- Verifier先核对source-manifest/snapshot，按冻结需求限定只读审核；不启动游戏/UI/F8，不修改被验源码。独立审核及用户人测尚未完成，不能据Coder自测宣布发布通过。

生产写入已停止；允许后续新增独立验证输出和协调记录，不允许未对齐修改被验源码。
