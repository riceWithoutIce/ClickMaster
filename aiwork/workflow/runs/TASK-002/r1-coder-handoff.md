# TASK-002 R1 Coder 交接

日期：2026-10-21。Coder：本轮唯一源码写入者。状态：**READY_FOR_REVIEW**。源码已冻结；真实后台目标及UI行为仍为 **HUMAN PENDING**。

## 稳定快照与入口

- 快照：`TASK-002-R1-CODER-S1`，基于TASK-002原稳定快照；既有未提交状态保留，未提交、推送、发布或清理历史。
- [42文件源码清单](r1-source-manifest.sha256)，清单SHA-256：`58a3757abd827e143338defebfe7203a9511b4da841efcb1f513cdc16ebcc726`。在原40文件基础上加入生产筛选辅助模块及其离线测试脚本。
- 最终入口：`build/verification/TASK-002/coder/r1/final/ClickMaster.exe`；9,063,936 bytes；SHA-256：`93e1fecb01e16fb2d36e5027d6d809251ae7a6cecefd23a095d08cf50e59680a`。
- final与本次`build/cargo/release/clickmaster.exe`一致，仅复制，**未启动**；final没有data。旧final及两个锁文件哈希核对保持不变；未覆盖旧产物、旧夹具或旧验收记录。
- 结构化摘要：`build/verification/TASK-002/coder/r1/evidence.json`；图标证据：同目录`pe-icon.json`。临时收口脚本`finalize.mjs`也留在本轮项目内目录，包含防覆盖、旧final/锁文件校验及二进制路径扫描。
- 交接后停止修改清单内文件；父级契约、人工记录、roadmap、devlog不在生产源码清单中。

## 实现

### 范围与显示

- `refresh_targets(includeHidden)`将范围传到查询适配层，默认前端false。默认可见且有标题，包含最小化主窗口；开启后允许隐藏、cloaked及无标题应用顶层窗口，仍只用EnumWindows，不把无窗口进程列为目标。
- 明确排除：自身PID、GetDesktopWindow/GetShellWindow句柄；类名`Progman`、`WorkerW`、`Shell_TrayWnd`、`Shell_SecondaryTrayWnd`；WS_EX_TOOLWINDOW、WS_EX_NOACTIVATE；没有WS_EX_APPWINDOW的owned辅助窗口。开启隐藏范围不解除这些辅助排除；它并不承诺列出系统所有后台进程/窗口。
- IsWindowVisible/IsIconic/Dwm cloaked合成`visible`、`minimized`、`hidden`；隐藏或cloaked优先标隐藏，其他最小化标最小化。空标题前端显示“无标题窗口”；必要身份不可读取项继续只返回排除数量，不提权。

### 身份保持

- 枚举候选规则和`inspect`分开；已绑定live identity不再依赖可见性、非空标题、候选列表成员或辅助样式。
- 隐藏、标题清空、关闭后台范围、刷新中不再列出该项，均保留可验证的原HWND/PID/进程创建FILETIME/窗口线程ID。周期检查更新标题/EXE/状态及revision。
- 仍在绑定前重验可信token对应身份。刷新（包括失败）使旧候选token失效；枚举本身失败不清除仍可单独验证的绑定。
- 真正窗口销毁、进程退出、身份变化或必要权限/身份无法验证仍解绑；不按同名EXE/PID跟随重启。原快速HWND复用边界未改变。

### 即时筛选与会话状态

- 输入EXE、标题或PID的部分内容即可匹配，忽略大小写及首尾空白；仅本地过滤已查询候选，显示匹配数/候选总数，区分空候选和零匹配。
- 范围变更会刷新；刷新保留筛选词及范围开关。根级会话状态保留切页草稿，不新增配置格式或持久化。
- 过滤掉的未绑定选择立即清空；绑定按钮基于过滤后可选项，因此不会继续提交被隐藏的旧选择。刷新重置候选选择，使用新token。有效后端绑定与本地筛选独立，摘要始终可见。
- 原托盘、关闭选择、图标、偏好IPC、导航及执行按钮禁用均保留；页面仍可滚动，底部控件保持可达。没有发送输入、热键或目标激活代码。

## 修改范围

| 范围 | 文件 |
| --- | --- |
| 查询、状态与接线 | `src-tauri/src/windows_query.rs`、`targets.rs`、`lib.rs` |
| 筛选、范围与状态UI | `src/App.vue`、`src/style.css`、新增`src/target-filter.ts` |
| 离线筛选测试入口 | 新增`scripts/test-target-filter.mjs`、`scripts/project.mjs`、`package.json`仅增加脚本 |
| 直接相关说明 | `README.md`、`src/README.md`、`src-tauri/src/README.md`、`scripts/README.md` |

