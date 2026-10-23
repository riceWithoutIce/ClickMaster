# TapTapLoot 输入机制只读分析

日期：2026-10-22。用户授权检查当前运行的游戏，以确定替代输入方案。本文主体为当时的只读分析；后续用户另行批准有限 IPC POC，两轮传输完成，第二轮人工反馈无游戏响应，见文末。当前不接入产品。

## 范围与证据

- 对唯一正在运行的 `TapTapLoot.exe` 查询主模块版本、已加载模块名称、顶层和后代窗口；不输出真实 PID/HWND、窗口标题、安装路径或其他应用内容。
- 从该实例安装目录读取 PE 元数据和托管 IL。使用 .NET `PEReader`/`MetadataReader`，没有将游戏程序集加载到执行上下文，没有运行游戏方法、读取进程内存或附加调试器。
- 未发送窗口消息、系统输入或 IPC 数据，未创建管道服务器/连接客户端，未激活、移动、关闭或重启游戏。未修改游戏、ClickMaster 生产源码或配置，未下载、安装或启动浏览器。
- 诊断脚本：`build/verification/TAPTAPLOOT-READONLY/inspect.ps1`；脱敏证据：同目录 `result.json`。TEMP/TMP/TMPDIR 在运行诊断编译前设为仓库内 `build/tmp/`；脚本和报告均在项目内。没有系统级写入监控，不宣称系统零外写。
- 执行入口：仓库根目录 PowerShell，`& ./build/verification/TAPTAPLOOT-READONLY/inspect.ps1`。脚本只读取目标、将自己的结果写到诊断目录；不是输入测试。
- 当前主程序集 SHA-256：`BC5318AC557E4039ADCCAE8D79EAF66F6C0200D18FC043FD2AA632F6513C35B8`。结论限定于该安装版本。已读取程序集不等于证明对应组件当前正在运行；没有读取运行中程序集内存以比对磁盘映像。

## 确认结果

| 项目 | 直接证据 | 边界 |
| --- | --- | --- |
| Unity 版本 | 进程 EXE/UnityPlayer 版本为 `6000.2.8f1 (c9992ac36c34)` | 不是根据窗口类推测 |
| 脚本后端与架构 | 进程加载 `mono-2.0-bdwgc.dll`；存在 Managed 主程序集；原生 UnityPlayer PE 为 AMD64 | 托管 PE 的 I386 + ILOnly 不能据此称为 32 位游戏 |
| 当前游戏窗口 | 一个可见、非最小化、采样时非前台的 UnityWndClass，客户区 1920×1080；另有两个隐藏 IME 辅助窗口，未发现后代窗口 | 瞬时采样，不证明全程焦点/逻辑更新 |
| Unity 输入组件 | 主程序集引用 InputLegacyModule，存在 Input.GetKey/GetKeyDown 调用；未发现 `Unity.InputSystem.dll` | 辅助 UI 使用 Unity Input 不代表核心计数也使用它 |
| 系统状态输入 | GlobalKeyHook.Process 调用 user32.GetAsyncKeyState | 是实际方法体调用，不仅是导入声明 |
| 键鼠范围 | Awake 枚举 Window.Keys；枚举含 LeftMouseBtn=1、RightMouseBtn=2、Num2=50 | 不是运行中键表或具体输入效果观察 |
| 后台意图 | BongoCat.Fps.Awake 中存在 `Application.runInBackground = true` | 没有证明当前组件激活、运行值或其他暂停机制 |
| SDL 分支 | 已加载 SDL2；GlobalKeyHook 初始化/轮询 GameController，并受 controllerSupport 设置约束 | 不是已确认 SDL 用于键盘/鼠标计数，也不是手柄兼容通过 |
| 窗口处理 | 存在 UniWindowController、LibUniWinC 插件及窗口样式相关代码 | 未据此修改样式或认定其为输入失败根因 |

## 核心玩法输入路径

安装文件中的直接 IL 调用关系为：

```text
GlobalKeyHook.Awake
  -> Enum.GetValues(Window.Keys) -> BUTTONS
GlobalKeyHook.Update
  -> Process
     -> 对 BUTTONS 调用 GetAsyncKeyState
     -> 高位 0x8000 / 低位 1 与 WasPressed/IsDown 判定新输入
     -> 累加 _keysDown
  -> _keysDown > 0 时触发 OnKeyPressed(数量)
  -> Statistic.AddClick / AchievementManager.AddStatProgress

ClickCounter.OnEnable -> 订阅 OnKeyPressed
TickClock.OnEnable -> 订阅 OnKeyPressed
```

这为现有失败提供了具体解释：`WM_KEYDOWN/UP` 与鼠标按下/释放消息的队列投递，不会因为成功投递就改写 `GetAsyncKeyState` 所查询的系统按键状态。因此仅修正前后台焦点或继续修改相同窗口消息，不能解决已识别的这条核心计数路径。该结论与[此前人工消息失败](taptaploot-key-posted-without-response.md)一致，但不等于排除游戏的全部其他输入入口。

诊断自身未调用 GetAsyncKeyState；以上为游戏磁盘 IL 中的调用证据。没有用户输入采样、按键记录或运行时调用追踪。

## 新候选：已有联动 IPC

主程序集包含 `BongoCat.IPC.Ipc` 和 `BongoCat.IPC.StreamString`：

