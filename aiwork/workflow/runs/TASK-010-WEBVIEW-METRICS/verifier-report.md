# TASK-010 独立限定验收报告

日期：2026-10-22。Verifier：task007_verifier（受协调者准入承担 TASK-010）。结论：约定源码、原生/helper/非游戏 fixture、日志计算与候选检查通过，**无新增 BLOCKER 或 FOLLOW-UP**。真实 COM/UI、共享环境行为及真实游戏四场资源仍 NOT RUN，不能用模拟 adapter 或后端状态事实代替。历史 V008-01 未修未处置，保持原 finding。

## 稳定来源与独立方法

对象 `TASK-010-WEBVIEW-METRICS-rc.6-coder-final`，73 文件 dirty 快照；相对 rc.5 修改 15 项、增加 2 模块，无移除。manifest SHA256 `247850E775997D6C4E702F84D0FC7660CEAB95C98B710344EC1C2D04BBDFB1CD`，EXE SHA256 `A4A5123111CA05B6D3E7BBAE0943855DAD48C7065E16379D343C805CBDE5B221`，ZIP SHA256 `C1C36FD22F344C919FFCF7EDD4B61A2BD4A5B67FF566D741E9A4C092F450E4B2`。Git HEAD 不替代此未提交快照。

独立输出根 `build/verification/TASK-010-WEBVIEW-METRICS/verifier/`。仓库根运行本目录 `check.mjs`，继承 `scripts/environment.mjs`；Cargo release/offline/locked，独立新测试根为 `tests/`，TEMP/TMP/TMPDIR 与 Node cache 留本目录，编译产物仍走项目统一 `build/cargo/`。不覆盖 Coder 证据。

检查先 `cargo test --release --locked --offline --no-run --message-format=json`，编译退出 0；从生成的 compiler-artifact 定位库测试 EXE，仅用已有 SDK mt 嵌入独立输出根内的 Common Controls v6 清单，退出 0。再运行通常后端测试；未先执行会失败的无清单入口，也未改生产 build.rs/目标配置。提取最终 rc.6 候选、rc.5 候选与测试清单，普通候选清单逐字节一致，仅库测试程序具单独 v6 清单。实际记录见 test-compile.log、test-manifest-preparation.json、manifest-check.json 和三份提取 XML。

其余独立命令均为本目录 `package-check.mjs`、`dependency-check.mjs`、`validate-logs.mjs`、`sanitization-check.mjs`；全部退出 0。未运行 fmt/version-sync/重新打包、普通 EXE、真实 COM/游戏/固定游戏管道/浏览器/F8/托盘或焦点窗口操作，无提交推送、正式发布、外传或历史清理。无系统级外写监控，不能声称整个系统零外写。

## 六条限定验收

| 条目 | 独立结果、证据类别与边界 |
| --- | --- |
| 官方归属、身份和缺失 | perf_webview.rs 仅由当前 PlatformWebview.environment cast 到 Environment8.GetProcessInfos；本地锁定 Tauri/COM binding 接口一致。PID 仅用于立即打开只读资源查询句柄，之后按创建时刻加原句柄保留，不按名称/父 PID/命令行补猜。最多 64 项、去重、访问缺失、类型 unknown、成员变化、首次基线、过期恢复均显式标记。模拟集合/原生句柄测试通过；原 Child 退出末段只累计一次。没有实际 COM 调用证据，枚举与打开之间极短的 PID 复用窗口不声称被穷尽消除。 |
| 单 pending、会话隔离与托盘 | with_webview 只把 COM 留 UI 线程闭包内，跨线程为闭合字段与自有原生句柄。全局 Permit 至多一个排队/执行闭包，取消不释放旧 pending 去追加第二个；独立 shared 回执槽与 alive、窗口状态 epoch 检查阻止迟到污染。2.5 秒前后 freshness 检查；后台 collector 每次样本主动 request，无需前端轮询。停止/退出不等待 UI，默认关闭无新增 COM 查询/采样线程。Permit/epoch/helper 与源码验证通过，未将模拟许可测试写作真实队列/COM 实测。 |
| CPU/内存与范围 | 固定 scope=webview-environment、sharedPossible=true、excludesCrashpad=true，沿官方同数据目录环境可能共享语义，不能解释为 EXE 独占。CPU 用原句柄差值/实际采样秒数/逻辑处理器数；首次、查询缺失和过期不补零，退出末段只计一次，partial 与缺失分别记录。工作集/专用内存为已观察进程求和，工作集可能共享/重叠、短命进程/峰值可漏；日志/摘要/说明均明确。原生与模拟测试通过，真实列表归属 NOT RUN。 |
| 四状态与分母 | 后端 Status.busy/任务代次及本工具实际显隐事件驱动；State revision/lost 检查包含往返切换和争用丢失，不把末端状态充当整个区间。六组为四状态加 mixed/unknown，未观察组 not-observed/null；采集尾段明确不归严格组。生产 helper 可控状态与实际 collector 后端事实 fixture 通过；独立复算每组时长、发送量/速率、主/worker CPU、内存均值/最大值及覆盖分母一致。无真实桌面/游戏状态切换。 |
| collector/IPC 紧邻回归 | 实际 Collector/原生非游戏管道/send_frame 开关各 24 帧、24 worker、0 错误；独立 CPU 与原句柄总量一致、无缺失或边界截断。日志 schema2、预算轮换、生命周期/手动/短预算自动结束/有界退出、原停止/单帧/计数/身份保护测试通过。19 个 IPC/input/hotkey/overlay 等模块 hash 与 rc.5 相同；模拟 WebView 测试单列，不冒充开关比较包含真实 COM 开销。 |
| 包、依赖、脱敏和保留 | ZIP 五条目 CRC/逐字节一致，无用户 data/日志/cache/debug；EXE x64、图标匹配，实际字符串版 rc.6、数字 0.1.0.0、NotSigned。反向仅改根包版本及移除两个直接依赖，即精确重建 rc.5 Cargo.lock hash，证明所有非根块未变；前端包/锁文件也仅版本变化，生产 build.rs 与 rc.5 相同。73 源码及候选前后冻结一致，652 历史证据、15 旧候选、2 用户原日志未变；独立已知前缀与日志禁身份字段检查通过。 |

