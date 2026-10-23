# TASK-002 Coder 交接

日期：2026-10-21。Coder：本轮唯一源码写入者。状态：**READY_FOR_REVIEW**；源码、配置、锁文件及资源已冻结，真实Windows查询及UI仍待用户人工验收。

## 稳定快照和产物

- 快照 `TASK-002-CODER-S1`；Git基线仍为 `d3d8f58`，实际比较基准是TASK-001-UI R1稳定清单及随后原有文档。既有未提交状态全部保留，未提交、推送或发布。
- [完整40文件清单](source-manifest.sha256)：源码、配置、两个锁文件、资源及相关说明；清单SHA-256 `30003b238c24a6cb985a1cb4cd83e4fba1df36baf0faa8c47e10521066cecadc`。
- 最终人工入口 `build/verification/TASK-002/coder/final/ClickMaster.exe`，9,061,376 bytes，SHA-256 `5BD948D95B9272EE1CCA3F13F234F8B1CBE7612D29758AADDCC7728A1D57A751`。
- 与本次 `build/cargo/release/clickmaster.exe`一致；仅复制、**未启动**。final无data，未复制、覆盖或操作旧final及用户data。
- 交接后停止修改清单内被验文件，Verifier只读审核并在自身项目内隔离副本运行非UI检查。父级任务/roadmap/devlog记录不属于生产源码清单。

## 实现

### 真实候选与可信绑定

- `windows_query.rs`使用EnumWindows，只读取目标信息。规则为可见、有标题的顶层主窗口，包含最小化窗口；排除当前PID、桌面/shell、tool/noactivate/cloaked窗口、无appwindow样式的owned辅助窗口。同名EXE、同PID多个独立主窗口按HWND分别保留，不去重合并。
- 用有限查询权限取得进程创建FILETIME及EXE文件名，完整路径仅在内存解析；不能取得必要身份的项排除并返回数量摘要，进程退出/权限不足/刷新失败不冒充成功，不提权或修改其他进程。
- `targets.rs`持有候选token及可信身份：HWND、PID、进程创建FILETIME、窗口线程ID。客户端只传后端token；客户端标题/EXE/PID不作为可信绑定输入。选中候选不绑定，显式绑定时重新查询并比较身份。
- 刷新使旧候选token过期，但保留可重新验证的绑定；解绑/失效清除摘要，不根据EXE或旧PID自动跟随重启实例。
- Rust后台线程约每秒检查一次，独立于页面、WebView计时器、最小化及托盘。身份或可验证性变化解除并提供原因，标题变化只更新展示；退出事件通知线程停止。本轮没有输入任务或发送副作用。

### 关闭二级选择

- CloseRequested先prevent_close，原子pending状态抑制重复请求；发事件显示单一原生HTML dialog。初始化信息同时带pending，可恢复前端尚未监听时到达的关闭请求。
- dialog取消初始焦点、Esc取消、原生模态焦点约束，取消/收起失败不退出；操作中禁用重复提交，错误留在对话框。导航和会话草稿不重新挂载。
- `window_policy.rs`处理自身窗口：保留任务栏偏好时普通最小化；默认隐藏前必须有托盘；任务栏设置/隐藏失败尝试显示、还原并恢复任务栏，返回提示。正常最小化复用该保护；托盘退出仍直接退出。
- 保留偏好和版本真实IPC、既有导航/配色/尺寸/图标/便携数据，输入执行/按键/热键等未实现功能仍禁用；设置中的×说明同步为选择收起/退出。

## 修改清单

| 范围 | 文件 |
| --- | --- |
| 新增目标与策略模块 | `src-tauri/src/targets.rs`、`windows_query.rs`、`window_policy.rs` |
| 后端接线、生命周期、测试根 | `src-tauri/src/lib.rs`、`settings.rs` |
| 现有依赖feature | `src-tauri/Cargo.toml`仅为windows-sys增加Threading/Dwm功能；无新包或版本升级 |
| 前端与样式 | `src/App.vue`、`src/style.css` |
| 非UI测试落点入口 | `scripts/project.mjs` |
| 直接相关使用说明 | 根`README.md`、`src/README.md`、`src-tauri/src/README.md`、`scripts/README.md` |

两个锁文件哈希保持不变；图标资源、build.rs、PE校验脚本和版本未变。旧数据/配置测试只是替换测试根解析，沿用原断言，不修改生产配置格式。

## 命令和检查

所有命令从仓库根运行，经原项目内环境入口设置临时目录和缓存，无新安装、服务器或桌面启动：

```powershell
./scripts/npm.ps1 run typecheck
./scripts/npm.ps1 run build:desktop
./scripts/npm.ps1 run test:backend
```

实际过程：类型检查与前端构建通过；首轮Rust编译指出后台检查循环末尾Mutex临时值生命周期问题，增加表达式分号缩短临时值生命周期后重新构建成功。最终Windows x64 release构建2m24s，前端17模块，JS84.82KB/CSS7.32KB，构建末端PE资源校验通过；不打安装器。随后完整release非UI测试17 passed/0 failed（0.02s测试执行，编译链接另1m49s）。

