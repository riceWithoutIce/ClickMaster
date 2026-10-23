# TASK-001-UI R1 Coder 交接

日期：2026-10-21。状态：**READY_FOR_REVIEW**，限定修复EXE及标题栏旧图标；生产源码/配置/锁文件与资源已冻结，未启动程序。

## 原问题、原因与修复

用户反馈前轮EXE和标题栏仍显示紫C，其余人工验收通过。本轮仅修原finding的编译资源及紧邻构建校验，没有重做UI。

离线PE资源解析确认：前轮final的 `RT_GROUP_ICON` ID32512、language1033引用 `RT_ICON` ID1，载荷319 bytes、SHA-256 `cc636d42441ea874da96fc9957d2b091286f2322aed77b98ad9420bb29afbc7a`，与前轮旧ICO严格一致；对当前新ICO（1104-byte PNG）比较失败。此前EXE含新PNG只能证明其他嵌入资源已更新，不能证明Windows应用图标资源正确。

构建入口未显式跟踪ICO/PNG变更，Cargo缓存保留旧Windows资源。`build.rs`新增两条 `cargo:rerun-if-changed=icons/icon.ico` / `icons/icon.png`；本次使用原缓存增量构建，未删除缓存，即使图标本身不再改动也因build.rs改变重新编译资源。实际Cargo build-script输出已包含两条跟踪规则，证据见下。

新增无依赖 `scripts/verify-pe-icon.mjs`，只读解析PE资源目录与ICO，不加载或执行EXE。对每个图标组核对其引用的图像数量、尺寸描述、长度与逐字节载荷；任意不一致返回非零。桌面构建入口在Tauri/Cargo成功后运行此检查，失败则整体命令失败，避免仅查源码图标即交付。

没有额外覆盖窗口图标：Windows默认窗口图标继续使用修复后的应用PE资源；`lib.rs`、前端、配置格式、依赖/锁文件、版本、素材均与前轮快照一致。

## 修改范围与稳定快照

- 修改：`src-tauri/build.rs`、`scripts/project.mjs`、`scripts/README.md`、`src-tauri/icons/README.md`；新增 `scripts/verify-pe-icon.mjs`。仅后两份说明记录对应新规则。
- 快照：`TASK-001-UI-R1-CODER-S1`；Git基线 `d3d8f58`，全部原有未提交状态保留；未提交/推送/发布。
- [完整37文件源码/配置/锁文件清单](r1-source-manifest.sha256)，清单SHA-256 `1a8fc6a738cef232c38865accbc9c2661e77a740b7f80606e6e049f46182b5fa`。
- 最终人工入口：`build/verification/TASK-001-UI/coder/r1/final/ClickMaster.exe`；8,998,912 bytes；SHA-256 `8EA412D1DCE51CAF6A3A00309864D53E8BF354AAA86C7FB90A6BA3C04D23A43E`。
- 此final与本次 `build/cargo/release/clickmaster.exe`一致，只复制未启动，无data；旧final、用户运行目录及data未覆盖、未清理。
- 交接后被验文件冻结；Verifier仅复查原图标finding及资源构建邻近回归，不修改源码。所有新增验证输出留自身项目内隔离目录。

## 实际检查

从仓库根执行，沿用项目内临时/缓存/编译输出环境，无依赖安装：

```powershell
node scripts/verify-pe-icon.mjs build/verification/TASK-001-UI/coder/final/ClickMaster.exe
node scripts/verify-pe-icon.mjs build/verification/TASK-001-UI/coder/final/ClickMaster.exe build/verification/TASK-001/verifier/source/src-tauri/icons/icon.ico
./scripts/npm.ps1 run build:desktop
```

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| 旧产物对新ICO | EXPECTED FAIL | 资源解析严格不匹配；检查脚本CLI退出1 |
| 旧产物对旧ICO | PASS | 旧group引用资源与旧ICO逐字节一致，CLI退出0 |
| 图标变更跟踪 | PASS | 实际Cargo build-script输出两条rerun规则，不依靠清全缓存 |
| 生产桌面构建 | PASS | Windows x64 release增量构建3m06s，锁定依赖，无installer；同入口前端构建通过 |
| 新产物对新ICO | PASS | group32512/lang1033引用icon1，载荷1104 bytes，SHA-256 `ac160094426f536177ad7101ccbff0ede9fc5b1ee2a9684dcd88611f70756eb9`，描述/数量/载荷一致；构建末端自动检查退出0 |
| 旧资源残留辅助检查 | PASS | 旧319-byte PNG在新EXE中无字节命中；资源归属结论仍以PE解析为准 |
| 入口脚本语法 | PASS | 两个JS脚本node --check通过 |
| 生产应用范围 | PASS | 相比UI-S1清单仅上述4个已有文件变更+新增验证脚本；前端/lib/settings/两个锁文件/图标资源未改 |
| 脱敏与格式 | PASS（限定范围） | 本轮文本常见本机路径/凭据扫描无命中、git diff --check通过；最终EXE对当前项目/用户目录ASCII和UTF-16精确扫描无命中，非完整二进制审计 |
| 桌面实际显示 | HUMAN PENDING | 未启动EXE/浏览器/Explorer、未操作窗口、未清用户系统图标缓存；由用户只复验EXE及标题栏图标 |

证据均为项目内脱敏JSON/跟踪摘要，不含原始完整日志：

- `build/verification/TASK-001-UI/coder/r1/old-pe-resources.json`
- `build/verification/TASK-001-UI/coder/r1/new-pe-resources.json`
- `build/verification/TASK-001-UI/coder/r1/build-tracking.txt`

本轮不重复已人工通过的导航/草稿/偏好/托盘完整验收，不重复无变化的后端文件测试，也不以资源检查替代Windows实际图标呈现。用户此前其余通过结果仍绑定前轮产物，当前修复仅影响构建资源及其校验。没有新后台服务或开发服务器。

保留限制：本地源图当前为单张48×48 RGBA，此修复不扩展多分辨率图标设计；资源检查仅处理标准Windows PE的静态RT_GROUP_ICON/RT_ICON及ICO，不是通用二进制安全审计。操作系统图标缓存如仍显示旧图，需由人工记录现象，本轮不自动清系统缓存或更改系统设置。
