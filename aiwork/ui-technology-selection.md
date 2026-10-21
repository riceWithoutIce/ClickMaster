# 技术选型补充：圆角界面与 UI 动效

日期：2026-10-21。

状态：用户已表达优先采用 Tauri 的选择，前提是可行。官方架构与本机基础环境检查支持将 Tauri 2 + Rust + TypeScript 作为第一版首选技术路线。尚未完成项目构建或后台输入 POC，未安装依赖或创建项目。

## 用户补充与选型调整

用户明确希望考虑视觉风格，例如圆角界面和 UI 动效。因此，上一轮仅偏重简单表单与小体积、优先比较 WinForms / Win32 的顺序需要调整。

“轻量”与“现代界面”共同参与决策。具体圆角、动效、配色、布局与性能预算尚未冻结；提到圆角不等于承诺 Windows 10/11 上完全一致的窗口外框效果。

## Tauri、Rust、TypeScript 的职责

它们可以组合使用，并非三个互斥框架：

| 部分 | 候选职责 |
| --- | --- |
| Tauri 2 | 桌面窗口、WebView、前后端通信、构建与分发 |
| Rust | 枚举与校验目标实例、Windows API、重复调度、停止热键和状态管理 |
| TypeScript + HTML/CSS | 目标列表、输入配置、圆角控件、视觉动效与运行状态展示 |
| React / Vue / Svelte 等 | 可选的前端组件组织方式；本轮未选定，也不要求必须加入 |

官方架构和进程模型说明确认，Tauri 使用 Rust 核心与 Web UI；Windows 使用 Microsoft Edge WebView2，不把一套 WebView 库随最终 EXE 打包。Rust 和 Node.js 等开发工具不需要按同样方式安装到最终用户机器，但 WebView2 运行时是用户侧依赖。

## 与本项目输入功能的集成建议

以下是候选设计，不是已经实现的行为：

- UI 提交绑定与配置，Rust 持有经校验的目标实例及窗口；前端传入的句柄或参数不能直接视为有效绑定。
- 重复调度与完整按下/释放在 Rust 后端执行，不依赖网页 `setInterval`，避免将业务节奏绑在 UI 渲染或隐藏时的计时器行为上。
- 全局停止、目标退出与失效处理放在后端；UI 负责呈现真实后端状态。
- 控制命令与状态同步通过 Tauri 通信，不为每次点击往返前端。
- 后端调用 PostMessage，不因使用 Rust/Tauri 而增加全局 SendInput 回退或其他输入方式。
- 使用本地静态界面资源，按实际功能限定前后端命令与权限；不需要为当前功能引入远程页面、账号或在线服务。

使用 Rust 不代表全部代码自动内存安全；Windows FFI 与状态、句柄生命周期仍需正确处理。更换 UI 框架不会提高游戏对窗口消息的接受程度。

## 圆角与动画的适配

- 控件圆角、卡片、配色、悬停反馈、淡入淡出和状态切换，适合使用 CSS 实现。
- 整个顶层窗口的外轮廓、阴影、自定义标题栏、拖动与缩放属于桌面窗口行为，需要按支持的 Windows 版本单独验证，不能只加 CSS 圆角就宣称完成。
- Tauri 官方提供自定义标题栏、无装饰窗口和窗口管理 API；自绘标题栏后，应保留可用的拖动、关闭、最小化及约定的缩放行为。
- 动效应服务于交互反馈；停止命令立即提交给后端，不能等待退出或切换动画结束。

## 候选比较

此表为工程评估，尚无本项目体积、内存或流畅度实测。

| 候选 | 圆角与动效实现 | 用户侧依赖与开销 | 开发维护取舍 | 本轮定位 |
| --- | --- | --- | --- | --- |
| Tauri 2 + Rust + TS | Web 样式与动画灵活，便于定制视觉 | Windows 依赖 WebView2，有 WebView 多进程开销 | Rust、前端及通信两侧均需维护 | 用户已确认的首选方向 |
| C# + WPF | 控件模板、XAML 与内置动画系统适合定制界面 | 依赖所选 .NET 发布策略；不要求 WebView | Windows 桌面方向集中，需管理样式与动画 | 保留比较记录 |
| C# + WinForms | 可定制，但较强视觉需求常增加自绘或组件工作 | 依赖所选 .NET 发布策略 | 简单表单成本低，复杂动效优势较弱 | 保留，优先级降低 |
| C++ + Win32 | 能实现，自绘与交互细节工作较多 | 可控制依赖与体积；具体链接方式待定 | 相同视觉目标的实现维护成本较高 | 极小体积优先时再重点考虑 |
| 维持现有文档基线 | 无可运行 UI | 无新增依赖 | 不产生开发成本，也不能交付工具 | 比较基线 |

WPF 官方文档明确提供动画与时序系统、属性动画、样式和控件模板中的动画使用方式，故不能因为需要现代视觉就排除 WPF。

