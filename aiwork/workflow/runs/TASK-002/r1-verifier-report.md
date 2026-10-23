# TASK-002 R1 独立限定复审

日期：2026-10-21。对象：`TASK-002-R1-CODER-S1`。范围仅限后台/隐藏候选、展示与身份分离、即时筛选及紧邻绑定回归。

## 结论与快照

**限定源码审查、独立类型检查、生产筛选辅助函数测试及前端构建通过，未发现 BLOCKER 或 FOLLOW-UP findings。真实后台目标及 UI 仍 HUMAN PENDING。** 目标恢复正常显示后可绑定，不能作为后台场景通过证据；本轮也未读取其真实窗口属性。

- [42 文件清单](r1-source-manifest.sha256) SHA-256：`58A3757ABD827E143338DEFEBFE7203A9511B4DA841EFCB1F513CDC16EBCC726`。
- 开始和结束原始 42 文件均 0 mismatch；结束时隔离副本亦 42 文件 0 mismatch。生产源码、资源、锁文件、旧交接与夹具未改写。
- 隔离根：`build/verification/TASK-002/verifier/r1/source/`，复制清单文件及现有前端依赖，不安装依赖或复制 Rust 缓存。
- R1 最终入口：`build/verification/TASK-002/coder/r1/final/ClickMaster.exe`；SHA-256 `93E1FECB01E16FB2D36E5027D6D809251AE7A6CECEFD23A095D08CF50E59680A`，与交接一致，核验时同级无 `data/`。
- 旧 TASK-002 final SHA-256 仍为 `5BD948D95B9272EE1CCA3F13F234F8B1CBE7612D29758AADDCC7728A1D57A751`，未覆盖。

## 实际独立检查

以下命令均在隔离根使用副本包装入口执行，临时目录、缓存及前端输出均在该副本内，退出码均为 0：

```powershell
./scripts/npm.ps1 run typecheck
./scripts/npm.ps1 run test:targets
./scripts/npm.ps1 run build
```

- 筛选测试实际导入生产 `target-filter.ts` 编译结果，覆盖标题/EXE/PID部分匹配、大小写/首尾空白、空候选/零匹配、计数、被过滤候选不可选、原数组不变及无标题/状态标签。
- 前端构建 Vite 7.3.1、18 模块，约 657 ms，JS 86.65 KB、CSS 7.49 KB。未启动浏览器或开发服务器。
- [Coder 交接](r1-coder-handoff.md) 及本轮结构化证据记录 21 项 Rust 测试通过、完整 Windows x64 桌面构建和 PE 图标核验通过。本次核对其测试断言与虚构查询边界，但没有独立重跑 Rust 测试或完整桌面构建，不把 Coder 结果写成 Verifier 实跑结果。

## 原问题与紧邻回归

| 项目 | 独立结果与依据 | 人工边界 |
| --- | --- | --- |
| 后台范围与无标题 | STATIC PASS：默认关闭；开启时允许隐藏、cloaked、无标题顶层应用窗口；仍排除自身、desktop/shell、明确系统类、tool/noactivate及非 appwindow 的 owned 辅助窗口；无窗口进程不构造候选 | 真实目标是否有符合规则的 HWND 未知，NOT RUN |
| 状态与占位 | STATIC / PURE TEST PASS：隐藏/cloaked优先为 hidden，否则最小化为 minimized；前端显示可见/最小化/隐藏，空标题为“无标题窗口” | 真实状态与实际显示 NOT RUN |
| 展示与身份分离 | STATIC PASS：candidate 仅供枚举；inspect 不以可见性、标题或辅助样式判断 live identity。仍检查窗口、PID、创建 FILETIME、线程及必要进程信息 | 实际隐藏、标题清空及权限变化 NOT RUN |
| 隐藏/范围关闭/列表缺失保持绑定 | STATIC / CODER TEST EVIDENCE PASS：refresh 先独立 validate；候选集与 bound 分离。测试覆盖隐藏空标题、窄范围为空、扩大范围仍被列表省略、枚举失败但身份可验证时保持 bound | 实际刷新和后台检查时序 NOT RUN |
| 真正失效与重启保护 | STATIC / CODER TEST EVIDENCE PASS：可信 token 查表、刷新旧 token 过期、绑定前全身份比较保持；真实销毁/退出/必要身份不可验证仍清除；不按名称或旧 PID 跟随 | 实际退出/重启 NOT RUN |
| 筛选与选择安全 | PURE TEST / STATIC PASS：本地 computed 部分匹配，忽略大小写/首尾空白；匹配数与总数分别显示；无候选和零匹配分开。selected 基于过滤后列表，同步 watcher 清除失选 token；绑定前再检查 selected，不能提交过滤掉的旧选择 | Vue 控件输入、焦点及切页体验 NOT RUN |
| 刷新/范围/会话保留 | STATIC PASS：刷新仅重置候选与选中 token，不清 filterText/includeHidden；范围切换带 includeHidden 刷新；筛选、范围、草稿位于根 setup，不新增持久化；过滤本身不发送绑定命令、不清后端 bound | 实际交互 NOT RUN |
| 紧邻既有功能 | STATIC PASS：lib.rs 生产变化仅 refresh_targets 范围参数接线；关闭选择、托盘、偏好 IPC、数据目录、图标资源和依赖/锁文件不变。页内容继续独立滚动，底部执行仍禁用 | 视觉/托盘回归不在本次自动验证范围 |

## 脱敏、限制与停止

- 42 文件文本机器路径/常见敏感模式扫描无命中；报告只留相对路径及必要摘要，不保存真实窗口标题、完整进程路径、截图、原始环境或日志。最终二进制敏感路径扫描及 PE 核验沿用 Coder 证据，未伪称本次重新执行。
- 未执行真实 EnumWindows/目标进程查询，未启动应用、浏览器、服务器或可控窗口，未使用桌面自动化或清理系统缓存，未操作外部进程。没有运行完整后端测试，未写任何旧文件夹具。
- 没有系统级外写监控；受管理副本与输出在项目内，不宣称整个系统零写入。没有新依赖/版本/配置格式变化。

本轮未发现 findings，不向 Coder 追加修复。限定复审到此结束；真实 TapTapLoot 后台窗口属性、后台候选出现、保持绑定及 UI 交互由用户对 R1 最终产物验收。任务管理器的进程分类、纯逻辑测试或已恢复窗口后的截图均不能替代后台场景证据；后台绑定通过也不代表输入或游戏响应通过。
