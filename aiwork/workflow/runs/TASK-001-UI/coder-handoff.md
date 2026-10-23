# TASK-001-UI Coder 交接

日期：2026-10-21。状态：**READY_FOR_REVIEW**。本轮生产源码、资源、配置和锁文件已冻结；桌面人工验收待用户完成。

## 稳定快照与产物

- 快照：`TASK-001-UI-CODER-S1`；Git 基线仍为 `d3d8f58`，既有未提交文档及前轮实现均保留，未提交、推送或发布。
- [完整源码清单](source-manifest.sha256)：36 文件，包括原32文件和4个新增源码/资源/类型声明；清单 SHA-256：`d0c0c7d9ef30a4cb7328831d0dceda46355f6d26babb47fd0ddafb76c890d817`。
- 最终人工入口：`build/verification/TASK-001-UI/coder/final/ClickMaster.exe`，8,997,888 bytes；SHA-256：`2AB9F65C084085FD00CF54FB5271FD06E7B942A4054DFF9DE590579CBED80502`。
- 编译输出 `build/cargo/release/clickmaster.exe` 与最终入口一致。本轮只复制文件，**未启动**；final目录尚无运行数据，不覆盖前轮runtime/final目录或data。
- 后续Verifier仅在自身隔离副本写测试输出；本工作区被验源码、配置、锁文件、资源停止修改。协调文档可单独补充，但清单内README改动会改变快照。

## 修改清单

| 文件 | 修改 |
| --- | --- |
| `src/App.vue` | 五页局部导航：连点控制下的目标窗口、输入方式、执行参数，以及设置/关于；默认目标页、展开侧栏，可收起；顶部摘要/底栏常驻；真实偏好IPC和版本迁移；跨页模式/中心选项/X/Y草稿 |
| `src/style.css` | 侧栏/页面滚动区/常驻区域布局，中性蓝配色、基础圆角动效、窄宽重排、焦点和减少动效 |
| `src/NavIcon.vue`（新增） | 项目内少量SVG线条图标，无字体/图标库依赖 |
| `src/vite-env.d.ts`（新增） | 标准vite/client声明，支持本地PNG模块类型 |
| `src/assets/app-icon.png`（新增） | 用户图标的前端副本 |
| `src-tauri/icons/source.png`（新增） | 用户提供原图的只读来源副本，用于可重复生成；外部原图未改写 |
| `src-tauri/icons/icon.png`、`icon.ico` | 黄色背景蓝色指针平台资源，供默认窗口、EXE和托盘使用 |
| `scripts/create-icon.mjs` | 从本地source生成PNG/ICO/前端资源，不恢复旧紫C；输出剔除非图像必要PNG块 |
| `src-tauri/src/lib.rs` | 仅将默认客户区尺寸改800×520，最小尺寸改520×400 |
| `src/README.md`、`src-tauri/README.md`、`src-tauri/icons/README.md`、`scripts/README.md` | 更新直接受影响的布局与图标说明 |

没有新增依赖、路由、持久化字段、版本或安装器。前轮settings/main/Cargo/Tauri配置/capability/package与两个锁文件哈希均保持原样。对lib.rs只逆向恢复两行尺寸后，整文件SHA-256与前轮稳定快照完全一致，故托盘生命周期/配置/便携路径无其他生产变化。

用户图标为48×48 RGBA，源图及生成PNG检查均只有IHDR/IDAT/IEND块；ICO包裹同一PNG。脚本不访问原始外部路径或网络；持久说明不记录原图机器路径。实际各尺寸视觉清晰度由人工确认。

## 验收与命令

全部命令在仓库根，经既有统一入口运行，继承项目内TEMP/TMP/TMPDIR、npm/Cargo/Node缓存及编译输出环境；未安装新依赖：

```powershell
node scripts/create-icon.mjs
./scripts/npm.ps1 run typecheck
./scripts/npm.ps1 run build:desktop
```

首次类型检查报本地PNG模块无类型声明；补标准vite/client声明后再次类型检查通过。桌面入口自动运行前端生产构建，最终结果：

