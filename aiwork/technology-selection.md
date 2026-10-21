# 技术选型：参考产品第一轮调查

日期：2026-10-21。

状态：公开资料与源码调查已完成；框架定案延后。未下载、安装、运行参考软件，未实现 POC，未验证游戏兼容性。

后续补充：用户提出圆角界面和 UI 动效要求后，候选优先顺序已调整，见 [UI 技术选型补充](ui-technology-selection.md)。本文件保留第一轮参考产品调查，不作为当前候选排序定案。

## 目标与基线

用户新增明确要求：本工具应为轻量级 Windows 软件。功能边界继续以 `requirements.md` 为准，核心为指定窗口后台消息输入，用户可同时操作其他软件。

当前基线只有需求与工作文档。维持现状不引入任何依赖，但不能完成重复输入。此次从用户指定的 MouseClickTool GitHub 仓库、发布页和鼠大侠官方网站及教程调查，不把第三方下载站的猜测作为技术结论。

轻量化应分别衡量发布文件大小、安装后体积、运行时前提、启动时间、空闲/运行 CPU、内存及常驻进程。具体数值预算尚未冻结，不能用 EXE 大小替代所有指标。

## MouseClickTool：可从源码确认的路线

### 框架与运行时

- `Directory.Build.props` 的 `TargetFrameworks` 为 `net462`，即 .NET Framework 4.6.2；`OutputType` 为 `WinExe`、`PlatformTarget` 为 `AnyCPU`。
- 核心源码引入 `System.Windows.Forms`，主类继承 `Form`，使用 WinForms 控件和 `System.Drawing` 绘制界面。
- 当前检查到的核心是 **C# + WinForms + .NET Framework + Windows API P/Invoke**。
- 发布说明虽然提过 WPF，但最新发布说明把迁移计划划去并注明搁置；不能据提交文字认定已迁移到 WPF。
- 小 EXE 仍依赖目标系统可用的 .NET Framework，并非把全部运行时打进几十 KB。

### 输入与调度

- 通过 `user32.dll` 的 `SendInput` 注入全局鼠标输入；核心源码第 1075 行声明 API，第 1097 行调用它。
- 普通连点分支在第 825、831 行分别发送按下与释放。
- 脚本坐标分支在第 706—710 行使用屏幕绝对坐标和 `MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE`，会发送鼠标移动输入。
- 使用 `RegisterHotKey`、`WM_HOTKEY` 处理键盘热键；可选鼠标中键触发通过低级鼠标钩子实现。
- 重复调度使用后台 `Task.Run` 与 `Task.Delay`，取消信号由 `TaskCompletionSource` 承载。
- 检查到的完整核心文件未见 `PostMessage` 或 `EnumWindows`，普通输入调用也没有接收目标 HWND。因此不能作为满足本项目后台隔离需求的现成输入实现。

### 发布体积与数据位置

- 仓库介绍写有“20KB、在线版 minimal 5KB”，仅记录为页面宣传数字；本次未下载测量发布文件，不能当成当前构建实测。
- `App/App.cs` 从嵌入资源解压 GZip 程序集，再通过 `Assembly.Load` 创建主窗体。
- 当前 `AppMini/AppMini.cs` 与工程配置把核心 DLL 嵌入资源后内存加载，不含该路径上的在线 DLL 下载。不能把旧的“在线版 5KB”描述当成当前分支的依赖模型。
- 最新可见 Release 标签为 `3.3.0.0`，对应 `f2cc79b`；当前分支相关文件还显示后续修改。发布二进制与当前分支是否一致未验证。
- 配置写入“我的文档”中的 `MouseClickTool_V3.3.ini`；脚本日志为工作目录相对路径 `MouseClickTool.LOG`。这些是参考软件行为，不是本项目已选定的数据位置。
- 当前检查范围未确认明确的源码复用许可证，故仅作设计参考，不引入或复制实现。

结论：可参考其简单 WinForms 界面、热键和异步等待思路；全局 SendInput 输入路线不符合冻结需求，不能直接沿用。压缩程序集是体积优化手段，不能证明较低的运行内存或更好的维护性。

