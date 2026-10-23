# TASK-003-OVERLAY Coder 交接

日期：2026-10-21。Coder为本轮唯一源码写入者。状态：**READY_FOR_REVIEW**，源码已冻结。范围为用户批准的原生Win32/GDI坐标预览有限POC，基于TASK-002 R2；后台输入及单次测试仍未开发。

## 稳定快照与产物

- 快照`TASK-003-OVERLAY-CODER-S1`；[46文件源码清单](source-manifest.sha256)继承R2的42文件并加入4个生产模块，SHA-256 `402dee158d74d9de34ceba8c93806ad449e58b3caf28796290806259bf6ccaf3`。
- 最终入口`build/verification/TASK-003-OVERLAY/coder/final/ClickMaster.exe`；9,163,264 bytes；SHA-256 `27064d1b62d26f208531ee33575836421a34138f98c1879d4491d21b01a7f8ff`。
- final与本次`build/cargo/release/clickmaster.exe`一致；独占复制创建，没有启动或生成data。原TASK-002、R1、R2 final和两个依赖锁文件哈希核对未变。
- 结构化摘要及PE图标证据位于`build/verification/TASK-003-OVERLAY/coder/evidence.json`、`pe-icon.json`；本轮临时收口脚本`finalize.mjs`包含防覆盖、旧final/锁文件校验及精确二进制路径扫描。均保留项目内。
- 交接后停止修改清单内文件，父级核对后准入限定Verifier；父级契约及人工记录不在生产快照清单内。

## 实现

- 前端只在现有鼠标位置区域增加“显示坐标预览（实验）”及状态/错误说明，无应用内画布或新WebView。沿用中心/X/Y草稿，默认关闭、会话内有效，不改配置格式。
- `overlay.rs`：纯几何、状态机和fake renderer测试。中心用当前客户区尺寸取整；自定义必须有限、非负、整数且在`[0,width)`/`[0,height)`内。非法输入仍可显示边框及无效说明，但不画误导性点击点。无有效客户区则暂停。
- `overlay_native.rs`：原生窗口/GDI和目标几何适配。WS_POPUP窗口，WS_EX_LAYERED/TRANSPARENT/NOACTIVATE/TOOLWINDOW；色键透明，WM_NCHITTEST返回HTTRANSPARENT，WM_MOUSEACTIVATE返回MA_NOACTIVATE。只在绑定目标前台时以客户区大小SetWindowPos，使用NOACTIVATE，无全桌面常驻窗口、无任务栏按钮、不激活目标。
- `overlay_runtime.rs`：独立命令/确认线程，默认不创建覆盖窗；启用且目标合格时才惰性创建。约80ms间隔采样当前绑定身份、目标可见/最小化/cloaked、必要前台句柄及客户区；切其他软件/隐藏/最小化暂停，返回目标恢复。显示前后复查前台资格，不能宣称零延迟，实际显示/隐藏可能有一个采样周期的延迟。
- PMv2只设置覆盖层线程，退出恢复旧上下文。GetClientRect的两个角经ClientToScreen映射为物理屏幕坐标，保留负屏幕原点，不二次乘DPI；自定义坐标不按尺寸比例缩放。文字/十字装饰大小按覆盖窗自身DPI缩放。实际跨进程DPI语义及混合屏位置仍以人工验收为准。
- `targets.rs`：新增后端可信身份只读快照和独立绑定epoch，标题/状态/刷新不改变epoch；重绑（包括同一候选）、解绑/身份失效改变epoch。预览IPC携带期望epoch，后端只接受当前可信绑定；过期草稿不会自动启用到新绑定。
- 解绑/重绑先通过命令确认关闭原生窗；预览线程也独立验证绑定和epoch，失效关闭，不自动重开。主窗口退出发送shutdown并join；原生窗销毁和GDI释放都在所属线程。自有笔/字体从DC恢复后释放，stock brush不删除；注册类、窗口及数据的生命周期一并收口。创建/更新/绘制/隐藏失败关闭并报告，用户可重试。
- 前端通过单调revision拒绝旧预览状态；草稿变动短暂合并，绑定代次变化/关闭或错误取消待发更新。切键盘模式关闭预览，切页保留草稿。服务错误单独展示，不混入目标列表错误。

## 修改与保留

新增`src-tauri/src/overlay.rs`、`overlay_native.rs`、`overlay_runtime.rs`及`src/overlay-position.ts`；修改`lib.rs`接线、`targets.rs`快照/epoch、`App.vue`与少量样式、现有前端辅助测试及直接README。

