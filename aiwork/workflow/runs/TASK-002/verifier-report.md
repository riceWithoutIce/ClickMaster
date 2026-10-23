# TASK-002 独立 Verifier 报告

日期：2026-10-21。对象：`TASK-002-CODER-S1`。Verifier：独立 Verifier agent。

## 结论与快照

**本轮源码审查及已执行的独立非 UI 检查通过，未发现 BLOCKER 或 FOLLOW-UP findings。真实窗口查询、绑定与关闭框/托盘交互仍为 HUMAN PENDING，不宣称本任务无条件验收通过。**

- [40 文件清单](source-manifest.sha256) SHA-256：`30003B238C24A6CB985A1CB4CD83E4FBA1DF36BAF0FAA8C47E10521066CECADC`。
- 开始与结束核对：原始 40 文件和结束时隔离副本 40 文件均 0 mismatch，未修改生产源码、锁文件或 Coder 记录。
- 隔离根：`build/verification/TASK-002/verifier/source/`，按清单复制源码并复制现有前端依赖，未安装依赖或复制 Rust 编译缓存。
- 最终入口：`build/verification/TASK-002/coder/final/ClickMaster.exe`，SHA-256 `5BD948D95B9272EE1CCA3F13F234F8B1CBE7612D29758AADDCC7728A1D57A751`，与交接一致。核验时 final 无 `data/`。
- 未启动本工具、目标、浏览器、开发服务器或桌面自动化；未执行真实 EnumWindows/进程查询，未操作用户窗口或数据，未停止外部进程。

## 实际检查与证据归属

在隔离根使用副本的受管理入口执行：

```powershell
./scripts/npm.ps1 run typecheck
./scripts/npm.ps1 run build
```

均退出 0。Vite 7.3.1 构建 17 模块，约 693 ms，JS 84.82 KB、CSS 7.32 KB；未启动预览服务器。

独立以 `rustc --test` 编译副本中无外部依赖的 `window_policy.rs`，输出至 `build/verification/TASK-002/verifier/window-policy-tests.exe`；进程 TEMP/TMP/TMPDIR、Node 缓存及二进制输出均在 verifier 目录内，编译使用项目路径重映射。3 项原有纯策略测试均通过：无托盘不隐藏、保留任务栏采用普通最小化、隐藏失败恢复任务栏/显示/还原路径。测试只使用 Fake，不查询或操作真实窗口。

最终 EXE 离线 PE 检查通过：group 32512 / language 1033 → icon 1，载荷 1,104 bytes、SHA-256 `AC160094426F536177AD7101CCBFF0EDE9FC5B1EE2A9684DCD88611F70756EB9`，与当前 ICO 一致；仅读取字节，没有加载或执行 EXE。

[Coder 交接](coder-handoff.md) 的完整 Rust release 桌面构建及 17 passed / 0 failed 属 Coder 运行证据。本次逐项读取 17 个测试的断言、夹具与注入边界，确认目标状态测试使用虚构窗口、窗口筛选测试只调用纯规则，旧 6 项文件测试改为本轮受管理测试根。Verifier 没有重跑全部 17 项或完整桌面构建，不将其写作独立执行结果；已独立运行其中 3 项窗口策略测试。

## 逐项验收

