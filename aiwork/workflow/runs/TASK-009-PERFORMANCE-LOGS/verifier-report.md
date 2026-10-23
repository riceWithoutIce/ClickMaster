# TASK-009 独立限定验收报告

日期：2026-10-22。Verifier：task007_verifier（受协调者准入承担 TASK-009）。结论：本轮约定的源码、非游戏测试、日志计算与候选包检查通过，**无新增 BLOCKER 或 FOLLOW-UP**；真实 UI、实际 60 秒采集与真实游戏性能仍待用户验证。历史 V008-01 未修复、仍待用户裁定，本轮通过不覆盖原失败。

## 稳定来源与方法

对象为 `TASK-009-PERFORMANCE-LOGS-rc.5-coder-final`，71 文件 dirty 源码快照；相对 rc.4 为 18 项修改、2 项新增、无删除。manifest SHA256 `2423B0DA7062173A6D92E22B63A2F09340886C22545F044FE3958A444F81A551`，EXE SHA256 `5CDC17F605E27A096E9667DEF08C8CA4C13630A065F069AE0CC4587ADD8AEF93`，ZIP SHA256 `6B8B5397DC401028EC51E043F7CDCA90DF8BAE85BB67DD33E455145C43B11104`。Git HEAD 仅供参考，不替代此快照。

独立证据根为 `build/verification/TASK-009-PERFORMANCE-LOGS/verifier/`。仓库根执行 `node build/verification/TASK-009-PERFORMANCE-LOGS/verifier/check.mjs`：继承 `scripts/environment.mjs`，Cargo 使用 release/offline/locked，缓存和 TEMP/TMP/TMPDIR 留项目内，独立新测试根为本目录 `tests/`。并行运行后端、vue-tsc、前端 helper、版本和 PE 图标检查。另执行本目录 `package-check.mjs` 与 `validate-logs.mjs`，只读核对候选和独立 fixture 输出。

未改生产源码、锁文件、版本或候选；未执行 fmt/sync/重新打包、普通 UI、真实游戏、固定游戏管道、真实 F8/托盘/全局输入或焦点操作；无提交推送、正式发布和历史清理。工具链内部系统记录没有系统级监控，不声称全系统零外写。

## 验收结果

| 条目 | 独立结果与边界 | 证据 |
| --- | --- | --- |
| 默认关闭 | ACTIVE 默认 false；测量点先检查开关，无采样线程/日志或额外进程枚举；off fixture 未创建 logs。诊断状态命令独立，未持久化自动开启 | performance.rs、lib.rs、App.vue 只读；backend.log |
| 生命周期与停止隔离 | 重复开始拒绝、结束幂等、手动结束/短预算自动结束/退出通过；输入取消与自有 worker shutdown 先于诊断最多 250ms 等待；采集线程不启动或停止连点 | backend.log；lib.rs 退出顺序；lifecycle fixture |
| 有界开销 | 输入测量使用 try_lock 更新固定阶段汇总/有界 worker 列表，争用可丢弃；文件写入仅采集线程，1 秒采样、5 秒批写；实际 fixture 已走批写路径 | performance.rs 只读；backend.log、log-validation.json |
| 主程序 CPU/内存 | GetProcessTimes 差值按实际单调区间和逻辑处理器归一，查询失败为 null；工作集/专用内存只取本进程，样本 CPU/百分比与发送率公式独立复算通过 | perf_native.rs；CPU helper；log-validation.json |
| worker 归属与边界 | 复制直接持有的原 Child 句柄，不按名称/PID 扫描；短命退出后可读 CPU，失败后成功基线不重复累加；首/末截断测试通过，结束采集不终止仍存活 worker；独立 24 个匿名记录 CPU 总量等于原句柄累计 | backend.log；独立 on/logs；log-validation.json |
| 固定 schema 与脱敏 | schema 1、固定阶段/状态/失败枚举；不记录输入内容、目标身份、路径、标题、真实 PID/HWND 或令牌。注入的外来状态文本不复制；独立日志已知本机前缀/身份字段及绝对盘符检查为零 | closed_context 测试；log-validation.json |
| 阶段与 WebView 限制 | 10 个固定阶段、嵌套墙钟明确不可相加当 CPU；worker 报告按会话代次过滤，不完整另计；WebView2 未证明归属，明确 unavailable，非 0 占用 | ipc_worker/runtime、performance.rs 只读；独立日志 |
| 文件预算与保护 | JSONL+TXT 每会话 1MiB，最多 5 套/5MiB；未知/不完整记录保留，预算不足拒绝；目录 share=0 互斥，完成对先取得双方独占句柄才轮换；失败无外部回退，不改输入状态 | budget/rotation/logger 测试；Files::prepare 只读 |
| IPC 紧邻回归 | 仅有测量、可选统计与代次字段；私有授权、每帧哈希/实际客户端/写前核验、一次 write、读取仅诊断、停止和自有 Child 清理语义保留；报告增至固定 4096 字节并检查指标 | worker/runtime/native/lib.rs 只读；既有非游戏后端测试 |
| 独立检查 | 后端 **91 passed / 0 failed / 2 ignored**；两个 ignored 是只能由隔离驱动唤起的子进程 fixture，驱动测试通过。类型/helper/版本/图标全部退出 0 | backend/typecheck/frontend/version/icon.log、results.json |
| 候选包 | ZIP 五条目 CRC/逐字节一致，无 data/debug/log/cache；EXE 为 x64，实际图标与源 ICO 一致；版本字符串 rc.5、数字 0.1.0.0、NotSigned；已知本机前缀二进制扫描通过 | package-validation.json、pe-version-signature.json、pe-imports.json、binary-sanitization.json |
| 冻结与历史保留 | 审核前后 71 源码、manifest、EXE、ZIP 全部匹配；652 项旧证据及 rc.1～rc.4 共 12 项 EXE/ZIP/manifest 未变 | freeze-before.json、freeze-after.json |