## Tauri 分发与轻量边界

- 官方提供 NSIS `-setup.exe` 与 WiX `.msi` 构建路径。
- 默认安装策略在 WebView2 缺失时下载并运行 bootstrapper；安装可能需要网络，不能承诺完全离线部署。
- 若目标已有适配的 WebView2，可复用运行时；不能把“常见 Windows 已有”当成每台目标机器必有。
- 嵌入离线 WebView2 安装器或固定运行时会明显增大包体。官方页面当前示例分别约增加 127MB / 180MB，仅为官方说明中的估算，随版本变化，非本项目实测。
- 应分别记录应用包、额外运行时安装、安装后数据、启动时间，以及归属于本应用的核心和 WebView 进程开销；不能只看主 EXE 或主进程。
- WebView 用户数据、缓存、配置和日志的实际位置与清理方案仍待明确；不预先承诺免安装、无缓存或单文件即运行。

## Tauri 初筛门槛

| 门槛 | 状态与证据 |
| --- | --- |
| 目标环境 | unknown：官方支持 Windows / WebView2，项目最低 Windows 版本及目标机器运行时尚未验证。 |
| 分发与构建路径 | pass（官方路径）：Rust/MSVC、前端构建和 NSIS/MSI 有官方说明；本机可构建性 unknown。 |
| 生成数据边界 | defer：需明确 Cargo、前端依赖、构建产物及 WebView 用户数据/缓存目录。 |
| 忽略与隔离 | defer：项目尚未创建，后续显式排除 node_modules、target、dist 等生成物；运行数据单独规划。 |
| 安全与副作用 | defer：本地静态资源、有限命令与后台输入边界是设计约束；WebView2 安装/更新行为需纳入分发策略。 |
| 可重复设置与移除 | unknown：尚未执行工具链、构建、安装或卸载验证。 |
| 集成边界 | pass（设计）：Rust 承担 Windows 输入，TS 展示界面，可保持独立项目；实际行为待验证。 |
| 基线收益 | pass（选型判断）：能覆盖用户提出的视觉定制与动效方向；具体资源成本待测。 |

首轮仅检查过 PATH 入口。用户表达 Tauri 优先意向后，追加只读环境核验，结果如下；这些属于存在性和版本证据，不是本项目构建通过证据。

| 项目 | 当前只读检查结果 |
| --- | --- |
| Windows | DisplayVersion 22H2，Build 19045.6466。 |
| WebView2 Runtime | EdgeUpdate 注册信息显示 155.0.4283.62；未通过本项目实际创建 WebView 验证。 |
| Rust | 已安装且激活 stable-x86_64-pc-windows-msvc；直接执行已安装工具链返回 rustc 1.96.1、cargo 1.96.1。 |
| Microsoft C++ 工具 | vswhere 找到带 x86/x64 C++ Tools 的 Visual Studio Community 2026；MSVC 14.51.36231 的 x64 编译器与链接器文件存在。 |
| Windows SDK | 检测到 Include 目录版本 10.0.26100.0；未执行链接验证。 |
| 前端开发工具 | node --version 返回 v24.14.1，npm.cmd --version 返回 11.13.0。 |

检查未安装或更新任何工具，未执行项目初始化、依赖还原、编译或打包。前表中的“本机可构建性 unknown”仍保持，需由后续实际构建证据关闭。先前 PATH 未找到 cl 不代表未安装，本次已通过 Visual Studio 安装信息找到相关文件。

## 本轮结论

决定：**用户偏好确认为 Tauri 2 + Rust + TypeScript 首选路线；实现与 POC 仍未开始。**

架构上可覆盖当前 UI 与 Windows 后端职责，本机已检测到主要基础工具和运行时。尚未证明具体依赖组合可构建，也未证明目标游戏响应后台消息。WPF 保留为比较记录，不作为默认并行开发方案。

下一步围绕 Tauri 明确最低支持 Windows 版本、WebView2 分发方式、前端组件框架和轻量预算，并落实数据目录与生成物排除约定，再进入获授权的最小验证。用户本次选择技术方向，不视为安装依赖或开始实现的指令。

## 本轮核对来源

- [Tauri 官方架构](https://v2.tauri.app/concept/architecture/)
- [Tauri 官方进程模型](https://v2.tauri.app/concept/process-model/)
- [Tauri 开发前提](https://v2.tauri.app/start/prerequisites/)
- [Tauri Windows 安装器与 WebView2 策略](https://v2.tauri.app/distribute/windows-installer/)
- [Tauri 窗口定制](https://v2.tauri.app/learn/window-customization/)
- [Microsoft WPF 动画概述](https://learn.microsoft.com/en-us/dotnet/desktop/wpf/graphics-multimedia/animation-overview)

以上通过 Chrome 实际查阅，本次打开的页面已关闭。