## 鼠大侠：功能路线可见，内部技术栈未知

- 官网展示基础鼠标连点、轨迹与键盘录制回放、可视化宏、循环/判断/延时，属于功能较完整的桌面自动化产品。
- 官方使用教程明确列出默认屏幕定位、窗口定位和后台模式，并声称后台模式不影响前台操作。
- 上述属于**厂商功能声明**，未通过安装运行确认，也没有证明它对 TapTapLoot.exe 或其他具体游戏有效。
- 官网提供 Windows EXE 安装包，页面可见版本 `2.9.0.2`；教程称安装包约 17MB。本次没有下载或核验实际体积，安装后体积、内存、CPU 与依赖均未知。
- 本次查阅的官网、教程与 FAQ 未提供可核对的客户端源码、开发语言、UI 框架或具体输入 API，不能认定为 C++、Qt、WinForms、WPF 或其他框架。
- 2015 年 FAQ 使用“利用鼠标驱动编辑的宏”“驱动程序的运行”表述，但未给出驱动名称、安装说明、签名或实现证据；不足以证明现版本安装自有内核驱动。
- 2025 年问题汇总介绍键盘按下/松开命令，并提醒必须配对；不能据此推断其后台模式支持相同命令。

结论：适合作为配置与交互设计参考，不能从现有公开资料复原其底层技术路线，也不能借它为本项目的后台兼容性背书。

## 产品与候选路线比较

下表后两项为本项目候选设计，属于工程判断，尚无本地构建或性能实测。

| 路线 | 作用与成熟度 | 分发/运行前提 | 数据与隔离 | 后台需求适配 | 维护与基线收益 |
| --- | --- | --- | --- | --- | --- |
| 维持文档基线 | 无新增技术 | 无构建和安装 | 无新增运行数据 | 无法自动执行 | 成本最低，但不解决需求 |
| 直接使用或改造 MouseClickTool | 源码公开，有发布记录 | Framework 依赖；仓库提供 EXE 与工程 | 配置写文档目录、日志相对目录；需重设计 | 当前全局输入路线不合要求 | UI 可参考，改造还涉及绑定、输入、状态与许可确认 |
| 直接采用鼠大侠 | 已有产品，含宏/录制 | 官方安装包；内部依赖未知 | 配置、网络、缓存及卸载残留未核验 | 官方声称后台模式，实际目标兼容性未知 | 可参考交互；不能获得自主可维护实现 |
| 自建 C# WinForms + Windows API | 成熟 Windows UI，研发成本相对低 | 可选系统 Framework 依赖或现代 .NET 发布；版本和支持周期待选 | 可显式设计配置/日志目录与产物排除 | 可调用 PostMessage，游戏是否响应仍未知 | 适合少量表单控件；框架与输入层可分离 |
| 自建 C++ Win32 + Windows API | 成熟原生 Windows 路线 | 编译器、Windows SDK、C/C++ 运行库链接方式待确认 | 可显式设计配置/日志目录与产物排除 | 同样可调用 PostMessage，兼容性不因换语言而提高 | 对发布体积与运行依赖更有控制；UI、资源及生命周期实现成本较高 |

本轮候选池包含用户指定的两款参考产品，以及两个适合小型 Windows 工具的自建方向。没有扩大到 Electron、WebView 或跨平台方案，因为当前没有 Web 界面或跨平台需求支撑新增运行依赖。

## 选型门槛

`pass` 仅表示该项有当前证据，不代表整体可采用。自建候选没有进行构建验证；尚不进入 POC。

