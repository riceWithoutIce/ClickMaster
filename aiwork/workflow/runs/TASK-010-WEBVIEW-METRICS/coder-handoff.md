# TASK-010 Coder交接

状态：Coder交接完成，生产源码及rc.6候选已冻结，待独立只读Verifier。日期：2026-10-22。

## 修改与边界

- 新增`perf_webview.rs`：只在`with_webview`的UI线程闭包内使用当前WebView的Environment8.GetProcessInfos；跨线程仅传自有原生句柄和闭合类型数据。最多64条，跨采集共享一个pending许可；每次采集有独立取消/回执槽及epoch核对，2.5秒过期不用于资源归属，迟到闭包不污染新会话。收起后由后端采集循环刷新，不靠前端轮询。
- 保留创建身份及原始句柄，按真实采样间隔计算CPU差值，首次/过期恢复/访问失败不补零；去重、成员变化、退出末段只计一次，内存按已观察进程求和并列缺失。匿名输出`webview-environment`、`sharedPossible=true`、crashpad排除、类型/数量/新鲜度/覆盖。环境可能跨实例或应用共享，工作集可重叠，不称EXE独占CPU或物理内存；短命进程及峰值仍可能漏采。
- 新增`perf_states.rs`，在`performance.rs`接入后端busy与窗口事实、转变代次、丢失标记及六个固定汇总组：待命/执行×显示/收起，另列mixed/unknown。未观察组not-observed，资源空值不补零；含有效采样时长、发送/速率、CPU分项/分母/覆盖、内存样本。采集尾段不纳入严格分组。
- `lib.rs`仅必要启动、窗口事件和自有显隐操作测量钩子；前端只更新诊断说明。IPC、授权、哈希、单帧、计数、读取诊断、停止/退出保护均未修改。无优化、持久worker/pipe、游戏操作或覆盖层修复。
- 新日志schema为2，安全轮换仍兼容旧schema1完成记录；默认关闭、60秒、1秒采样、5秒落盘及1MiB/5套5MiB保留。旧版本、历史证据、用户原日志保持原位。
- 自有版本同步至0.1.0-rc.6。只显式加入已锁定的webview2-com=0.39.1、windows-core=0.62.2；所有非根锁包块不变，无升级、下载或Runtime安装。

## 自测入口与结果

均从仓库根运行，任务内`check.mjs`使用`scripts/environment.mjs`，Cargo offline/locked，所有测试根和缓存/临时输出留在项目内。持久记录路径均相对仓库，不保存真实PID、创建时间、窗口标题或授权内容。

| 方法 | 结果 | 证据（相对仓库） |
| --- | --- | --- |
| `CLICKMASTER_CHECK_RUN=closure`后运行`node build/verification/TASK-010-WEBVIEW-METRICS/coder/check.mjs fmt typecheck backend frontend`（环境变量按当前shell设置） | fmt/typecheck/helper通过；后端96通过、0失败、2个隔离入口忽略 | `build/verification/TASK-010-WEBVIEW-METRICS/coder/*-closure.log` |
| 原生原句柄/创建身份/退出、模拟官方成员集合、去重/过期/接口失败/访问拒绝/单pending/会话隔离、四状态helper | PASS；模拟环境不是实际COM证据 | 同上backend日志及源码测试 |
| 实际collector、隔离非游戏私有管道、每侧24帧/24worker，off/on比较 | 每侧24帧、0错误；CPU为量化的单次顺序观测，不能证明零开销或真实游戏降幅 | `build/verification/TASK-010-WEBVIEW-METRICS/coder/tests-closure/performance-integration/comparison.json` |
| 实际collector生命周期/停止/短预算自动结束/有界退出/四状态事实夹具 | PASS；不操作真实桌面窗口 | 同目录`lifecycle/`、`states/`与backend日志 |
| `node build/verification/TASK-010-WEBVIEW-METRICS/coder/validate-logs.mjs` | PASS；6样本、24发送及24原句柄CPU复算、六组时长/发送复算、单次23736字节、禁身份字段及已知路径前缀0匹配 | `build/verification/TASK-010-WEBVIEW-METRICS/coder/log-validation.json` |
| Windows过滤offline metadata及非根锁块比较 | PASS | `build/verification/TASK-010-WEBVIEW-METRICS/coder/lock-refresh.json` |
| `node build/verification/TASK-010-WEBVIEW-METRICS/coder/check.mjs desktop` | 退出0；前端生产构建、locked release构建及真实PE图标匹配通过 | 同目录`desktop-closure.log` |
| `node build/verification/TASK-010-WEBVIEW-METRICS/coder/verify-manifests.mjs` | 普通主EXE清单与rc.5精确相同；仅测试EXE具任务内嵌入的v6清单 | 同目录`manifest-check.json`及三份提取清单 |
| `node build/verification/TASK-010-WEBVIEW-METRICS/coder/package.mjs` | 退出0；x64、PE版本/图标、两次无授权worker入口返回64且无数据目录、ZIP CRC/条目/提取后逐字节、来源/保留/已知前缀检查通过 | 同目录`snapshot.json`、`pe-*`、`worker-entry.json`、`zip-validation.json`、`*-preservation.json`、`preservation.json`、`sanitization.json` |

