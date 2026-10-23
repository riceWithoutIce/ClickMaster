# TASK-005-REPEAT 独立 Verifier 报告

日期：2026-10-22。对象为冻结快照 `TASK-005-REPEAT-coder-final`。结论：冻结需求的源码、调度/停止路径、离线测试和本地包核对 PASS，无 BLOCKER；允许交付包供用户进行真实桌面验收。本报告不替代真实 UI、F8、托盘、游戏或输入隔离验收，按约定结束本轮审核。

## 冻结与保留核对

- 57 个源码/配置/锁文件全部匹配 `coder/source-manifest.json`；manifest SHA256 为 `D1CA33BC0A2AB47F2C0FCBBF4390E29F79B7CEA1FB66BF4E1EDC3A5B7005753D`，审核前后无变化。
- `coder/final/ClickMaster.exe` 为 9,423,360 字节，审核前后 SHA256 均为 `97A7B2FC9F844E6F7BA64824FE7DCCED7209EFDB6647A89B0DF3C21655E8676D`；final 目录无 data。
- Coder 的 216 文件旧证据基线与协调者的 172 文件保留基线均重新计算且变更数为 0。结果见 `build/verification/TASK-005-REPEAT/verifier/freeze-before.json` 与 `freeze-after.json`。

## 冻结流程核对

- `repeat::Request` 对窗口消息 100–60000、IPC 2000–60000 做有限整数校验，拒绝空值、非整数、越界、最小大于最大和溢出；固定区间不调用随机源。拒绝采样对闭区间均匀取样，最大128次拒绝后停止。
- `input::Engine` 在3秒准备后首轮立即开始；按下/释放完整成功后，以实际完成时刻建立下一次单调 due，等待闭区间区间值，晚到不追赶。准备、按下后配对、等待和身份/投递失败均停止；不重叠、不批量、不重连。完成计数用 checked_add 和安全上限，持续任务没有60秒会话上限。
- IPC 每次传输仍为数量1固定帧，并将每次 60 秒预算用于该次连接/写/drain；重复阶段复用一个有界随机槽位，前一个自有 Child 已确认退出后才清理/renew，不按帧增加目录或日志。无法确认清理时保持锁定，旧证据目录不被扫描或删除。
- F8 由同一拥有线程注册、PeekMessage 分发和注销；注册失败不会启动任务，成功后才 arm。按钮、托盘停止、F8、实际退出共用取消路径，热键服务异常时后台任务主动停止；终态和异常 finally 释放注册。
- 主界面运行期间锁定目标、通道和间隔。托盘停止项由实际 busy 状态启用，tooltip 区分待命/单次/重复/停止中及完成数；收起保持运行，退出先取消并等待自有 worker。IPC worker 仅通过同 EXE、匿名 stdin、当前父进程实例/创建时间、二进制与一次 claim 授权；父端只保留并终止直接创建的 Child，不按名称/PID/进程树终止。
- 既有消息按下/释放的原身份复核、IPC 目标版本/实例/窗口/后台资格、客户端 PID、current-user ACL、first-instance/max1、固定单帧及原托盘/配置行为保留。未发现 SendInput、焦点抢占、光标移动、注入、驱动、游戏文件写入或外部进程终止路径。

## 独立实际执行

入口 `build/verification/TASK-005-REPEAT/verifier/run-checks.mjs` 使用项目环境和离线 Cargo，设置本任务 verifier/tests、temp 和 Node 缓存；未更新锁文件、下载依赖、启动普通 UI、注册真实 F8、查询游戏、连接真实游戏管道或发送桌面输入。

| 检查 | 结果 |
| --- | --- |
| `vue-tsc --noEmit` | 退出 0 |
| `scripts/test-target-filter.mjs` | 退出 0，含随机/固定间隔、边界、空值/小数/顺序/溢出、通道下限和等待状态 helper 回归 |
| `cargo test --release --locked --offline` | 61 passed / 0 failed / 1 ignored；包含随机采样、实际完成后等待、持续超过60秒、时钟安全上限、消息状态机、F8 fake 注册/冲突/dispatch/释放、托盘/停止紧邻测试、IPC 非游戏 fixture、阻塞 drain 与自有 Child 清理、单槽位及旧数据保护 |
| 冻结 EXE PE icon | 退出 0，资源与项目 ICO 一致 |

Coder 的未授权 `--ipc-worker` 两组入口检查均退出64、无输出、无 data 创建；Coder 脱敏扫描为0已知本机前缀。独立日志和结果保存在 verifier 目录；本报告不复制原始路径、PID、HWND、令牌或窗口标题。

## 未验证边界

真实 UI 渲染、参数控件实际锁定、F8 冲突/送达/注销、托盘继续与停止、实际退出/目标退出、随机时间的桌面观感、真实 IPC/窗口消息游戏效果、精确游戏计数以及工作窗口输入/鼠标/焦点隔离均留给 `human-test.md`。非游戏 fixture 不能替代这些证据；完成次数是投递/读取次数，不是游戏统计。60 秒是每次 IPC 传输监督预算，持续重复的长期稳定仍未验证；系统调度及最多2秒清理等待会影响实际结束时刻。以上未执行项不被纯测试宣称通过。
