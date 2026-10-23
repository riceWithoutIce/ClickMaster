# TASK-003 Coder 交接：单次后台输入

日期：2026-10-21。Coder为本轮唯一源码写入者。状态：**READY_FOR_REVIEW**，源码已冻结。基线为46文件OVERLAY POC稳定源码；本轮只实现用户批准的单次后台输入，不实现重复、热键或修复OVERLAY-001。

## 稳定快照与入口

- 快照`TASK-003-CODER-S1`；[50文件源码清单](source-manifest.sha256)继承46文件并纳入4个新增生产模块，SHA-256：`3bcc7e67a1282933f6610a204b26a588f1d5d104cb67979aa88557a9016a73ee`。
- 最终入口`build/verification/TASK-003/coder/final/ClickMaster.exe`；9,222,144 bytes；SHA-256：`3bcf9f5f12ed69dd208e9be35445c0ce3640da1fdda1a9635853153b7f33e8b9`。
- 与本次`build/cargo/release/clickmaster.exe`一致，独占复制、未启动，final无data。旧TASK-002/R1/R2/OVERLAY final及锁文件均核对哈希未变。
- 结构化摘要及PE证据为`build/verification/TASK-003/coder/evidence.json`、`pe-icon.json`；项目内临时收口脚本为同目录`finalize.mjs`，含旧产物/未改模块检查与防覆盖。
- 交接后停止修改源码清单内文件；父级契约/人工记录等不属于生产清单，核对后可准入限定Verifier。

## 行为

- 单次测试捕获后端可信绑定identity+epoch和配置快照；固定3秒倒计时，正常按下/释放间隔约50ms。计时在Rust，不依赖前端页面。10ms节拍检查单一在内存任务；重复请求拒绝，不并发、不排队、不自动重试。
- 左右键按下/释放使用正确WM和按钮wParam，释放复用按下时已校验坐标。按下前读取当前客户区：中心随尺寸计算，自定义非负整数且客户区内，同时不超过有符号16位上界32767；无效不截断或取模。
- 键盘默认空格，下拉42项：A–Z、0–9主键区、空格/回车/四方向。后端严格字符串白名单；根据目标窗口线程的键盘布局映射扫描码，保留E0 extended，repeat为1，释放设置previous/transition位，不读取或改变设备键盘状态。
- 只由原生适配层PostMessageW投递，不要求前台/可见，不主动激活、显示、移动目标或真实鼠标；隐藏目标仍以必要身份/坐标可验证为条件。按下前拒绝最小化、客户区不可读取/非正尺寸；无最小化兼容承诺。客户区在临时PMv2线程上下文读取并恢复，与overlay完全独立。
- 每阶段保存内存Receipt：按下/释放是否成功投递、失败原因及Win32错误码；保留当前/最近任务id、脱敏模式摘要、phase、剩余时间。区分countdown/sending/delivered/cancelled/rejected/failed/partial，绝不把按下成功而释放失败报成完整成功。成功文案明确“已投递，目标是否响应请人工确认”。

## 取消、绑定转换及退出

- 按下前及释放前重新验证原identity和当前epoch，准备坐标/扫描码后、实际按下前再验证。原HWND/PID/创建时间/线程或epoch变化时不能跟随替代实例；同一目标重绑也取消。
- 统一锁序input→targets；worker的阶段验证/投递在同一目标锁域内执行。请求不能传入原始目标句柄、PID、VK或消息来决定投递。
- 取消倒计时直接结束，无输入。已投递按下时取消会缩短剩余约50ms等待，在原identity+epoch仍可验证时立即尝试一次配对释放；已入队消息不可撤回。原身份不可验证则不向替代窗口补发，明确partial；不重试释放。
- 换绑/解绑先持input和targets完成取消/配对；随后释放targets再请求overlay关闭，仍持input锁防新增任务，最后重新取targets完成转换。既不会先换目标再释放旧任务，也不持targets等待overlay线程确认；原生overlay及枚举源码未改。
- 实际退出（关闭选择的“退出”或托盘退出）先标记stopping、取消/配对，再停止并join输入worker；取消无需等待完整3秒或完整50ms，仅正常查询/非阻塞PostMessage及约一个worker节拍。worker观察stopping也会取消；新增任务在退出时拒绝。没有持续长按或额外调度线程。
- 收起/切页保留任务计时；关闭选择框的取消/收起仍按原语义，实际选择退出才走退出清理。

## 前端与保留能力

启用键盘下拉、单次测试和“取消单次”。倒计时/发送期间禁用再次测试及模式/按键/坐标草稿；可导航，相关状态通过Rust事件和轮询恢复，单调revision防旧状态覆盖。结果在常驻底部显示，不把单次取消称为全局热键；重复“开始”及参数/全局热键继续禁用。

