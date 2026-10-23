# TASK-001 独立 Verifier 报告

日期：2026-10-21。Verifier：本任务独立 Verifier agent。

后续人工结果补记（主协调者）：用户已确认窗口与托盘行为通过，见 [人工验收记录](human-acceptance.md)。以下保持独立审查当时的实际执行状态，不把人工结果冒记为 Verifier 工具实测。

## 验收对象与结论

- 被验快照：`TASK-001-CODER-S1-FINAL`，生产源码、配置、测试及锁文件以 [32 文件清单](source-manifest.sha256) 为准；清单 SHA-256 为 `58C49867E0BFC666674FE9073BAE854820462036A93804A9788199D7CCF29C98`。
- 开始、过程中及结束核对：原始 32 文件和隔离副本 32 文件均为 0 mismatch；两个锁文件未被改写。未修改生产源码或 Coder 记录。
- **本次非 UI 验证通过，未发现 BLOCKER 或 FOLLOW-UP findings；整体人工桌面验收尚未完成，不能据此宣称 TASK-001 无条件通过。**
- 用户正在操作电脑，已明确接管桌面验收。本次未启动应用 EXE、浏览器或开发服务器，未调用桌面自动化、截图、激活、最小化或关闭操作，也未触及原运行产物及其数据。
- 隔离根：`build/verification/TASK-001/verifier/source/`。只按清单复制必要文件，不复制 `.git/`、依赖目录或编译缓存；npm/Cargo 下载缓存从原项目复制为隔离种子，后续使用副本自己的缓存和输出。

## 实际检查

以下命令均在隔离根执行，使用该副本自己的包装入口；安装、构建和测试均以退出码 0 结束：

```powershell
./scripts/npm.ps1 ci --offline
./scripts/npm.ps1 run typecheck
./scripts/npm.ps1 run build:desktop
./scripts/npm.ps1 run test:backend
```

- 离线还原安装 48 个 npm 包；类型检查通过。
- 桌面入口先完成 Vite 生产构建，随后以 `--no-bundle -- --locked` 构建 Rust release；全量 Rust 构建约 5 分 14 秒，无安装器产物。
- 后端 release 测试构建约 2 分 15 秒，6 passed、0 failed；测试运行约 0.03 秒。只操作副本内文件夹具，不创建桌面窗口。
- 核对包装脚本在 npm 启动前设置 TEMP/TMP/TMPDIR、npm/Node/Cargo 缓存和 Cargo target；下游脚本再次传递这些设置。受管理产物均在隔离根内。

| 验收 | 结果 | 证据及适用边界 |
| --- | --- | --- |
| AC-01 | PASS | 根级 Vue/TS 与 `src-tauri/` 布局、两个锁文件、版本同步及依赖约束核对通过；原始快照未改动 |
| AC-02 | PASS | 类型检查、前端生产构建及 Windows x64 Rust release 构建通过；未打包安装器 |
| AC-03 | PASS | 清单还原的独立副本完成离线 npm ci 和 Cargo locked 生产构建；构建后清单与锁文件均不变 |
| AC-04 | NOT RUN | 原生启动、缩放、最小化、还原及关闭交用户人工验证；本次未运行 UI |
| AC-05 | INSUFFICIENT EVIDENCE | Rust `get_app_info` 已注册，前端真实 invoke 并显示返回版本及初始化错误路径；运行时 IPC 呈现仍待人工 |
| AC-06 | INSUFFICIENT EVIDENCE | 静态确认四区域、左/右/键盘模式、标签/焦点/减少动效及执行操作禁用；实际布局、Tab、缩放待人工 |
| AC-07 | PASS（源码范围） | 无目标枚举/绑定、窗口消息输入、全局输入或业务热键代码；本次未操作目标应用 |
| AC-08 | INSUFFICIENT EVIDENCE | EXE 父目录定位、配置目录冲突时先失败、探针清理测试通过；WebView builder 创建前设置同级目录。开发真实启动、生产有效 WebView 路径及不可写原生提示待人工；未以源码代替运行证据 |
| AC-09 | PASS（限定扫描范围） | 32 文件文本扫描无本机路径/常见凭据模式命中；独立 EXE 对当前项目及用户目录的 ASCII/UTF-16 精确扫描零命中；完整二进制和工具链元数据未穷举 |
| AC-10 | PASS | 根 README 包装入口与实际命令一致；交接含文件清单、产物哈希、边界、未执行项及有限资源观察，没有将其当作完整性能证据 |
| AC-11 | NOT RUN | 静态确认托盘保持、菜单/左键还原、默认隐藏与任务栏移除、失败回退逻辑；真实任务栏/托盘与失败恢复交人工 |
| AC-12 | INSUFFICIENT EVIDENCE | 偏好序列化与重新读取通过，前端保存成功后更新有效值、失败恢复原值；实际开关、重启和窗口状态待人工 |
| AC-13 | NOT RUN | 静态确认关闭和托盘退出使用自身应用退出接口，停止菜单禁用，没有外部进程终止代码；真实进程/托盘清退待人工 |
| AC-14 | INSUFFICIENT EVIDENCE | 非法 JSON 保留、Windows 文件锁导致替换失败、原文件和有效偏好保留、解除锁后保存恢复均通过；界面错误反馈待人工 |
| AC-15 | PASS（受管理落点） | 独立安装/构建/测试使用副本内临时目录、缓存及输出；文件夹具位于副本内。没有系统级写入监控，不宣称已证明整个系统零外写 |