后端结果为 **96 passed / 0 failed / 2 ignored**，驱动测试实际调用两个隔离子进程入口，ignored 不是跳过其驱动场景。类型检查、前端 helper、版本、图标均退出 0。详情见 backend/typecheck/frontend/version/icon.log 与 results.json。独立检查没有本轮失败或自动重试；Coder 首次 loader/metadata 等失败仍保留于原目录，后来的通过不会改写其历史。

## 独立日志与等量比较

两侧沿同一非游戏接收策略：24 帧/24 worker，父/子哈希测试 EXE，每帧后额外等 100ms，末尾等 3000ms，开启侧实际经过 5 秒批写。仅随机 OfflineFixture 管道，无游戏身份/程序集/私有授权链或 Tauri UI/WebView。此 fixture 无 Source，WebView 正确为 unavailable/null，不能当作真实 COM 或开销测量。

| 模式 | 帧/worker/错误 | 主 CPU 秒 | worker CPU 秒 | 合计 CPU 秒 | 实际耗时秒 |
| --- | --- | --- | --- | --- | --- |
| 关闭 | 24 / 24 / 0 | 0.218750 | 0.578125 | 0.796875 | 6.6760947 |
| 开启 | 24 / 24 / 0 | 0.156250 | 0.546875 | 0.703125 | 6.6062637 |

开启会话 6 个样本、24 个匿名 worker，JSONL+TXT 合计 **23733 字节**；六个样本全部归 unknown，原因是 fixture 没有实际窗口事实，四组因此保留 not-observed/null。所有样本已观察发送之和 24，worker CPU 与原 Child 句柄累计相等，CPU 百分比/实际区间/发送率/分组统计独立复算通过。

另一个 states fixture 使用同一真实 collector 与人工构造的后端状态事实，不调用 native UI/COM。8 个样本、合计 **23674 字节**：四个严格组各 1 区间，mixed 4 区间，unknown 未观察；区间时长、资源分项、发送量与汇总一致，尾段未混入分组。helper 单独覆盖状态往返、丢失、unknown 及未观察空值。详细数值见 log-validation.json 和 tests/performance-integration/。

比较为单次顺序小样本，CPU 约 15.6ms 量子、外部负载未隔离；差值不证明改善、负开销或零开销，尤其不量化新 UI-thread COM 查询的成本。要定位真实产品资源仍需用户 rc.6 实际采集，不能从此数据直接进入优化归因。

## 证据边界、历史项与停止

2.5 秒新鲜度/接口失败/访问拒绝/身份替换主要是生产 helper + 模拟成员集合证据；原句柄退出查询为原生受控进程证据；COM 来源、异步派发/关闭路径为源码与锁定绑定证据。实际 Environment8、共享数据目录、多实例/访问权限/Runtime 差异、真实托盘后台刷新、真实四场与完整游戏性能均 NOT RUN。60 秒仍为边界 helper 加短预算 collector 收尾，不冒称实跑 60 秒；慢盘沿用未响应 busy 模拟，不是磁盘故障注入。

V008-01 保持原 FOLLOW-UP/待用户裁定，hotkey.rs 未修改；本次该测试通过不构成修复，TASK-008 首次 81/1/1 结果保留。OVERLAY-001 仍延期。本轮无新增 finding，不向 Coder 追加工作。

本轮限定审查已完成并停止。rc.6 可作为本地候选交用户按 human-test.md 实际采集；不表示真实环境验收或正式发布通过。