非游戏开关比较：off墙钟6.5481217秒，主0.140625/worker0.484375 CPU秒；on墙钟6.6615078秒，主0.156250/worker0.453125 CPU秒。两侧总CPU分别0.625000/0.609375秒；单样本噪声、CPU量化和阶段覆盖均限制因果归因。此夹具未连接WebView，采样明确unavailable；原生句柄/模拟成员集合测试单独验证采样逻辑，不冒充实际COM关联。

## 测试加载器问题及修正

首次后端编译成功但测试EXE在进入正文前返回0xc0000139。导入核对定位为默认comctl32无TaskDialogIndirect，新增Tauri dispatcher引用使库测试也需要Common Controls v6。仅测试链接参数尝试不可用于库单测，且全目标嵌入会与普通bin资源重复；这些尝试均撤回，生产`build.rs`与rc.5哈希相同，Cargo目标配置未改。

任务内`prepare-tests.mjs`现先`cargo test --release --locked --no-run --message-format=json`，从compiler-artifact定位生成的库测试EXE，再用已有Windows SDK的mt嵌入仅该测试EXE的v6清单，然后执行通常的Cargo测试。无新工具/Runtime安装，无普通UI启动。Verifier若在独立输出目录重建测试，须同样准备生成的测试清单，并将脚本输出根改为Verifier目录，不能覆盖Coder证据。普通主EXE仍用Tauri原清单，最终候选需与rc.5清单精确比对。

首次失败记录：`backend-initial-locked.log`、`backend-first-compile.log`、`backend-test-loader-before.log`、`backend-test-manifest-duplicate-before.log`、`backend-test-manifest-target-before.log`、`import-function-check.json`。初次未过滤平台的offline metadata因缺少非目标Android缓存失败，没有联网，见`lock-refresh-all-platforms-not-cached.json`；Windows过滤后成功。失败记录保留并脱敏，不据其声称测试正文运行过。

## 尚未运行与旧项

- 实际WebView COM/UI、共享环境真实归属/变化、用户真实四场状态、真实游戏性能/体验、跨系统和旧/缺失Runtime：NOT RUN，归用户人工采集。
- 未打开普通候选主窗口，未操作游戏、F8、全局输入或焦点；只运行无授权worker拒绝入口时须单列包验证证据。
- 既有V008-01未修改或替用户处置；本次相关测试通过不等于关闭旧finding。OVERLAY-001仍延期。
- 未提交、推送、上传、正式发布、清理历史或轮换用户提供日志。

## 最终冻结快照

- ID：`TASK-010-WEBVIEW-METRICS-rc.6-coder-final`，自有版本`0.1.0-rc.6`；未提交dirty快照，HEAD仅作参照，不能替代下列73文件清单。
- 来源清单：`build/verification/TASK-010-WEBVIEW-METRICS/coder/source-manifest.json`，SHA-256 `247850E775997D6C4E702F84D0FC7660CEAB95C98B710344EC1C2D04BBDFB1CD`。
- EXE：`build/verification/TASK-010-WEBVIEW-METRICS/coder/final/ClickMaster.exe`，9743872字节，SHA-256 `A4A5123111CA05B6D3E7BBAE0943855DAD48C7065E16379D343C805CBDE5B221`。
- ZIP：`build/staging/TASK-010-WEBVIEW-METRICS/ClickMaster-0.1.0-rc.6-windows-x64.zip`，3060049字节，SHA-256 `C1C36FD22F344C919FFCF7EDD4B61A2BD4A5B67FF566D741E9A4C092F450E4B2`。配套`source.json`、`SHA256SUMS.txt`。本地候选未签名，非正式发布。
- 变更相对rc.5：15个既有文件、2个新模块、0移除，详见`source-changes.json`；所有IPC/热键/覆盖层模块及生产build.rs未变。
- 原652历史证据、15旧候选、2份用户原日志逐项SHA保持，见`preservation.json`、`candidate-preservation.json`、`user-log-preservation.json`。保留全部无关dirty状态，不清理历史。
- 源码及候选已冻结；Verifier仅写自己的隔离检查输出，不能改生产源码、锁文件、候选、Coder证据或用户日志。任何生产修正先经用户处置并建立新快照。

脱敏：source/EXE已知本机路径前缀扫描0匹配；JSONL/TXT禁真实身份字段扫描通过，编译失败链路的本机链接命令已省略。此为规定范围检查，不宣称穷尽所有隐私或系统级外写监控。所有新生成输出均在本项目任务目录与统一缓存/构建目录，未做全系统监控；OS/已安装共享Runtime内部记录不作零外写保证。
