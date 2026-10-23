# TASK-008独立限定验收报告

日期：2026-10-22。Verifier：task007_verifier（受协调者准入承担TASK-008）。状态：本轮IPC功能/保护与性能证据检查通过；**全量后端独立复跑未全通过**，81通过、1失败、1子进程fixture忽略。1项FOLLOW-UP待用户裁定，无BLOCKER。不得将本报告写作全部测试PASS或正式发布通过。

## 快照与检查范围

对象 `TASK-008-IPC-EFFICIENCY-rc.4-coder-final`，69文件manifest SHA256 `59BD412C7CF93A919E675720750AB661032FB682CC62B59818145BB941C3FEBD`；EXE SHA256 `0F11C14A7BF0C4DC1AF6D0DC62A5BECF105775F9651D037268EF0AAED6C7F638`，ZIP SHA256 `366B7D35CEE1A7D9B18B2BC8349882D4C5A0BA02D89EDC0A199A154975D7F7BB`。仅检查相对rc.3的20项修改和紧邻回归，未扩展全面产品审计。

独立证据位于 `build/verification/TASK-008-IPC-EFFICIENCY/verifier/`。仓库根运行本目录check.mjs，通过scripts/environment.mjs设置项目内缓存/临时和独立新测试根，Cargo offline/locked。未运行fmt/sync/打包入口，未更改生产源码；没有普通UI、真实游戏、固定游戏管道、真实F8/托盘/桌面输入。性能检查只读原始结果与适配源码，未重跑90秒测量。

## 验收结果

| 条目 | 独立方法与结果 | 证据 |
| --- | --- | --- |
| IPC前后台策略 | native/runtime/worker源码检查，无焦点API或主动激活；直接窗口policy原进程/窗口身份、可见非最小化及客户区拒绝逻辑保留；策略测试通过 | backend.log、ipc_native.rs只读 |
| fast/full分层 | 监督10ms/等待50ms只调用validate_fast；worker写前validate_for_send执行fast/唯一进程/唯一Unity窗口，随后真实client再次核对，唯一一次WriteFile；完整guard顺序/短路测试通过 | ipc_native/runtime/worker.rs、backend.log |
| 缓存不能授权写 | channel普通查询force=false按epoch复用，select/start均force=true；worker每帧Lease::open核对程序集，缓存没有写入入口 | channels.rs、lib.rs、backend.log |
| 授权与哈希 | 每帧父EXE哈希与子授权EXE哈希、程序集hash、一次claimed、私有stdin源PID/父创建时刻/同EXE、用户DACL/first-instance/实际client均仍存在，无path/mtime缓存 | ipc_storage/worker/native.rs只读；既有非游戏授权与单帧测试通过 |
| 首次脱敏failure | 闭合枚举stage/reason及可选系统码，不含原路径/身份/令牌；Receipt::fail和Observation保留首错；前端未知标签固定映射，读取失败仅诊断；写后异常仍计一帧 | backend.log、frontend.log、single-input.ts |
| 取消/计数/隔离 | 非游戏受控管道：写前停止计0、写后停止/到期/目标失效计1、最终报告竞态、读取不阻断、仅终止直接持有Child、槽位有界均通过 | backend.log |
| 后端全量 | **81 passed / 1 failed / 1 ignored**，失败为下方V008-01；本轮新增IPC测试均通过，Coder原自测82通过与本次独立结果分别保留 | backend.log、results.json |
| 类型/前端/版本/图标 | 独立退出0；版本rc.4，图标匹配源ICO，PE字符串版本rc.4、NotSigned | typecheck/frontend/version/icon.log；Coder PE资源证据 |
| ZIP与冻结 | 独立CRC/5条目逐字节一致，无data/debug/cache；69源码/manifest/EXE/ZIP审核前后匹配，旧652项及rc.1～rc.3九份EXE/ZIP/manifest未变 | package-check.json、freeze-before/after.json |

## 性能证据复核

baseline-source六份原始文件全部匹配rc.3源清单；两份冻结manifest匹配报告hash。optimized-final native逐字节匹配本轮生产native，harness native仅替换fixture EXE/窗口类/程序集路径/预期hash。optimized-final-source的ipc_runtime.rs与最终生产不同，但harness没有编译该文件，代表性监督由harness main实现；不将整个源码副本称为完整产品等价。

核对双方runner均每帧父/子hash相同rc.3 EXE，同4MiB synthetic assembly、90ms接收延迟、数量1、相同额外等待，短命worker通过保留的原进程句柄在退出后累计GetProcessTimes。2000ms同14帧；100ms固定时长87→115帧，报告同时提供每帧CPU和候选固定87帧比较。原始CPU/逻辑处理器归一、总CPU及comparison.json计算一致；等87帧5.296875对23.375 CPU秒，约77.3%下降，支持受控fixture单位工作改善。

限制披露充分：单次顺序样本/全机负载未隔离、baseline尾等待、计时分辨率、内存快照漏峰值、synthetic assembly非实际游戏大小、harness未包含Tauri/WebView/私有worker授权/完整JSON/200ms未确认路径/UI缓存收益。不得转述为真实ClickMaster或游戏CPU降幅；不承诺实际发送频率。源码归属与计算证据见performance-check.json、source-attribution.json。独立核查辅助脚本首次因默认文本编码读取失败，显式UTF-8后完成，非产品测试失败。

## Findings

### V008-01 / FOLLOW-UP：既有热键测试缺少完成同步

- 对应验收：acceptance第4项全后端检查；位置 `src-tauri/src/hotkey.rs` 中normal_terminal_releases_registration_without_shutdown_request及with_port线程终态。
- 实际：独立全量测试81通过，唯一失败断言为 `!runtime.live()`；前一条 `released` 断言已通过。源码顺序先port.release设置Fake released，再alive.store(false)。测试只等待released就立即读live，因而能在二者之间观察到live=true。
- 影响：本次独立全后端检查不通过，不能写作82/82 PASS。该文件与rc.3哈希一致，本轮未修改；现证据指向测试完成同步窗口，未证明真实热键释放或停止功能回归，因此不是正常用户路径BLOCKER。
- 建议：后续由Coder对测试采用明确线程完成同步/有界等待，再限定复跑原finding及紧邻回归；不要弱化生产停止条件。可由用户选择当轮处理或延期/接受本次证据缺口。
- 处置：**待用户裁定**。Verifier保留首次失败，未改源、未自动重试，也未向Coder追加工作。辅助证据finding-hotkey.json；日志线程标识已脱敏。

## 未执行及停止边界

真实游戏前台人工交互、玩法、长期CPU、完整桌面隔离、真实F8/托盘/最小化与退出、跨Windows/Runtime均NOT RUN，留用户验收。未签名及原WebView/覆盖层限制保留。源码/fixture通过与真实运行证据不互相替代。

本轮限定检查已完成，findings一次性返回协调者/用户，不继续全面审计。用户需裁定V008-01，原全测试失败不会因延期或接受风险改写成PASS。无提交推送、正式发布、历史清理或外部游戏操作。
