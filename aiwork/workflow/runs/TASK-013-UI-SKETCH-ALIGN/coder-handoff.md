# TASK-013 Coder 稳定交接

状态：Coder 完成，源码冻结供独立 Verifier；未提交、未推送、未正式发布。

## 实现范围

- `src/App.vue`：保留浅色主题，按精确草图去面包屑，标题增加按需帮助；目标搜索/刷新同排，窗口详情、绑定状态与操作集中；专用模式只显示简短说明，中心坐标时隐藏自定义 X/Y；执行参数采用分组、单位和动态等待说明；底部更多菜单打开独立诊断对话框。
- `src/style.css`：对齐固定导航、状态标签、卡片/表单间距、两行底部状态与操作；补充低高度紧凑和窄窗口布局；弹窗不挤占页面流，支持内容滚动。
- `src/NavIcon.vue`：增加本地帮助、搜索、检查、更多、诊断和关闭图标，无新增依赖。
- 保留所有诊断日志、错误、回执、IPC 连接/写入/读取、执行摘要、性能采集及实验预览入口；技术说明进入帮助/诊断，接收设置进入窗口详情。
- 新对话框支持关闭、知道了、原生 Escape 和焦点返回；不捕获 F8，后端全局停止逻辑未改。
- 同步自有版本为 `0.1.0-rc.9`，更新 changelog 和 `docs/USAGE.md` 的实际入口说明。`version/README.md` 旧 rc.6 标记按任务约定维持历史 P2；`docs/LIMITATIONS.md` 沿用现有 rc.7 技术限制，不声明新一轮运行验收。

## 自测与边界

- `./scripts/npm.ps1 run typecheck`：通过。
- `./scripts/npm.ps1 run build`：通过；桌面构建前置步骤再次构建最终低高度样式，通过。
- `./scripts/npm.ps1 run build:desktop`：通过；自动 PE 图标核验通过。
- ZIP 包含 EXE、README、CHANGELOG、使用说明及限制；不包含用户 data。ZIP 内 EXE hash 与候选 EXE 一致，签名状态 NotSigned。
- 初次前端构建曾发现迁移残留闭合标签，已修正后复检通过；详见 `build/verification/TASK-013-UI-SKETCH-ALIGN/coder/checks.md`。
- 默认窗口 800×520、最小 520×400 未更改。常用永久执行页收紧，限时/自定义/窄窗口可通过主区滚动访问，底部操作固定；这仅是静态布局证据，未声称已实际渲染或零滚动。具体 CSS 与交互依据见 checks.md。
- 未运行后端测试、真实 GUI/游戏、F8/托盘运行、性能、长期运行或正式发布验收；未启动普通桌面程序或浏览器。
- 所有构建和打包使用项目内环境入口；生成物位于 `build/`，未更改全局工具/配置，也未声称系统内部零外写。

## 稳定快照与改动边界

- 源码 manifest：`build/verification/TASK-013-UI-SKETCH-ALIGN/coder/source-manifest.json`，84 个唯一文件，SHA256 `12882DA4D4B1DB411E8B9C047B3DC70046C2640A330A836F5864E941816B1521`。
- rc.8 差异：`build/verification/TASK-013-UI-SKETCH-ALIGN/coder/rc8-changes.json`。旧 manifest 的 89 项有 7 个重复路径；本轮去重为 82 个并补入两个交付 docs，故总数 84，不表示删除源码。
- 相对 rc.8，变更仅 `src/App.vue`、`src/NavIcon.vue`、`src/style.css`、自有版本 metadata、`version/VERSION` 与 `version/CHANGELOG.md`；使用说明在旧 manifest 外，因此本轮单独纳入。现有无关 dirty 文件保持不变。
- 所有 `src-tauri/src/` 后端源码与 rc.8 manifest 的 SHA256 一致；未改 IPC 协议、worker、输入语义、调度/性能、F8、托盘或停止。

## 候选

- EXE：`build/staging/TASK-013-UI-SKETCH-ALIGN/ClickMaster.exe`，9774592 bytes；SHA256 `1E0D28BBC669F1E0A4E92CB5DA594D79B4179C83DE841550E84A750B85B31F1E`。
- ZIP：`build/staging/TASK-013-UI-SKETCH-ALIGN/ClickMaster-0.1.0-rc.9-windows-x64.zip`，3091209 bytes；SHA256 `A4BBB6FA2C9D5072D066477EDB0CF1BF1BA027C6EDA3E9E226F79A418422DD27`。
- 来源和校验：同目录 `source.json`、`SHA256SUMS.txt`，本地 dirty 快照，未签名，非正式发布。

等待主协调者核对快照并启动独立只读 Verifier；人工视觉与运行体验继续由用户反馈。