| 契约条目 | 结果 | 证据与边界 |
| --- | --- | --- |
| 1 页面与草稿 | STATIC PASS / HUMAN PENDING | target/input/execution/settings/about五页内部状态；mode、center、coordinateX/Y为根setup级ref，页面及鼠标/键盘区域切换销毁DOM不会销毁草稿；实际点击待人工 |
| 2 导航与键盘 | STATIC PASS / HUMAN PENDING | 按钮原生键盘、aria-label/title、aria-current、收起aria-expanded、图标及焦点样式；无新路由依赖。真实键盘/提示体验待人工 |
| 3 摘要底栏及小尺寸 | STATIC PASS / HUMAN PENDING | header/footer为不缩小flex区，内容独立滚动；窄宽底栏重排和侧栏滚动；默认800×520/最小520×400。真实DPI/窗口布局待人工 |
| 4 诚实状态与错误 | STATIC PASS / HUMAN PENDING | 无示例EXE/PID/模拟绑定/计数；刷新/选择目标/绑定/执行/热键/未实现按键仍禁用。初始化warning与保存notice位于所有页面外的常驻提示区，初始化warning不被保存结果清除；实际错误呈现待人工 |
| 5 IPC与生命周期 | STATIC PASS / HUMAN PENDING | 真实get_app_info/set_minimize_preference、保存成功后更新/失败回退；关于版本来源为Rust返回。后端除尺寸两行外与前轮稳定字节一致 |
| 6 检查与构建 | PASS | vue-tsc通过；Vite7.3.1生产构建16模块，JS77.41KB/CSS6.76KB；Windows x64 release桌面构建通过，2m57s，无bundle/installer |
| 7 交接及最终EXE | PASS | 本记录、36文件完整manifest、最终EXE路径及hash；未启动 |
| 8 独立审核 | NOT RUN（待Verifier） | 只读审查本轮差异及偏好/托盘邻近回归，可隔离类型/前端构建；不要求重复桌面构建 |
| 9 桌面人工验收 | HUMAN PENDING | 本轮严格未启动应用、浏览器或任何桌面自动化；未操作用户窗口及数据 |

前轮6项后端纯文件测试结果保留为前轮证据，本轮因后端仅尺寸变化未重跑，也不将其当作新增UI测试。没有为了导航写镜像实现或模板测试。前端/桌面构建通过不证明实际UI或托盘回归通过。

## 脱敏及落点

- 源码/配置/说明/锁文件常见机器路径和凭据模式扫描无命中；`git diff --check`通过。
- 最终EXE对当前仓库、斜杠形式仓库及用户目录的ASCII/UTF-16精确字节扫描均无命中；不是完整二进制元数据审计。
- 图标只有图像必要块，无附加PNG文字元数据。未保存截图、外部日志或完整环境。
- 编译缓存延用项目内固定位置；最终验收文件仅本轮 `build/verification/TASK-001-UI/coder/final/`。前端构建移除的旧 `.gitkeep` 已恢复，未清理任何历史产物。
- 未运行系统级文件监控，不能声称整个系统零外写；操作系统/共享Runtime/执行平台内部记录仍属已声明边界。

## 人工最短清单

由用户自行退出旧版本后打开本轮final入口，不由agent操作：

1. 默认目标页和约800×520窗口；展开/收起侧栏，五页、图标、提示文字可识别。
2. 输入页切右键、指定坐标填写X/Y，切其他页再返回；切键盘再返回鼠标，确认草稿保留；执行操作仍禁用。
3. 调整窗口至较小尺寸与常用DPI，确认内容可滚动，目标摘要和底栏不被遮挡；Tab及焦点可用。
4. 设置页任务栏偏好保存/重启、关于真实版本、错误提示；托盘最小化/还原/关闭退出按前轮约定回归。
5. 检查品牌、标题栏、任务栏、EXE及托盘均使用新图标；实际系统缓存显示问题单独记录。

本轮没有运行后台服务或开发服务器，没有新增输入能力，也未进行游戏、Windows11、发布或安装验证。
