# TASK-003-OVERLAY 独立限定复审

日期：2026-10-21。对象：`TASK-003-OVERLAY-CODER-S1`。范围：原生坐标预览 POC、绑定身份/代次、退出及紧邻既有功能。

## 结论与快照

**本次源码审查及独立非 UI 检查通过，未发现 BLOCKER 或 FOLLOW-UP findings。** 原生焦点/穿透、客户区对齐、不同 DPI/多屏和真实窗口生命周期仍为 HUMAN PENDING；纯测试和原生设置不代替这些门槛，POC 尚未成为正式方案验收结论。

- [46 文件清单](source-manifest.sha256) SHA-256：`402DEE158D74D9DE34CEBA8C93806AD449E58B3CAF28796290806259BF6CCAF3`。
- 原始 46 文件开始/结束均 0 mismatch；结束时隔离副本亦 46 文件 0 mismatch。未修改生产源码、锁文件或 Coder 记录。
- 独立副本：`build/verification/TASK-003-OVERLAY/verifier/source/`，复制清单文件及现有前端依赖；未安装新依赖或复制 Rust 缓存。
- 最终人工入口：`build/verification/TASK-003-OVERLAY/coder/final/ClickMaster.exe`；SHA-256 `27064D1B62D26F208531EE33575836421A34138F98C1879D4491D21B01A7F8FF`，与交接一致，核验时同级无 `data/`。

## 独立实跑检查

在副本使用其受管理入口执行，均退出 0，临时/缓存/输出保留副本内：

```powershell
./scripts/npm.ps1 run typecheck
./scripts/npm.ps1 run test:targets
./scripts/npm.ps1 run build
```

- 前端生产筛选及坐标转换辅助测试通过，覆盖空值/null、0、非有限值、负数/小数保留和有限 JSON；不等于实际控件交互。
- 前端构建 Vite 7.3.1、19 模块、约 618 ms，JS 88.47 KB / CSS 7.60 KB；未启动浏览器或服务器。
- 为避免重复完整 Rust release，建立 `verifier/overlay-pure-tests.rs`：复制冻结 `overlay.rs` 的几何、Engine、Renderer trait 和全部原测试，仅移除 serde import/derive/属性，补与生产字段一致的 Identity 数据定义。使用 `rustc --test` 独立运行；生产逻辑和原测试断言未修改，未链接原生适配层，不创建/查询窗口。
- 4 项独立纯模块测试全部通过：中心/自定义/边界/非法值；负屏幕原点与 DPI 不重复缩放；前后台/隐藏/最小化/cloaked/无效几何暂停恢复；绑定代次/身份变化及渲染 show/hide 失败关闭与显式停止。输出源及测试二进制、TEMP/TMP/TMPDIR 均在本任务 verifier 目录，编译使用路径重映射。

[Coder 交接](coder-handoff.md) 中 27 项完整 Rust 测试、Windows x64 release 构建及 PE 图标检查属于 Coder 实跑证据。本次核对测试源码及接线，但没有独立重跑完整 27 项、桌面构建或 PE 校验，不将其写成 Verifier 独立运行结果。纯模块 harness 也不验证 serde IPC 序列化或真实 Win32/GDI。

## 源码与紧邻回归

| 检查范围 | 结论和证据 | 真实桌面边界 |
| --- | --- | --- |
| 前后台与资格 | STATIC / PURE TEST PASS：仅同一可信绑定、目标前台/可见/非最小化/非cloaked、有有效客户区时显示；显示前后再次核对前台资格，其他状态隐藏但保持启用待恢复 | NOT RUN；80 ms采样及调用间隙不构成零延迟承诺 |
| 身份/epoch与换绑 | STATIC / PURE TEST PASS：独立绑定 epoch，重绑/解绑/失效变化，展示更新/刷新不变；启用 IPC 对 expected epoch 和后端可信身份验证；解绑/重绑先等待 stop 确认，运行线程也再次比较 identity/epoch，变化后关闭而非跟随 | NOT RUN：实际换绑/退出与显示时序 |
| 客户区与坐标 | STATIC / PURE TEST PASS：GetClientRect 双角 ClientToScreen，保留负屏幕原点、checked边界，无二次DPI乘法；中心随当前宽高、自定义不比例缩放；非法值不画点，无有效几何暂停 | NOT RUN：跨进程DPI语义、真实客户区及混合屏对齐 |
| 不抢焦点/穿透 | STATIC PASS：WS_POPUP，LAYERED/TRANSPARENT/NOACTIVATE/TOOLWINDOW；WM_NCHITTEST为HTTRANSPARENT、WM_MOUSEACTIVATE为MA_NOACTIVATE，SetWindowPos带NOACTIVATE；不调用目标激活/移动接口 | NOT RUN：真实鼠标穿透、焦点、任务栏与独占全屏行为 |
| 原生资源/线程 | STATIC PASS：一个专属线程设置PMv2并恢复原上下文；惰性创建覆盖窗、同线程消息泵/销毁/注销类；窗口绑定的PaintData贯穿生命周期；笔/字体恢复原对象后释放，stock brush不删除；shutdown消息及join收口 | NOT RUN：实机资源占用/退出观察；不以静态检查宣称无所有泄漏 |
| 错误恢复 | STATIC / PURE TEST PASS：创建/SetWindowPos/绘制/隐藏失败关闭并返回独立状态；异步paint failed在pump后检查；DPI上下文失败不创建窗口；用户可重试，不改变配置或目标状态 | NOT RUN：真实原生失败及主窗口错误呈现 |
| 前端请求 | STATIC / HELPER TEST PASS：预览默认关、会话内；revision拒绝旧状态、120ms合并草稿更新、epoch变化取消待发；切键盘关闭；绑定忙时禁用操作；异常单独显示 | NOT RUN：实际控件、切页、错误可感知性 |
| 绑定/托盘紧邻回归 | STATIC PASS：原目标枚举/范围排除、窗口关闭/托盘策略、便携目录、settings和图标未改；新增覆盖窗属自身PID排除；执行控件仍禁用，无输入发送或监听 | NOT RUN：真实托盘/关闭/绑定完整回归，不重做全面审计 |

## Findings、证据限制与停止

本次冻结范围内未发现 findings，不向 Coder 追加修复；约定非 UI 审查到此结束。

用户应对最终哈希对应入口手动验证：目标前台时客户区边框和点对齐、目标可点击且不抢焦点；移动/缩放/已有不同 DPI 屏幕；切其他软件隐藏、返回恢复；隐藏/最小化暂停；解绑/换绑/退出后消失。不具备多屏环境时保留未执行，不能以几何测试补成实机通过。

全程未启动应用、overlay、GUI夹具、浏览器、服务器或桌面自动化，未执行真实窗口/目标查询、重跑诊断、清系统缓存或终止外部进程。源码文本有限路径/敏感模式扫描零命中；报告只保存相对路径和脱敏摘要，不保存真实标题/PID/HWND/完整路径、截图或原始日志。二进制精确路径扫描沿用 Coder 证据，非完整元数据审计；无系统级写入监控，不声称系统零外写。

后台输入、游戏响应、热键、点击取点和拖动未实现。若人工效果不满足 POC 目标，由用户决定修正或重新选型，不因编译通过自动扩大集成范围。
