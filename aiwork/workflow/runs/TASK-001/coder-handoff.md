# TASK-001 Coder 交接

日期：2026-10-21。Coder：本任务独立 Coder agent。状态：**READY_FOR_REVIEW（非 UI 独立审核）；人工桌面验收待用户完成**。

用户明确接管桌面验收，因为仍在操作电脑。此后未再调用桌面交互、截图、输入、启动或关闭窗口；不得将本记录中的进程或文件观察当作 UI/IPC 呈现通过。

## 稳定快照

- 标识：`TASK-001-CODER-S1-FINAL`；Git 基线 `d3d8f58`，实现尚未提交。
- 源码、配置、两个锁文件、相关入口说明共 32 个文件的 SHA-256：见 [source-manifest.sha256](source-manifest.sha256)。清单本身 SHA-256：`58c49867e0bfc666674fe9073bae854820462036a93804a9788199d7ccf29c98`。
- 此清单冻结生产源码、测试源码、构建配置与锁文件；任务协调文档不在生产源码清单内。交接后 Coder 不再改被验文件。
- 允许的后续测试输出为独立验收副本内的 `build/verification/TASK-001/`、该副本的 `build/` 缓存及 `node_modules/`；不修改原工作区生产配置、锁文件或用户正在操作的运行数据。
- 原有未提交的选型、需求、目录、workflow、版本、release 文档均保留；未重置、提交、推送或发布。

## 改动

| 范围 | 内容 |
| --- | --- |
| 根级工程 | package/锁文件、index、TS/Vite 配置，官方常用的根 `src/` 与 `src-tauri/` 布局 |
| `src/` | 紧凑纵向四区域、左/右/键盘模式切换、客户区坐标展示、CSS 圆角/轻微动效/减少动效、禁用的未实现操作 |
| `src-tauri/` | Rust 入口、真实版本/偏好 IPC、托盘还原/禁用停止/退出、默认收起托盘、任务栏偏好、关闭退出、数据目录初始化、原子配置保存与异常保护 |
| `scripts/` | npm 安装前环境包装、下游统一环境及开发/构建/测试入口、自绘 PNG/ICO 可重建脚本 |
| 说明与忽略 | 更新根及相关源码目录 README；忽略运行 data 与 PDB，未改写冻结业务需求 |

未实现窗口枚举、绑定、热键、输入发送或业务输入配置保存；界面无虚构的已枚举窗口、投递结果或运行成功状态。托盘的停止菜单与全部执行按钮禁用。

## 依赖和环境

- 实测系统：Windows 10 build 19045（x64）。Node 24.14.1、npm 11.13.0、rustc/Cargo 1.96.1（`x86_64-pc-windows-msvc`）、已有 VS 2026 18.6.1 x64 编译组件。
- 固定前端：Vue 3.5.44、Tauri API 2.12.4、CLI 2.12.3、Vite 7.3.1、TS 5.9.3、plugin-vue 6.0.10、vue-tsc 3.3.13。
- Rust：tauri 2.12.4、tauri-build 2.7.3，要求 Rust 1.90；完整依赖由 Cargo.lock 固定。安装前核对官方 npm/crates 元数据及现有工具兼容性，未升级全局工具。
- npm 安装结果 48 个包；安装脚本检查仅发现 esbuild 的官方 `node install.js`。未引入组件库、动画库、路由、全局状态或新插件。

## 实际命令与落点

以下命令均从仓库根执行，不是界面操作：

```powershell
./scripts/npm.ps1 install
./scripts/npm.ps1 run typecheck
./scripts/npm.ps1 run build:desktop
./scripts/npm.ps1 run test:backend
```

首次 Cargo 锁文件由统一环境下的 `cargo generate-lockfile --manifest-path src-tauri/Cargo.toml` 生成。后续桌面构建为 Tauri `build --no-bundle -- --locked`，内部先构建前端；纯文件测试为 `cargo test --release --locked --manifest-path src-tauri/Cargo.toml`。未运行安装器打包、runtime 安装、游戏或正式 release。