| 条目 | 非 UI 结果与依据 | 人工边界 |
| --- | --- | --- |
| AC-01 候选列表 | PASS（静态）：EnumWindows 只读适配层；可见/有标题主窗口筛选，排除自身、desktop/shell、tool/noactivate/cloaked及非 appwindow 的 owned 辅助窗口；不使用 IsIconic 排除最小化；按 HWND 保留，不按名称/PID合并；完整进程路径仅内存取文件名 | NOT RUN：实际可用窗口、特殊应用与最小化候选表现 |
| AC-02 显式可信绑定 | PASS（静态及 Coder 测试证据）：选中仅改候选 token；后端查表持有 HWND/PID/创建 FILETIME/线程；绑定重新 inspect 比较完整身份，旧刷新 token 和伪造 token 拒绝；客户端展示字段不作身份输入 | NOT RUN：真实选择、绑定、进程重启与权限变化 |
| AC-03 刷新/失效/解绑 | PASS（静态及 Coder 测试证据）：刷新先重验现有绑定、使旧候选过期；解绑/失效清空并递增 revision；不按 EXE/PID自动绑定重启实例；独立 Rust 一秒线程校验，不依赖前端页或 WebView 计时器；前端事件和轮询按 revision 接受状态 | NOT RUN：托盘期间退出检测与实际摘要更新时序 |
| AC-04 错误与权限 | PASS（静态及 Coder 测试证据）：必要身份不可读项排除并计数；刷新错误、无窗口、加载与权限摘要各有路径；不自动提权；查询适配层无目标修改/激活或输入 API | NOT RUN：真实 Windows 权限差异和特殊失败 |
| AC-05 关闭选择 | PASS（静态及独立 3 项策略测试）：CloseRequested prevent_close；原子 pending 防重复，初始化信息补偿前端监听前关闭请求；单一 HTML dialog，取消初始焦点、Esc 取消、操作中禁用提交；收起成功才清 pending，失败保留错误和恢复路径；托盘退出直接 app.exit | NOT RUN：真实 ×、焦点、取消/Esc、收起/恢复和退出 |
| AC-06 紧邻 UI 回归 | PASS（静态）：草稿仍为根 setup refs，导航不重挂根；关闭框位于页面分支外；真实版本与偏好 IPC 保留；全局错误、绑定摘要存在；输入执行/热键/按键配置仍禁用；图标资源与既有尺寸保留 | NOT RUN：真实布局、导航、偏好、图标及托盘回归 |
| AC-07 检查/构建 | PASS：独立类型、前端构建、3 项既有策略测试、PE 检查；Coder 完整生产构建与 17 项测试记录可追溯 | 本次未独立重跑 Rust 完整构建或其余 14 项测试；真实程序未启动 |
| AC-08 清单/脱敏/落点 | PASS（受管理范围）：40 文件始末不变，最终 EXE 哈希匹配；文本敏感模式扫描零命中；隔离副本/输出/临时缓存均在任务 verifier 目录 | 未做系统级监控或完整二进制敏感信息审计，不宣称系统零外写 |

## 核心审查补充

- 绑定检查比较进程创建时间，避免仅靠复用 PID/HWND 跟随新进程；线程身份额外核对。NativeQuery 打开有限查询权限句柄，确认进程仍活动、读取创建时间/文件名后，再核对窗口线程与 PID。
- 状态由后端 Mutex 序列化，前端仅传 token；刷新保留有效绑定、失效原因和不同 revision，不把列表选择当成绑定成功。标题变化可更新展示，身份变化解除。
- 退出检查线程独立于页面/最小化；程序 RunEvent::Exit 通知停止。当前无输入调度，因此不把约一秒身份检查表述为未来输入的实时停止保证。
- 关闭弹框采用原生 dialog 模态与基本焦点处理；取消不退出，失败提示保留对话框；默认无托盘时拒绝隐藏，保留任务栏偏好走普通最小化。查询只操作读句柄，窗口收起策略仅接自身窗口，不按目标 PID 操作其他应用。
- 同进程同线程极短间隙完全复用同一 HWND 的不可区分边界已在 Coder 交接披露，不扩为本轮理论加固 blocker；后续输入任务发送前校验仍需另行约定。

## Findings 与停止

本次冻结范围内未发现 findings，不向 Coder 追加修复。独立非 UI 审核到此结束。用户需对最终哈希对应产物完成真实列表/绑定/失效、权限表现、关闭选择及托盘交互；源代码、注入查询测试或 PE 资源检查不能替代这些人工门槛。

本文仅保存脱敏摘要和仓库相对路径，未记录真实窗口标题、完整进程路径、原始环境、截图或完整日志。Windows 11、后台输入/游戏响应、热键、正式发布及 Runtime 缺失实测不在本轮通过证据内。
