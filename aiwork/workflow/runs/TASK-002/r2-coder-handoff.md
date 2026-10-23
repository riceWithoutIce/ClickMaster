# TASK-002 R2 Coder 交接

日期：2026-10-21。本轮唯一源码写入者为Coder。状态：**READY_FOR_REVIEW**，源码已冻结。以[R2契约](r2-task.md)为边界，修复扩展范围排除工具型顶层窗口的问题；真实后台绑定仍待人工验收。

## 稳定快照与产物

- 快照：`TASK-002-R2-CODER-S1`。以R1冻结清单为基线，恰好4文件变化：`src-tauri/src/windows_query.rs`、其目录README、根README及`src/App.vue`说明文字。其余38文件保持R1哈希，包括身份状态、IPC、筛选辅助模块、两个锁文件及图标资源。
- [42文件源码清单](r2-source-manifest.sha256)SHA-256：`11b9a4a63a66bb311fb99443ca5f43bdec36c2f9c390ca6fd9713ec620fcfade`。
- 最终入口：`build/verification/TASK-002/coder/r2/final/ClickMaster.exe`；9,063,424 bytes；SHA-256：`f724722928e50e16846d375aaaaa66027eea9a0c04ca069ed379bff3764ff0ac`。
- final与本次`build/cargo/release/clickmaster.exe`一致，独占创建、未启动，未生成final/data。原TASK-002及R1 final哈希均核对保持不变。
- 结构化摘要及图标证据：`build/verification/TASK-002/coder/r2/evidence.json`、`pe-icon.json`；项目内临时收口脚本为同目录`finalize.mjs`，包含旧final/锁文件校验及防覆盖检查。
- 交接后停止修改清单内源码；父级可在核对后准入独立限定Verifier。

## 实现与范围

- `windows_query.rs`唯一生产逻辑变化：工具窗口条件从`!window.tool`改为`include_hidden || !window.tool`。只有开启现有扩展范围才允许工具型窗口，默认范围仍排除。
- owned且无APPWINDOW、NOACTIVATE、自身和系统Shell等条件未变。没有新增窗口类型字段、设置、路由、依赖、版本或配置格式。
- 纯规则测试新增工具型窗口默认排除、扩展允许、隐藏/空标题工具窗口、owned+APPWINDOW，以及工具标志与self/owned无APPWINDOW/noactivate/system组合仍排除。原非工具窗口范围测试保留。
- 根`README.md`、`src-tauri/src/README.md`及`src/App.vue`现有说明补充“含工具型窗口”；未修改其他UI逻辑。
- R1可信token、绑定前HWND/PID/创建时间/线程校验、周期失效、隐藏/空标题保持、刷新与筛选保护未改。目标名无特判，不加入子窗口、message-only或全进程枚举。

## 验证命令

工作目录为仓库根，沿用项目内环境包装入口：TEMP/TMP/TMPDIR、npm/Cargo/Node缓存与编译输出均在项目内。Rust格式化同样由`projectEnvironment()`环境调用Cargo，未修改用户级工具配置。

```powershell
./scripts/npm.ps1 run typecheck
./scripts/npm.ps1 run test:targets
$env:CLICKMASTER_TEST_ROOT = Join-Path (Get-Location) 'build/verification/TASK-002/coder/r2/tests'
./scripts/npm.ps1 run test:backend
./scripts/npm.ps1 run build
./scripts/npm.ps1 run build:desktop
node build/verification/TASK-002/coder/r2/finalize.mjs
git diff --check
```

完整后端文件测试使用新的`coder/r2/tests/`，不改写旧夹具。Verifier复审应使用自身新测试根，按契约仅复审工具窗口规则及紧邻排除/身份回归，不重复完整Rust release构建或无关全面审计。

已完成类型检查、生产筛选辅助函数测试及前端构建；完整Rust release测试22 passed/0 failed，执行0.01s，编译链接1m35s。前端18模块，JS86.68KB/CSS7.49KB。测试使用虚构窗口，原身份/隐藏保持/刷新隔离回归均通过。

Windows x64 release桌面构建PASS，2m10s，未打安装器或运行应用。PE图标校验PASS：group32512/lang1033引用RT_ICON1，1104 bytes，SHA-256 `ac160094426f536177ad7101ccbff0ede9fc5b1ee2a9684dcd88611f70756eb9`，与当前ICO一致。全部约定非UI检查通过。

本轮源码及说明常见本机路径/凭据模式扫描无命中，`git diff --check`通过。final对实际仓库路径两种斜杠形式及用户目录UTF-8/UTF-16精确扫描无命中；此为有限路径扫描，非完整二进制元数据审计或整个系统零外写证明。

## 证据边界

纯规则及注入查询测试不调用真实EnumWindows/inspect，不创建GUI夹具。本轮没有启动应用、浏览器、开发服务器或桌面自动化，没有再次诊断/查询/绑定真实目标，也没有提权、发送输入或更改目标窗口。

R1只读诊断只用于确认此前排除原因；本轮规则通过不代表真实可绑定、所选窗口就是游戏内容或游戏响应输入。用户须从R2最终入口开启扩展范围、筛选并绑定，继续分层人工验收。后台输入尚未开发。

全部缓存、中间产物与验收输出保持项目内，旧final/data/夹具/诊断及历史证据保留；不提交、推送、发布或清理历史。父级维护契约、人工记录、roadmap和devlog，本轮未修改这些文件。