`scripts/npm.ps1` 在 npm 本身执行前设置 TEMP/TMP/TMPDIR、npm 缓存、Cargo HOME/target、Node 缓存；`environment.mjs` 对下游再次设置并传入子进程。临时文件为 `build/tmp/`，缓存为 `build/cache/{npm,cargo-home,node}/`，Rust 输出 `build/cargo/`，前端输出 `build/frontend/`。Rust 使用项目路径重映射和基于项目的 dep-info 设置；不重定义 HOME/USERPROFILE/既有 Rust 工具链位置。npm 诊断日志留存已关闭，首次任务产生的 npm 诊断日志已移除，不保留原始日志或完整环境。

最终构建与完整测试通过后，只复制最终 EXE 到下述固定验收目录，未启动。Vite 构建清空输出时移除的原 `.gitkeep` 已原样恢复；未清理任何历史缓存或外部进程。

## 产物与人工验收绑定

| 用途 | 路径 | SHA-256 |
| --- | --- | --- |
| 最终生产构建 | `build/cargo/release/clickmaster.exe` | `E3E0974376F657428003F6F7ACC67D2DE0BDFF303842FACCF559F730EDCEB1D6` |
| 最终人工验收入口，已准备未启动 | `build/verification/TASK-001/coder/final/ClickMaster.exe` | `E3E0974376F657428003F6F7ACC67D2DE0BDFF303842FACCF559F730EDCEB1D6` |
| 用户接管时原先已启动的产物 | `build/verification/TASK-001/coder/runtime/ClickMaster.exe` | `30AE9057744BE91E362F3B1A4448675ABFA371E5B890B53EAF1A74B1AA2FE596` |

两个 EXE 均为 8,993,792 bytes。原产物没有被覆盖，其 data 没有被修改或复制；交接前未终止它。原启动时配置文件不存在，为默认不保留任务栏图标；其后用户是否已修改由人工结果记录，不能继续假定默认。最终验收目录尚无 data，用户自行启动后按 EXE 同级规则创建。

原启动到最终快照之间，生产应用的行为代码没有更改；只新增 `lib.rs` 中 `#[cfg(test)]` 下三个数据根目录/初始化文件测试，并完成重编译。两个二进制哈希不同，不作字节等同承诺；原产物的人工结果仅绑定原哈希，最终产物仍由用户决定并完成对应人工 gate。

原生产 EXE 曾由非 UI 启动命令从仓库根作为工作目录运行，观察到真实进程响应及 EXE 同级 `data/config/`、`data/webview/EBWebView/`，能支持非 CWD 定位的文件证据。它不替代界面、版本呈现、托盘或任务栏验证。没有启动 Vite/Tauri 开发服务器或其他自行运行的后台服务。

## 自测