非UI测试只使用人工虚构窗口与注入查询，不调用真实EnumWindows/inspect或创建窗口；旧6项文件测试使用本轮 `build/verification/TASK-002/coder/tests/`，未重写TASK001夹具。统一入口允许显式设置项目内 `CLICKMASTER_TEST_ROOT`，拒绝项目外路径，Verifier须使用自己的隔离输出。

17项涵盖：

- 同名/同进程多主窗口不合并、选中不绑定、显式绑定正确对象。
- 伪造token、刷新前旧token拒绝；绑定前PID、创建时间、线程身份变化拒绝。
- 刷新保留有效绑定；关闭/退出/实例替换后清除，重启不自动绑定。
- 绑定权限失败、失效检查权限失败、刷新失败及空列表结果不报绑定成功。
- 主窗口筛选纯规则；最小化通过不使用IsIconic过滤的源码路径体现，实际行为待人工。
- 无托盘不隐藏，保留任务栏走普通最小化，隐藏失败恢复可找回路径。
- 原6项EXE相邻路径/目录失败/探针清理、配置往返/非法配置保留/Windows文件锁失败与恢复测试。

## AC结果

| 编号 | 结果 | 证据与边界 |
| --- | --- | --- |
| AC-01 | STATIC/LOGIC PASS；HUMAN PENDING | 枚举适配层和规则、保留多窗口测试；真实Windows列表/最小化/辅助过滤未执行 |
| AC-02 | LOGIC PASS；HUMAN PENDING | 后端候选身份、token验证及身份变化拒绝测试；真实绑定待人工 |
| AC-03 | LOGIC/STATIC PASS；HUMAN PENDING | 刷新/解绑/失效/重启测试；Rust每秒线程不依赖前端；真实退出/托盘期间检测待人工 |
| AC-04 | LOGIC/STATIC PASS；HUMAN PENDING | 加载/空/失败/不可读计数UI路径及注入权限测试；不自动提权，实际权限差异未执行 |
| AC-05 | LOGIC/STATIC PASS；HUMAN PENDING | prevent_close、pending原子闸、原生dialog/Esc/取消、策略失败恢复与托盘直接退出路径；真实窗口交互待人工 |
| AC-06 | STATIC PASS；HUMAN PENDING | 页面根级草稿ref保持，绑定状态与关闭框不清草稿；偏好/版本真实IPC、图标/布局未回退、执行禁用 |
| AC-07 | PASS | 类型/前端/Windows桌面生产构建、PE图标核验和17项非UI测试全部通过 |
| AC-08 | PASS（受管理范围） | 完整manifest与EXE哈希、源码脱敏扫描及项目内落点；无系统级零外写证明 |

真实Windows查询可用性、权限表现、窗口/进程重启、关闭框/焦点/托盘和UI状态均为人工门槛，逻辑测试不替代。父级已准备人工清单，本交接不改该文件；实际人工结果须绑定本轮final哈希。

## 资源与脱敏

- PE校验实际group32512/lang1033引用RT_ICON1，新图1104 bytes，SHA-256 `ac160094426f536177ad7101ccbff0ede9fc5b1ee2a9684dcd88611f70756eb9`，匹配当前ICO；脱敏证据 `build/verification/TASK-002/coder/pe-icon.json`。
- 本轮源码/配置/说明常见本机路径与凭据模式扫描无命中，git diff --check通过；最终EXE对当前仓库、斜杠形式仓库、用户目录ASCII/UTF-16精确扫描无命中，非完整二进制元数据审计。
- 没有保存真实窗口标题、EXE完整路径、环境输出、截图或原始完整日志。没有执行实际窗口查询，所有测试标题/进程为明确人工夹具。
- 缓存/临时/编译仍在项目内固定位置，测试及最终入口仅本轮coder目录；前端生成删除的原 `.gitkeep` 已恢复，无历史清理。
- 未启动ClickMaster、目标、浏览器、开发服务器或可控窗口，未进行桌面自动化、系统图标缓存修改、外部进程操作。

## 已知边界与后续

本轮身份校验满足HWND/PID/进程实例比较。Windows本查询方案没有独立窗口创建序号，因此同一进程同一线程在检查间隙极快销毁并复用完全相同HWND，不能单凭这些身份字段区分；当前没有发送输入，后续输入任务发送前仍需再验证，不作零竞态承诺。可见/主窗口及可读权限规则保守，特殊应用不出现时由人工记录并再决定范围，不静默扩大到系统辅助窗口或提权。

没有验证后台消息、游戏响应、停止调度、热键、正式发布、Windows11或缺失WebView2；绑定完成不代表输入兼容性通过。Verifier只审核本轮核心绑定/关闭和紧邻回归，在冻结快照上独立执行相称非UI检查，findings一次性返回用户决定，不自动向Coder追加修复。
