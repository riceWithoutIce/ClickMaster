# 项目目录与版本管理约定

日期：2026-10-21。

依据：用户确认采用 Tauri 官方常用的根级前端与后端布局，并授权基础目录准备和一次 Git 提交。目录说明、输出槽位及 Git 规则已准备；未初始化 Tauri、安装依赖、构建或发布。

## 目录职责

```text
<repository-root>/
  .gitattributes       文本换行与二进制类型规则
  .gitignore           生成物、依赖和本地配置忽略规则
  README.md            项目入口
  src/                 前端源码预留，组件框架待定
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
    archive/<version>/ 最近历史正式版，最多 2 个
    preview/<version>/ 当前候选版，最多 1 个
  scripts/             后续统一开发/构建/版本/发布管理入口
  aiwork/
    workflow/          协作流程、脱敏规则与交接模板
      templates/       任务、Coder、Verifier 模板
    debug/             调试记录说明与模板
    ...                需求、设计、路线、开发日志与验收方案
```

树中的目录已建立，构建槽位使用 `.gitkeep` 保留在 Git 中，不包含编译结果。旧的 `src/frontend/`、`src/src-tauri/` 占位已整理为上述根级布局。

工程初始化后再增加仓库根的 `package.json`、前端锁文件、`index.html`、TypeScript/构建工具配置，以及 `src-tauri/` 内的 Cargo、Tauri 配置与源码。当前不生成依赖清单或空入口，避免提前选定框架与包管理器。构建脚本实现后需检查实际输出位置，不能将目录约定记为已接通。

## 存储与追溯

- `src`、`src-tauri`、`scripts`、`version`、`aiwork`、根配置及目录占位纳入 Git。
- 构建输出、依赖、临时产物和 release 二进制由 `.gitignore` 排除；源文件中的锁文件仍需跟踪。
- 版本号采用 SemVer；现存 `0.1.0-dev.0` 是暂存的开发标识，首个正式开发版本仍待确认，当前发布索引为空。
- Git 的 `main` 分支接续远端初始历史，`origin` 指向 [项目仓库](https://github.com/riceWithoutIce/ClickMaster)；源码历史通过提交/标签追溯，不按版本复制项目。本轮授权为本地提交，不包含推送或发布。
- 发布只保存选定的主要交付格式与架构，默认上限为当前 1 + 历史 2 + 候选 1，共 4 套；格式与架构本身尚待选定。
- 稳定保留与回退优先；晋级失败时保留可恢复的临时产物并暂停下一次发布，不继续堆积。
- 索引保留版本、来源、日期、大小、SHA-256、验证结论及 current / archived / preview / removed 状态信息；旧二进制移除后仍可追溯，但不能保证恢复原二进制。
- 归档不是第二套全量备份；archive 同样有数量上限。没有自动云端/外盘永久备份或远程发布承诺。

## 清理与证据边界

详见 `release/README.md` 和 `build/README.md`。仅把明确可再生的构建输出和清单可归属的过期发布产物纳入清理；原始用户数据、未知文件、未解决问题证据与其他项目不在范围内。

必要验收结果留在 aiwork 或发布元数据；不能把唯一验收证据留在随后会淘汰的普通日志槽位。

运行时配置、日志、WebView 用户数据和缓存位置另行设计，不使用源码或 release 目录承载。Cargo/npm 下载缓存策略也待工程配置明确，本轮没有改写用户级配置。

## 仍需落实

- 前端框架、包管理器与锁文件选择。
- 最低 Windows 版本、WebView2 分发策略和安装器格式。
- 版本同步、安装器版本映射、构建输出重定向及受管理产物清理脚本。
- 运行时数据位置与清理规则，轻量化的包体/运行资源预算。

以上为后续设计与实现任务，本轮未执行。