| 门槛 | MouseClickTool 直接沿用 | 鼠大侠直接采用 | 自建 WinForms / Win32 |
| --- | --- | --- | --- |
| 实际目标环境 | unknown：Windows 路线明确，目标游戏未测 | unknown：厂商支持说明，目标未测 | unknown：Windows 适配合理，目标未测 |
| 官方分发或支持的构建路径 | pass：官方仓库工程与 Release；本地构建未测 | pass：官网提供安装包；未安装 | unknown：开发工具链及发布方式未确认 |
| 生成数据范围可控 | unknown：配置/日志位置已知，是否采用该数据策略待定 | unknown：未核验配置/缓存/日志 | defer：设计时确定目录与清理边界 |
| 忽略与隔离行为 | unknown：仓库有 gitignore，本次未审阅内容 | unknown：安装及残留边界未核验 | defer：建项目时显式排除生成物 |
| 安全、网络、凭据和其他副作用 | fail：全局输入干扰隔离要求；脚本可启动外部程序 | unknown：官网功能说明不足以证明输入实现与副作用 | defer：限定本地 Windows API，不加入全局输入回退 |
| 可重复设置、验证、移除 | unknown：未构建、运行或清理验证 | unknown：未安装/卸载验证 | defer：选定工具链后明确命令 |
| 集成边界 | fail：输入路线与冻结需求冲突 | unknown：客户端实现不公开 | pass：设计上保持独立项目，UI 与输入层分离；实现待验 |
| 相对当前基线有价值 | pass：提供轻量 UI 参考，不能直接完成目标 | pass：提供产品交互参考，核心兼容性仍未知 | pass：具备覆盖冻结功能的合理路径，需后续证明 |

补充：当前机器注册表可读到 Framework v4 Full 的 `Version=4.8.09037`、`Release=533325`；本轮命令环境未找到 `dotnet`、`msbuild` 或 `cl`。前者不证明开发工具链齐备，后者也不能证明系统完全没有安装相关工具。未做工具链安装或全盘搜索。

## 本轮决定与下一步

决定：**defer——暂缓框架与打包定案，保留两个自建候选。**

1. 已确认轻量化是选型要求；具体预算需要覆盖发布体积与运行开销，不能照抄参考产品标题数字。
2. C# WinForms 是优先比较候选，C++ Win32 是偏向原生依赖与体积控制的对照候选；当前不声明任何一方已获得实现批准。
3. 下一轮核对支持的 Windows 版本、是否允许依赖系统 .NET、开发构建路径、维护成本与轻量预算，再形成明确建议。
4. 后台输入维持 PostMessage 方向；UI 框架选型不会消除目标游戏忽略消息的风险。后续最小验证仍按 `validation.md` 执行。
5. 本轮不安装竞品、不引入源码、不生成运行脚手架、不改变冻结功能范围。

## 来源

以下为本轮通过 Chrome 实际查看的公开页面；查看后已关闭本次打开的页面。

- [MouseClickTool 仓库与 README](https://github.com/lalakii/MouseClickTool)
- [公共构建属性](https://github.com/lalakii/MouseClickTool/blob/master/Directory.Build.props)
- [核心 WinForms 与输入代码](https://github.com/lalakii/MouseClickTool/blob/master/MouseClickTool/MouseClickTool.cs)
- [压缩资源启动器](https://github.com/lalakii/MouseClickTool/blob/master/App/App.cs)
- [启动器工程](https://github.com/lalakii/MouseClickTool/blob/master/App/App.csproj)
- [minimal 启动器](https://github.com/lalakii/MouseClickTool/blob/master/AppMini/AppMini.cs)
- [minimal 工程与嵌入依赖](https://github.com/lalakii/MouseClickTool/blob/master/AppMini/AppMini.csproj)
- [MouseClickTool Releases](https://github.com/lalakii/MouseClickTool/releases)
- [鼠大侠官网](https://www.shudaxia.com/)
- [鼠大侠使用教程，含窗口/后台模式与约 17MB 说明](https://www.shudaxia.com/doc/how-to-use.html)
- [鼠大侠“对鼠标有影响吗”旧 FAQ](https://www.shudaxia.com/faq/shi-yong-shudaxia-dui-shu-biao-you-ying-xiang-ma.html)
- [鼠大侠“设置了没反应”旧 FAQ](https://www.shudaxia.com/faq/shudaxia-she-zhi-liao-mei-fan-ying.html)
- [鼠大侠最新问题汇总，含键盘按下/松开说明](https://www.shudaxia.com/faq/shu-biao-lian-dian-qi-zui-xin-wen-ti-hui-zong.html)
