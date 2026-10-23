# TASK-001-UI R1 独立复审报告

日期：2026-10-21。复审对象：`TASK-001-UI-R1-CODER-S1`。范围仅限原 EXE/标题栏旧图标 finding 及紧邻构建校验。

## 结论

**原 finding 的静态资源修复与构建跟踪复核通过，未发现 BLOCKER 或 FOLLOW-UP。** 新 EXE 的 PE 图标组及载荷与当前 ICO 一致，旧 EXE 对当前 ICO 保持预期失败；其余前轮人工通过结果保持原记录。实际 EXE 与标题栏显示仍需用户对新入口复验，资源解析不代替桌面显示证据。

## 稳定快照

- R1 清单：37 文件，SHA-256 `1A8FC6A738CEF232C38865ACCBC9C2661E77A740B7F80606E6E049F46182B5FA`。
- 复审开始与结束核对：原始 R1 37 文件均 0 mismatch；未修改生产源码、资源、锁文件或 Coder 交接。
- R1 人工入口：`build/verification/TASK-001-UI/coder/r1/final/ClickMaster.exe`。
- R1 EXE SHA-256：`8EA412D1DCE51CAF6A3A00309864D53E8BF354AAA86C7FB90A6BA3C04D23A43E`，大小 8,998,912 bytes，与交接一致。
- 未启动 EXE、浏览器、Explorer 或桌面自动化；未清理图标缓存，未触及用户运行数据或旧入口。

## 原 finding 复核

使用 `scripts/verify-pe-icon.mjs` 对 PE 资源和 ICO 做离线解析，并以独立 PowerShell 解析交叉核验；没有加载或执行被测 EXE。

| 对象 | 当前 ICO | 结果 |
| --- | --- | --- |
| UI-S1 旧入口 | 新 ICO（1,104 bytes，SHA-256 `AC160094426F536177AD7101CCBFF0EDE9FC5B1EE2A9684DCD88611F70756EB9`） | **预期失败**，CLI exit 1；PE `RT_GROUP_ICON` 32512 / language 1033 引用 `RT_ICON` 1，实际旧载荷 319 bytes，SHA-256 `CC636D42441EA874DA96FC9957D2B091286F2322AED77B98AD9420BB29AFBC7A` |
| UI-S1 旧入口 | 旧 ICO（319 bytes） | **通过**，说明旧入口确实仍绑定旧图标资源 |
| UI-S1 R1 新入口 | 新 ICO（1,104 bytes） | **通过**，同一图标组引用 `RT_ICON` 1，载荷 1,104 bytes、SHA-256 `AC160094426F536177AD7101CCBFF0EDE9FC5B1EE2A9684DCD88611F70756EB9`，尺寸描述、数量及逐字节载荷均匹配 |

独立解析确认新 PE 的 `RT_GROUP_ICON` 32512 / language 1033 → `RT_ICON` 1 → 新 ICO PNG 载荷；与脚本输出一致。旧 319-byte 载荷在新 EXE 中没有字节命中辅助检查，最终结论以 PE 资源归属解析为准。

## 构建规则复核

- `src-tauri/build.rs` 新增并实际输出两条 Cargo build-script 规则：`cargo:rerun-if-changed=icons/icon.ico` 与 `cargo:rerun-if-changed=icons/icon.png`。这提供了图标资源变化的增量构建跟踪，不依赖清空整个缓存。
- `scripts/project.mjs` 仅在桌面构建子进程成功后运行离线 PE 校验；校验失败会使统一入口失败。检查进程继承项目内环境，未引入依赖或外部网络。
- `scripts/verify-pe-icon.mjs` 与 `scripts/project.mjs` 的 Node 语法检查均通过；脚本按资源目录、语言、图标组引用、ICO 描述和逐字节载荷核验，未加载/执行被测程序。
- R1 相对 UI-S1 清单的源码范围仅为 `src-tauri/build.rs`、`scripts/project.mjs`、新增 `scripts/verify-pe-icon.mjs` 及两份对应说明；前端、`lib.rs`、配置、依赖/锁文件、版本与图标素材未改变。

## 桌面边界与停止

本复审不重复前轮导航、草稿、偏好、托盘和布局验收。用户此前反馈的其余范围保持通过；本轮只需对 R1 最终入口复验：

1. EXE 文件图标显示为黄色背景蓝色指针。
2. 启动后窗口标题栏图标显示为同一新图标。

若 Windows 图标缓存仍显示旧图，用户记录实际现象；本轮不自动清理缓存或修改系统设置。上述两项实际显示仍为 HUMAN PENDING，静态 PE 资源通过不能替代观察。

本轮未发现 findings，不要求 Coder 修复。R1 限定复审到此结束；没有进行 Rust/前端全量重建、后端测试或其他全面审计。