## 文件测试与产物

6 项文件测试实际通过：

1. 数据根目录跟随 EXE 父目录，不依赖工作目录。
2. config 目录被文件阻挡时，在创建 webview 目录前失败。
3. 可写数据根初始化后无写入探针残留。
4. 偏好保存后重新读取成功，未知字段保留。
5. 非法 JSON 不被覆盖，保持默认偏好并提供错误原因。
6. Windows 文件锁使原子替换真实失败后，原文件和上次有效偏好保持；解除锁后可重新保存。

文件夹具虽使用测试代码中的 `coder/` 子目录名称，其实际完整相对位置位于本次隔离根的 `build/verification/TASK-001/coder/` 内，不是原工作区 Coder 数据。结束时 pending 文件与写入探针残留均为 0。

| 对象 | 路径或值 |
| --- | --- |
| 独立构建 EXE | `build/verification/TASK-001/verifier/source/build/cargo/release/clickmaster.exe` |
| 独立 EXE SHA-256 | `C4A173148D7B22313EE202A51D41CCF64D503F76509A87A92E0AC5899CFC44CE` |
| 独立 EXE 大小/架构 | 8,993,792 bytes；PE machine `0x8664`（x64） |
| 独立前端 JS / CSS 大小 | 71,074 / 5,137 bytes；不代表完整运行资源开销 |
| 用户最终人工验收入口 | `build/verification/TASK-001/coder/final/ClickMaster.exe` |
| 用户最终入口 SHA-256（复核一致） | `E3E0974376F657428003F6F7ACC67D2DE0BDFF303842FACCF559F730EDCEB1D6` |

两个 EXE 哈希不同，不宣称确定性字节复现。独立构建用于核实源码可还原构建，未替换用户最终入口，也未启动任一 EXE。本次副本未生成运行时 `data/` 或开发运行数据目录。

## Findings 与停止

本次冻结范围内未发现 findings；不要求 Coder 修复或扩展功能。

本轮独立非 UI 审查已结束。人工 gate 保留给用户：最终产物界面/IPC、默认托盘与任务栏偏好、重启、关闭与退出、不可写提示及配置异常反馈。既有旧产物的人工结果不能自动绑定最终产物哈希。Windows 11、游戏/后台输入、热键、正式便携发布、缺失 Runtime 处理实测及完整空闲资源基线均无本次通过证据。

本文只保存脱敏摘要和相对路径，不保留原始工具日志、PDB、未脱敏截图或环境转储。扫描并非完整敏感信息审计；缓存、系统/共享 Runtime 和执行平台内部记录没有被当作已全面受控的范围。