| 验收 | 结果 | 证据及边界 |
| --- | --- | --- |
| AC-01 | PASS | 32 文件稳定清单，两个完整锁文件与根级布局；保留既有本地改动 |
| AC-02 | PASS | 类型检查与 Vite 生产构建成功；Windows x64 Rust release 桌面构建成功、无 installer；最终一次生产重建 2m25s |
| AC-03 | NOT RUN | 独立隔离还原由 Verifier 执行；Coder 未在本工作区运行 ci 删除依赖 |
| AC-04 | HUMAN PENDING | 曾启动且有响应的真实进程，仅此；缩放/最小化/还原/关闭均交用户，未声称完成 |
| AC-05 | HUMAN PENDING | Rust 命令与前端 invoke 真实接线，运行时呈现尚未由人工确认；不以版本字符串静态相等替代 |
| AC-06 | HUMAN PENDING | 静态检查四区域/三模式/标签/焦点/减少动效；实际布局、Tab 和缩放待人工 |
| AC-07 | PASS（源码范围） | 无业务热键、目标枚举/绑定、PostMessage 或全局输入实现；未启动游戏。还原自身窗口不等于激活游戏 |
| AC-08 | PARTIAL / HUMAN PENDING | 纯文件数据根目录、初始化阻断与探针清理测试通过；旧产物真实创建 EXE 同级数据。开发实际启动、不可写原生提示、WebView有效目录完整检查未运行 |
| AC-09 | PASS（限定范围） | 源码/配置/锁文件常见路径及凭据扫描无命中；最终EXE对当前项目/用户目录的ASCII与UTF-16精确字节扫描无命中。非完整二进制/工具链元数据审计 |
| AC-10 | PASS | README入口、清单、命令、边界、产物摘要和人工步骤已交接 |
| AC-11 | HUMAN PENDING | 默认隐藏/跳过任务栏及托盘失败保护有源码路径；任务栏/托盘实际操作及失败恢复尚未测 |
| AC-12 | HUMAN PENDING | 配置读写文件测试通过；任务栏偏好UI、重启与窗口显示行为待人工 |
| AC-13 | HUMAN PENDING | 自身退出、托盘停止禁用有源码；关闭/托盘退出及归属子进程清退待人工 |
| AC-14 | PARTIAL / HUMAN PENDING | 非法JSON保护、Windows文件锁造成的真实替换失败、原文件/有效偏好保留、解除锁后恢复保存均PASS；UI错误呈现待人工 |
| AC-15 | PASS（受管理落点） | 安装/构建/测试环境及子进程传递、目录清单已核对，新增测试/最终验收文件在项目内；无系统级文件监控，未证明整个系统零写入 |

完整后端测试：6 passed、0 failed。覆盖：EXE父目录定位、不依赖工作目录；config目录冲突时在创建webview前失败；写入探针不残留；偏好重启读取及未知字段保留；非法JSON不被覆盖；Windows文件锁导致写入失败后原文件及有效偏好保留，解除锁后可保存。测试只操作项目内专用夹具，不启动本工具或操作用户配置。

## 人工清单与限制

人工按最终产物哈希记录；由用户自行决定何时退出旧产物并启动 final 入口，不由 agent 代替操作：

1. 观察版本 `0.1.0-dev.0`、四区域及三种输入切换；绑定、刷新、单次、开始、停止均禁用；Tab/缩放时内容可用。
2. 默认最小化后任务栏无本工具图标，托盘左键和“打开主窗口”可恢复；正常显示时有任务栏入口。
3. 开启“最小化时保留任务栏图标”，最小化、恢复、关闭再启动，核对偏好及正常可见启动；关闭偏好后重复。
4. 验证关闭 × 和托盘退出，停止菜单禁用，托盘自身图标移除；不结束外部应用。异常配置/写权限可在单独隔离副本由人工完成。

Computer Use 文本观察返回 accessibility=null；首次窗口截图超时，恢复尝试时收到用户按 Escape 停止信号，随后用户明确接管人工验收。此后未再调用或用其他工具绕过桌面停止；没有保存截图。

轻量化：EXE约8.58 MiB；前端产物约71.07 KB JS、5.14 KB CSS（未压缩传输口径，非完整运行开销）。原启动时主进程工作集单点观察约28.5 MiB；未采集全部WebView子进程的空闲资源、准确启动耗时或CPU基线，不能据此判定整体轻量化通过。共享WebView2具体版本与Windows11验证未完成。

缓存/编译中间文件与WebView内部数据不是对外交付的脱敏证据，未对其所有依赖内部元数据作全面审计；不归档原始PDB、堆栈或完整工具输出。运行时和操作系统/执行平台自身记录不属于已证明可完全控制的边界。Windows11、后台输入、游戏兼容、热键、正式便携发布与缺失Runtime流程均未运行。

Verifier 仅执行只读源码审查、隔离构建和非UI文件测试，不再启动或操作桌面窗口。先核对 manifest，再复制必要源文件到自己的固定隔离根，经该副本的 `scripts/npm.ps1 ci` 还原并运行约定检查；清单、锁文件须前后保持一致。人工 gate 保留给用户。
