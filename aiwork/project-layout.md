# 项目目录与版本管理约定

日期：2026-10-21。

依据：用户确认采用 Tauri 官方常用的根级前端与后端布局，并授权基础目录准备和一次 Git 提交。目录说明、输出槽位及 Git 规则已准备；未初始化 Tauri、安装依赖、构建或发布。

## 目录职责

```text
<repository-root>/
  .gitattributes       文本换行与二进制类型规则
  .gitignore           生成物、依赖和本地配置忽略规则
  README.md            项目入口
  src/                 Vue 3 + TypeScript 前端源码预留
  src-tauri/           根级 Rust/Tauri 工程预留
    src/               Rust 源码预留
    capabilities/      Tauri 2 能力声明预留
    icons/             应用图标预留
  build/
    frontend/          后续前端编译输出
    cargo/             后续 Cargo target 与 Tauri 中间产物
    staging/           单个待验证/发布槽位
    logs/              最近 10 次普通构建日志
  version/
    VERSION            版本号唯一人工来源
    CHANGELOG.md       发布变化及 Unreleased
    releases.json      发布元数据和产物保留状态
  release/
    current/           当前正式版，最多 1 个
    archive/           历史记录入口，不保留旧二进制
    preview/<version>/ 当前候选版，最多 1 个
  scripts/             后续统一开发/构建/版本/发布管理入口
  aiwork/
    workflow/          协作流程、脱敏规则与交接模板
      templates/       任务、Coder、Verifier 模板
    debug/             调试记录说明与模板
    ...                需求、设计、路线、开发日志与验收方案
```

树中的目录已建立，构建槽位使用 `.gitkeep` 保留在 Git 中，不包含编译结果。旧的 `src/frontend/`、`src/src-tauri/` 占位已整理为上述根级布局。

工程初始化后再增加仓库根的 `package.json`、`package-lock.json`、`index.html`、TypeScript/Vite 配置，以及 `src-tauri/` 内的 Cargo、Tauri 配置与源码。已确认 Vue 3 + TS + Vite 与 npm；当前尚未生成依赖清单或入口。构建脚本实现后需检查实际输出位置，不能将目录约定记为已接通。

## 存储与追溯

- `src`、`src-tauri`、`scripts`、`version`、`aiwork`、根配置及目录占位纳入 Git。
- 构建输出、依赖、临时产物和 release 二进制由 `.gitignore` 排除；源文件中的锁文件仍需跟踪。
- 版本号采用 SemVer；现存 `0.1.0-dev.0` 是暂存的开发标识，首个正式开发版本仍待确认，当前发布索引为空。
- Git 的 `main` 分支接续远端初始历史，`origin` 指向 [项目仓库](https://github.com/riceWithoutIce/ClickMaster)；源码历史通过提交/标签追溯，不按版本复制项目。本轮授权为本地提交，不包含推送或发布。
- 2026-10-23 用户修订：历史版本留记录，二进制只保留最新已验收的一套；替代原“当前 1 + 历史 2 + 候选 1”规则。首版已选 x64 便携 ZIP，不跨 staging/preview/current 重复存包，详见 [发布规则](../release/README.md)。
- 稳定保留与回退优先；晋级失败时保留可恢复的临时产物并暂停下一次发布，不继续堆积。
- 索引保留版本、来源、日期、大小、SHA-256、验证结论及 current / archived / preview / removed 状态信息；旧二进制移除后仍可追溯，但不能保证恢复原二进制。
- 归档保存版本、来源、哈希、验收和淘汰记录，不复制整套项目或保留旧 EXE/ZIP；不设额外备份目录规避淘汰规则。

## 清理与证据边界

详见 `release/README.md` 和 `build/README.md`。仅把明确可再生的构建输出和清单可归属的过期发布产物纳入清理；原始用户数据、未知文件、未解决问题证据与其他项目不在范围内。

必要验收结果留在 aiwork 或发布元数据；不能把唯一验收证据留在随后会淘汰的普通日志槽位。

历史测试目录保留关键报告、失败证据和必要脚本，删除可再生的重复工程、依赖与编译结果；不要求保留完整历史目录。旧运行数据提取必要配置和问题日志后，清理无用缓存与测试数据。共享构建和下载缓存暂留到正式包完成，再决定深度清理。当前 rc.9 留到正式版验收替换完成后再淘汰。

生产运行数据位于 EXE 同级 `data/`，开发数据隔离于 `build/runtime/dev/`，验收在 `build/verification/` 下独立运行目录执行，详见 [本地数据](local-data.md)。不在源码或 release 槽位运行程序。Cargo/npm 缓存见任务工程约定，本轮没有改写用户级配置。

## 仍需落实

- 具体依赖版本与工具链版本约束；已确认 npm 与 package-lock.json。
- Windows 10 22H2 / Windows 11 x64、共享 Evergreen WebView2 与便携程序的实际配置和验证。
- 版本同步、EXE 版本资源映射、构建输出重定向及受管理产物清理脚本。
- 已确认的数据位置接入、日志保留与清理细则、轻量化的包体/运行资源预算。

以上为后续设计与实现任务，本轮未执行。
