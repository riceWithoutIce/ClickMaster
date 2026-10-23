# TASK-004-CHANNEL 独立 Verifier 报告

日期：2026-10-22。对象：`TASK-004-CHANNEL-coder-final`，Coder 已终态冻结。结论：冻结需求的源码/策略/离线测试与测试包静态审核 PASS；无 BLOCKER，无需交用户裁定的新增 finding。允许交付本地测试包供用户验收，不表示真实游戏或正式发布通过。按约定停止本轮审核。

## 冻结与保留核对

- 54个实际源码/配置/锁文件的 SHA256 全部匹配冻结清单，核对前后无变化。清单 hash：`0AA24949D0915A816E9197BAD8F9D00AB1F44C2FBD6CDE8DE4CA8865A99C04C3`；未以 HEAD 代替 dirty 快照。
- `coder/final/ClickMaster.exe` 9,364,992字节，核对前后 SHA256 为 `AE4E289C0DB31D0C5C35DD41421800A1B4EC5DBF092EC79EF354703375E4FCD1`。未覆盖、重打包或启动普通入口，旁无 data。
- 独立重算 Coder 的188文件清单和主协调者的159文件子集清单，均无变更；两份清单数量口径不同，不是同一清单冲突。
- 结果见 `build/verification/TASK-004-CHANNEL/verifier/freeze-before.json`、`freeze-after.json`。

## 核心流程与边界

默认通道是会话内窗口消息；未新增配置键或跨重启通道恢复。IPC必须明确选择，目标失配保留所选IPC但拒绝发送，不回退。通道状态只读核验版本/实例，真实管道仅在显式单次、3秒准备与后台资格检查之后创建。消息参数保存在原Vue响应状态，IPC隐藏键鼠/坐标/路由控件；切回消息不重置本次草稿。

前端 inputLocked 与后端 execution_gate/require_idle 双重阻止忙碌时开始、切通道、改绑定/接收路由/坐标预览/偏好。IPC不依赖消息内容路由；请求及绑定epoch重新校验，实际接收对象显示原进程管道客户端。消息runtime只调度message任务，IPC不会被消息tick误处理。状态版本递增，连接、写入、读取独立显示，未确认值和取消/超时不冒称玩法未响应。

Rust worker沿用唯一具体实例/创建时间、保留进程句柄、EXE/已知程序集hash、唯一原Unity窗口与线程、可见非最小化/有效客户区、发送时后台资格及管道客户端PID校验。仅固定四字节数量1帧，无循环/重连。当前用户DACL、本地客户端、first-instance/max1边界成立。worker授权通过匿名stdin，绑定实际父进程实例/相同EXE、二进制hash与一次claim；未授权worker入口不会初始化UI。声明的最终包拒绝入口证据已读，未以普通启动补证。

同步读取确认隔离于自有worker，父监督使用Instant及相对worker时长，无墙钟日期转换；连接20秒、写2秒、读取10秒、总60秒，准备阶段可取消。停止/实际退出设置取消并等待监督清理；只对直接保留的Child调用终止与有限等待，无名称/PID/树杀进程。无法确认清理保持busy锁定而不误报正常完成。父进程退出的worker看门狗及单次总限保留。IPC在途帧不可撤回，读取不代表玩法响应。

生产数据从实际EXE位置推导data，不使用CWD或用户目录回退；worker临时环境继承该落点。仅只读目标身份通过stdin传递，不持久化真实PID/HWND/个人标题/秘密。设置未知字段、原配置失败保留及原消息配对释放路径经紧邻纯回归覆盖。未发现新增全局输入、焦点/光标操作、注入、驱动、游戏文件修改或外部终止路径。未扩展审核为重复、F8、其他适配器或OVERLAY-001。

## 独立实际运行证据

入口 `build/verification/TASK-004-CHANNEL/verifier/run-checks.mjs` 继承项目环境，设置离线Cargo、新建本任务verifier/tests测试根、任务内temp/Node缓存；按授权复用现有项目Cargo输出及缓存。所有报告和fixture在verifier目录。没有更新锁文件、下载依赖或执行Coder finalize。

| 检查 | 实际调用 | 结果 |
| --- | --- | --- |
| 类型 | `vue-tsc --noEmit`，使用既有本地入口 | 退出0，无诊断 |
| 前端生产helper | `node scripts/test-target-filter.mjs` | 退出0，含通道独立路由、失配/epoch/未知通道拒绝、读取措辞及原筛选/坐标/消息/接收路由回归 |
| 后端 | `cargo test --release --locked --offline --manifest-path src-tauri/Cargo.toml` | 52 passed / 0 failed / 1 ignored；ignored为由父测试显式启动的fixture入口 |
| 最终EXE图标 | `scripts/verify-pe-icon.mjs` 只读解析冻结EXE与项目ICO | 退出0，资源逐图像一致 |
| 包脱敏 | 最终EXE UTF8/UTF16已知项目/用户目录前缀扫描 | 0匹配；不宣称穷举全部敏感值或全系统零外写 |

后端实际独立非游戏fixture验证：随机测试管道独占/四字节单次传输、无客户端有限超时、授权秘密/父进程与一次claim、只结束自有Child而另一个独立Child存活、真实阻塞drain可由自有worker终止。fixture不查询真实游戏、不使用真实游戏管道名。源码纯测试覆盖协议、单调边界、默认/不回退/忙碌、实例名称/创建时间，以及消息/路由/配置/托盘邻接回归。日志和退出码见 `verifier/run-results.json`、各检查日志。

## 未执行与交付限制

未打开普通UI、查询实际游戏、创建游戏管道或发送。真实主程序IPC游戏响应、准备/连接/读取期间停止、实际退出/目标失效/托盘、消息通道计算器回归及完整桌面输入/鼠标/焦点隔离，按human-test.md留给用户。旧C#工具人工通过不能代替本Rust包证据。便携包沿用既有WebView2运行前提，无.NET sidecar；不是正式release。

独立非游戏fixture和纯策略不能证明真实游戏兼容或所有瞬时状态；时限仍受线程/系统调度和最多2秒清理等待影响。读取成功只证明传输缓冲被读走，不证明玩法、统计、成就等价。源码和声明构建证据已审核，无需为补充这些人工边界扩大自动运行范围。