日志权限失败/锁冲突证据分别为受阻目录与同目录独占测试；对已完成文件对的占用保护为源码证据，不声称测试穷举 Windows 所有锁/权限组合。慢盘仅用未响应 busy 模拟验证退出等待有界，不是实机慢盘故障注入。

## 独立等量开关比较

本轮使用新测试根实际重跑生产 Collector/native pipe/send_frame。开关均为 24 帧、24 个 worker、相同固定帧与接收策略，父/子哈希测试 EXE，每帧后额外等待 100ms，最后额外等待 3000ms。接收器为独立非游戏 fixture，仅使用随机 OfflineFixture 管道；没有游戏固定管道、真实游戏身份/程序集/私有授权链、窗口、Tauri UI 或 WebView。

| 模式 | 帧/worker/错误 | 发送端 CPU 秒 | worker CPU 秒 | 合计 CPU 秒 | 实际耗时秒 |
| --- | --- | --- | --- | --- | --- |
| 关闭采集 | 24 / 24 / 0 | 0.125000 | 0.328125 | 0.453125 | 6.4161442 |
| 开启采集 | 24 / 24 / 0 | 0.203125 | 0.203125 | 0.406250 | 6.4343443 |

开启日志为 6 个样本、24 个匿名 worker 记录、已观察发送 24，JSONL+TXT 合计 **11019 字节**；CPU 总量与所有原 Child 句柄之和相等，无缺失或首末截断。主程序与 worker 归一公式、发送率、阶段词汇、WebView unavailable 和字节预算独立复算通过。证据为 `tests/performance-integration/comparison.json`、`tests/performance-integration/on/logs/` 与 `log-validation.json`。

这是单次顺序小样本，外部负载未隔离，CPU 约 15.6ms 量子；发送端增加而 worker 降低不能直接解释为采集热点。合计差 0.046875 秒不能证明性能改善、负开销或零开销，也不能推广为完整产品资源结果。尚需用户真实场景日志定位成本，不建议由这组数据直接开始优化。

## 历史 finding 与未执行项

V008-01 原热键测试完成同步窗口保持 **FOLLOW-UP / 待用户裁定**；hotkey.rs 本轮未修改。TASK-008 的 81 通过、1 失败、1 忽略和首次失败证据仍有效；本轮该测试通过不构成修复或改写原结论。详见原 TASK-008 verifier-report.md。

本轮没有新增 finding。真实 UI 的开始/停止/收起/恢复、实际跑满 60 秒、真实 F8/托盘、真实游戏采集与长期性能、跨 Windows/Runtime 均 NOT RUN。60 秒边界只有生产 end_reason 的可控时间 helper；同一 capture 循环用 20ms 测试预算验证自动收尾，不冒称实跑 60 秒。

约定限定审核已完成，返回协调者/用户后停止当前审查。rc.5 可作为本地候选交用户实际体验；这不是人工体验通过或正式发布结论。