`Cargo.toml`只为已有windows-sys补GDI/HiDpi/LibraryLoader feature；没有新依赖、版本或配置格式变化。两个锁文件、原窗口查询/辅助排除、关闭/托盘策略、settings及图标资源保持。覆盖窗归自身PID，因此普通及扩展目标范围均排除。

保留全部dirty状态、旧final/data、测试夹具和验收/诊断记录；不提交、推送、发布、安装工具或清理历史。父级契约/人工验收/roadmap/devlog未由Coder修改。

## 命令与非UI验证

从仓库根使用原受管理环境入口，TEMP/TMP/TMPDIR、依赖缓存、编译输出及夹具均在项目内。Cargo格式化也使用`projectEnvironment()`，不修改用户级配置。

```powershell
./scripts/npm.ps1 run typecheck
./scripts/npm.ps1 run test:targets
$env:CLICKMASTER_TEST_ROOT = Join-Path (Get-Location) 'build/verification/TASK-003-OVERLAY/coder/tests-final'
./scripts/npm.ps1 run test:backend
./scripts/npm.ps1 run build
./scripts/npm.ps1 run build:desktop
node build/verification/TASK-003-OVERLAY/coder/finalize.mjs
git diff --check
```

类型检查和生产筛选/坐标转换离线测试PASS。最终Rust release测试27 passed/0 failed，执行0.01s、编译链接1m28s；第一次26项通过后补独立绑定epoch回归及异步绘制失败关闭检查，再使用新`tests-final/`夹具完整通过。初次`coder/tests/`保留，不覆盖历史测试根。

27项包含原22项身份/范围/关闭策略及配置回归，加5项覆盖：中心/自定义边界、空/负/小数/非有限坐标、无效尺寸/溢出；负屏幕原点及DPI不重复缩放；移动/尺寸变化下中心变化、自定义不比例缩放；前后台/隐藏/最小化/cloaked/几何不可读暂停恢复；解绑/epoch或实例变化关闭、渲染show/hide失败关闭及显式退出；标题/候选刷新不改变绑定epoch、同目标重绑改变epoch。

测试只调用纯函数、fake renderer和注入窗口查询，不创建窗口、读真实目标或执行GUI。前端辅助测试验证空值/null、0、非法数字、负/小数保留及有限JSON；没有实际控件交互证据。生产前端PASS，19模块，JS88.47KB/CSS7.60KB。

Windows x64 release桌面构建PASS，2m02s；没有安装器。自动PE图标检查PASS：group32512/lang1033引用RT_ICON1，1104 bytes，SHA-256 `ac160094426f536177ad7101ccbff0ede9fc5b1ee2a9684dcd88611f70756eb9`，与当前ICO匹配。

本轮变更源码/说明及交接的常见本机路径/凭据模式扫描无命中，`git diff --check`通过。最终EXE对实际仓库路径两种斜杠形式及用户目录UTF-8/UTF-16精确扫描无命中；不宣称完整二进制元数据审计或整个系统零外写证明。构建恢复前端槽位`.gitkeep`，未清理历史。

| 门槛 | 状态 |
| --- | --- |
| 中心/自定义/非法值/尺寸/负原点/DPI纯几何 | LOGIC PASS |
| 前后台暂停恢复、绑定代次/身份变化关闭、渲染失败关闭 | FAKE ADAPTER/LOGIC PASS |
| 类型、前端转换测试、27项Rust测试、生产前端/桌面及PE图标 | PASS |
| 原生窗/线程/GDI生命周期与无焦点/穿透设置 | STATIC PASS；实际HUMAN PENDING |
| 真实客户区对齐、DPI/多屏、视觉、焦点/穿透、移动/缩放、切换/退出 | HUMAN PENDING |

## 人工边界

本轮没有启动应用EXE、覆盖层、GUI夹具、浏览器、开发服务器或桌面自动化；没有真实窗口枚举/查询/目标操作，也没有重跑R1诊断。没有记录真实标题、PID/HWND、完整进程路径、截图、原始环境或未脱敏日志。

原生焦点/穿透、视觉、客户区实际对齐、移动/缩放/不同DPI屏幕、前后台切换及退出资源行为均为**HUMAN PENDING**。源码设置/纯策略通过不能代替实机证据；独占全屏没有兼容保证，技术POC未因编译通过而成为正式选型结论。后台输入/热键/长按/点击取点/拖动/全局监听均未实现，执行控件禁用。

稳定快照由父级核对后准入独立Verifier，只审本POC及紧邻回归；findings一次性回用户决定。Verifier使用本任务自身新测试根，按契约执行相称独立检查，不重复无必要完整Rust release构建。用户从本轮final手动验收，通过后才讨论正式集成或扩展；失败则记录具体表现再决定修正/换型，不自动切透明WebView或其他路线。