无新增依赖、升级、版本修改、配置格式变化或资源替换。Cargo格式化仅涉及本轮Rust修改文件。

## 实际命令与结果

工作目录为仓库根。所有npm调用使用原项目包装入口，下游继承项目内TEMP/TMP/TMPDIR、npm/Cargo/Node缓存和编译输出；格式化通过`projectEnvironment()`启动Cargo，不修改用户级配置。

```powershell
./scripts/npm.ps1 run typecheck
./scripts/npm.ps1 run test:targets
$env:CLICKMASTER_TEST_ROOT = Join-Path (Get-Location) 'build/verification/TASK-002/coder/r1/tests'
./scripts/npm.ps1 run test:backend
./scripts/npm.ps1 run build
./scripts/npm.ps1 run build:desktop
node build/verification/TASK-002/coder/r1/finalize.mjs
git diff --check
```

| 检查 | 结果与证据边界 |
| --- | --- |
| 类型检查 | PASS |
| 实际生产筛选辅助函数离线测试 | PASS：标题/EXE/PID部分匹配、大小写/空白、空候选/零匹配及计数、被过滤候选不可选、展示标题及状态；使用现有TypeScript编译器，无浏览器/服务器 |
| 完整Rust release测试 | 21 passed / 0 failed；测试执行0.02s，编译链接1m22s；仅虚构窗口和注入查询，不调用真实EnumWindows/inspect |
| 生产前端 | PASS，18模块，JS86.65KB/CSS7.49KB |
| Windows x64 release桌面构建 | PASS，2m00s；不打包安装器，未执行应用 |
| PE图标 | PASS，group32512/lang1033引用RT_ICON1；1104 bytes，SHA-256 `ac160094426f536177ad7101ccbff0ede9fc5b1ee2a9684dcd88611f70756eb9`，与当前ICO匹配 |
| 脱敏及受管理落点 | PASS：本轮源码/说明常见机器路径/凭据扫描无命中，diff检查通过；最终EXE对本机仓库两种斜杠形式及用户目录UTF-8/UTF-16精确扫描无命中；非完整二进制元数据审计或系统级零外写证明 |

21项Rust测试保留原数据/偏好及自身窗口策略测试，并覆盖候选范围和状态、辅助排除、隐藏/空标题保持绑定、窄范围列表为空、扩大范围仍不出现、枚举失败但live identity可验证、过期token、身份变化和退出。前端筛选与计数为纯函数证据；真实Vue控件交互与视觉未执行，不能宣称UI通过。

配置及目录夹具明确使用本轮`build/verification/TASK-002/coder/r1/tests/`，不复用原S1夹具。Verifier必须显式指定自己的`build/verification/TASK-002/verifier/r1/`下新测试根，避免覆盖现有夹具；同时执行`test:targets`及约定检查。

## 验收状态与下一步

| 契约项目 | 当前结论 |
| --- | --- |
| 后台/无标题候选与状态 | LOGIC/STATIC PASS；真实目标HUMAN PENDING |
| 隐藏、空标题、范围切换、候选缺失保持同一绑定 | LOGIC/STATIC PASS；真实Windows变化HUMAN PENDING |
| 身份失效、过期token及重启保护 | LOGIC PASS；真实退出/重启HUMAN PENDING |
| 即时筛选、计数、零匹配及选择安全 | PURE LOGIC/STATIC PASS；实际控件/切页/焦点HUMAN PENDING |
| 既有关闭框/托盘/偏好/图标及布局保留 | STATIC/BUILD PASS；真实视觉、交互及托盘HUMAN PENDING |
| 本轮类型/生产构建/非UI测试/PE图标 | PASS |

没有启动应用EXE、浏览器、开发服务器、可控窗口或桌面自动化；没有读取、绑定或操作用户真实目标窗口。未记录真实标题、完整进程路径、原始环境或未脱敏日志；原测试夹具均保留。除了非UI测试二进制和构建工具，没有运行应用或目标。

下一步由协调者核对本快照后交独立Verifier，只验R1及紧邻身份回归；findings一次性返用户决定。后台进程若没有可枚举窗口，或窗口属于仍排除的辅助类型，不承诺出现；具体目标窗口属性与是否命中上述规则仍未知，须人工观察，不声称已经定位全部原因。后台绑定通过也不代表后台输入或游戏响应通过。本轮不提交、推送或发布。