原导航、绑定、托盘、关闭选择、偏好、便携数据、版本及图标保留。OVERLAY-001延期，实验预览默认关闭，界面说明已知未显示问题；单次输入不调用预览绘制或由其计算坐标。

新增`input.rs`纯构造/状态机、`input_native.rs`原生适配、`input_runtime.rs`worker及`src/single-input.ts`下拉/结果辅助；修改必要`lib.rs`生命周期/IPC、App/样式、原辅助测试和直接README。只为已有windows-sys补KeyboardAndMouse feature，没有新包/升级/版本或配置格式变化。两个锁文件、原overlay模块、targets、枚举、窗口策略及图标保持基线哈希。

## 实际命令与证据

工作目录为仓库根，经统一包装入口设置项目内TEMP/TMP/TMPDIR、npm/Cargo/Node缓存和编译路径；Cargo格式化也使用`projectEnvironment()`。无用户级配置或工具更新。

```powershell
./scripts/npm.ps1 run typecheck
./scripts/npm.ps1 run test:targets
$env:CLICKMASTER_TEST_ROOT = Join-Path (Get-Location) 'build/verification/TASK-003/coder/tests-final'
./scripts/npm.ps1 run test:backend
./scripts/npm.ps1 run build:desktop
node build/verification/TASK-003/coder/finalize.mjs
git diff --check
```

- 类型检查、筛选/overlay坐标转换及单次下拉/倒计时/partial错误码辅助测试PASS；42项前端选项无重复且默认空格。
- 完整Rust release最终34 passed/0 failed，执行0.01s、编译链接1m23s。原27项保留；新增7项行为测试覆盖两种鼠标/键盘消息、白名单/scan/extended/up位、坐标边界/16位范围、3秒截止与50ms释放、禁止排队、快照不跟草稿、取消/退出配对、同目标重绑/新实例、阶段身份再验证、拒绝/按下失败/释放partial及错误码。
- 测试只使用人工时间数值、fake binding/query/sender；不调用真实Win32查询或投递。首轮32项PASS后补阶段/快照回归，使用新的`tests-final/`目录34项通过；初次`coder/tests/`保留，旧夹具未覆盖。最终桌面构建同时包含绑定转换释放targets后关闭overlay的接线。
- 生产前端PASS，20模块、JS91.04KB/CSS7.81KB；Windows x64 release构建PASS，2m02s，不打安装器。自动PE图标PASS：group32512/lang1033引用RT_ICON1，1104 bytes，SHA-256 `ac160094426f536177ad7101ccbff0ede9fc5b1ee2a9684dcd88611f70756eb9`，与当前ICO匹配。
- 变更源码/说明及交接常见本机路径/凭据模式扫描无命中，`git diff --check`通过。final对实际仓库两种斜杠形式及用户目录UTF-8/UTF-16精确扫描无命中；非完整二进制元数据审计或整个系统零外写证明。前端槽位`.gitkeep`已恢复，未清理历史。

| 验证项 | 结论 |
| --- | --- |
| 消息内容/白名单/坐标/计时/取消/部分失败 | PURE/FAKE LOGIC PASS |
| 可信捕获、阶段验证、绑定转换及退出锁序 | STATIC PASS；相应fake状态机PASS |
| 类型/前端辅助/34项Rust/生产前端/桌面/PE图标 | PASS |
| 真实PostMessage/目标接收、后台游戏/工作窗口隔离、鼠标焦点 | HUMAN PENDING，未执行 |
| 真实UI倒计时、托盘收起、关闭/退出/换绑行为 | HUMAN PENDING，未执行 |
| OVERLAY-001 | 保持延期，本轮未改原生overlay模块 |

## 操作与人工边界

未启动应用、GUI夹具、浏览器、开发服务器、桌面自动化或目标诊断；**没有真实PostMessage/SendInput投递，也没有真实窗口枚举/查询/目标操作**。构建包含原生API不等于已经调用验证。所有缓存/中间/验收产物在项目内，旧final/data、测试根、报告、dirty状态保留。

实际Win32消息接收/扫描码在当前目标布局中的表现、后台游戏响应、其他工作窗口是否受影响、鼠标/焦点、真实关闭/换绑/错误展示仍为HUMAN PENDING。后台消息失败不回退SendInput、提权、注入或其他路线。原快速HWND同进程同线程复用限制保持，不声称零竞态或玩法成功。

父级维护契约、人工记录、requirements/roadmap/devlog；Coder未改这些文件。无提交、推送、发布、下载工具或历史清理。独立Verifier在父级核对稳定快照后，只复审本轮发送/绑定/退出与紧邻前端回归；使用自身新输出和相称纯测试，findings一次性回用户决定，不直接追加修复。