- `Start` 在非 Editor 条件下启动两个后台线程；是否有活跃场景组件触发 Start 尚未确认。
- `TypingFarmerThread` 作为本机命名管道**客户端**连接 `TapTapLootxTheFarmerWasReplaced`；连接超时参数 1000 ms，读取循环有 90 ms sleep。它读取字符串后 `Int32.Parse`，累加到 `_taps`。
- `Update` 在 `_taps > 0` 时调用 `GlobalKeyHook.Sim(_taps)`，再清零。`Sim` 直接触发 `OnKeyPressed(数量)`。
- `StreamString.ReadString` 使用两字节大端字节长度头，随后读取对应数量字节，用默认 `UnicodeEncoding` 解码（UTF-16LE）。这只是静态协议描述，没有发送任何数据；单次 Stream.Read 未处理完整读取的实现细节是未来 POC 边界，不能将静态协议视作已验证稳定接口。
- 另一客户端管道 `TapTapLootxBongoCat` 解析 JSON 后更新 buff，属于不同语义，不能用于本次点击测试。

这是当前版本中已有的联动代码，不是已经查证的公开 API 或作者维护承诺。没有探测其他进程的管道、占用上述名称或向运行游戏发数据。

重要差异：真实键鼠的 `Update` 路径另外调用 Statistic/Achievement；`Sim` 自身只触发事件。因此即使未来能驱动回合，也必须分别观察计数、统计和玩法效果，不能承诺与真实输入完全等价。

## 方案筛选

| 候选 | 针对本次发现的优势 | 当前决策 |
| --- | --- | --- |
| 保留 PostMessage | 保留已人工通过的计算器与其他消息型目标 | 保留基线；不为本游戏继续盲测同一路径 |
| SendInput / 全局键鼠 | 可能改变被轮询的系统输入状态 | 不满足其他窗口零额外输入约束；不作为后台定向方案 |
| 虚拟键鼠 / 驱动 | 可能让系统看到设备输入 | 同样不天然定向，且安装/签名/便携约束冲突；不优先 |
| 游戏已有 IPC 联动适配 | 潜在不碰焦点、不改变系统键鼠、不改游戏文件；无需新包/驱动 | 最值得对齐有限 POC；当前 defer，尚未启用或通过 |
| 游戏内插件 / 注入 | 可接触内部玩法事件 | 当前边界排除；已有 IPC 候选优先调查，不升级 |

## IPC 候选门槛与后续边界

| Gate | 状态 | 依据 |
| --- | --- | --- |
| 目标 Windows 支持 | pass（静态）/ unknown（运行效果） | 当前安装含 .NET 命名管道客户端；场景启用情况未证明 |
| 构建/分发路径 | pass（有限诊断可用已有 .NET） | 不下载依赖；正式 Rust 适配与游戏协议支持承诺尚未评审 |
| 产物与隔离 | pass（可规划） | POC 仅仓库内 verification/tmp；不写游戏目录；现有 build 忽略规则 |
| 安全及外部副作用 | unknown，需限界 | 管道为固定联动名称；必须拒绝已有同名端点/歧义，不抢占真实联动；核验连接客户端身份后才发送 |
| 可重复建立/移除 | unknown | 游戏是否主动重连、断连与退出行为未验证；测试拥有的管道可随 worker 退出释放 |
| 清晰集成边界 | pass（候选设计） | 显式游戏适配器，与通用键鼠模式分开；不能伪装成已发送某个按键 |
| 相对基线优势 | plausible / unknown | 直接通往玩法事件，理论上无需系统输入；没有运行成功证据 |

建议决策：**只读分析完成，IPC 适配暂缓集成，先由用户对齐有限 POC**。当前不安装、不改产品、不写管道、不执行游戏内 Sim，也不启动重复任务。

若批准下一步，最小 POC 应限定为：唯一目标实例、同名端点无冲突、建立一个诊断拥有的管道，确认连接方确为已选游戏实例，只发送一次数量 1；用户只需观察一次明确玩法变化。将“回合/事件响应、显示计数、统计、副作用”分开记录。身份不符、无连接、协议异常、无明确效果或超时立即停止，不重试、不切换全局输入。结束后释放自己的管道/进程，保持游戏文件与用户实例不变。后续具体时限、界面/自动取证和单次副作用应在批准前冻结。

## 后续有限 POC：玩法未响应

用户批准后，独立工具完成编译/纯测试及独立审核，并执行首轮单次数量1。用户未注意首轮效果，明确重新授权第二轮观察；两轮均为游戏客户端身份校验通过、单次写入完成、自有资源释放。第二轮用户反馈“游戏没有响应”，记录玩法兼容未通过，停止本轮POC，不自动重发或增大数量。

真实客户端成功连接证明所测联动端点存在活动客户端，但没有应用层确认，不能认定已完成解析或事件分发。消费链、具体订阅效果或并发行为仍无直接运行证据；不把失败笼统解释为IPC原理不可行。计数/统计及完整工作隔离没有单独确认，不补造结果。见[验收记录](../workflow/runs/TAPTAPLOOT-IPC-POC/acceptance.md)。

用户后来明确计数和战斗均未变化。[后续只读消费链复核](taptaploot-ipc-consumer-analysis.md)确认数量1足够驱动正常启用的计数器更新；战斗另有状态限制。日志无相关标记，但接收线程会吞部分异常。读取/解析/事件分发的具体失效点仍未确定，本次没有新输入。
