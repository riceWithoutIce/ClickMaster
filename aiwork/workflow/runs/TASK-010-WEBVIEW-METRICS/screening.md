# WebView资源补采技术核对

日期：2026-10-22。使用technology-selection流程；仅核对公开官方文档和本地锁定依赖源码，未启动UI/游戏或新采集。

## 问题与基线

rc.5真实60秒日志主程序+worker平均整机CPU约0.376%，WebView未测，无法归因用户完整软件高占用。用户同意补齐可归属WebView资源，并比较待命/连点与显示/收起。目标是诊断完整度，不是本轮降低资源或提高发送频率。

## 候选

| 候选 | 成熟度/支持/分发 | 数据与隔离 | 边界/维护/优势 | 决策 |
| --- | --- | --- | --- | --- |
| 保持rc.5 unavailable | 已用、无改动 | 当前规则不变 | 无新维护，但无法补齐遗漏 | 保留为失败降级，不作为主方案 |
| Win32按名称或父子树枚举 | Windows可用、既有依赖 | 需要全机进程枚举，名称/父PID不足以证明归属 | 可能将其他软件或PID复用计入，也可能漏共享进程 | reject作为归属来源 |
| Tauri with_webview + Environment8.GetProcessInfos | 官方Windows接口，既有锁定绑定可用 | 仅查询当前WebView关联环境，资源从持有句柄读取；不记录身份 | 能分组CPU/内存，须标为同数据目录环境而非当前实例独占 | approve for bounded integration |
| ETW/外部性能工具 | 可分析更多CPU热点 | 新安装/追踪范围及系统外写成本 | 超出当前便携诊断边界 | defer |

## 官方证据与语义

- [Environment8](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2environment8)，本轮HTTP200核对：GetProcessInfos返回同user data folder全部进程，排除crashpad；接口自WebView2 Win32 1.0.1108.44提供。
- [WebView2进程模型](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/process-model)，本轮HTTP200核对：一个user data folder关联一个browser进程及其集合；不同应用/环境可共享同folder，renderer也可服务多个WebView。
- 本地tauri2.12.4 PlatformWebview.environment/controller、WebviewWindow.with_webview已有wry API，无需unstable；[锁定版本文档](https://docs.rs/tauri/2.12.4/tauri/webview/struct.PlatformWebview.html)HTTP200。
- 锁定并缓存的webview2-com0.39.1含Environment8/GetProcessInfos/ProcessInfo.Kind/ProcessId，windows-core0.62.2提供Interface.cast。允许显式依赖已有这两个确切版本以访问接口，不增加版本、下载或升级。

## 硬门槛

| 门槛 | 状态 | 证据与约束 |
| --- | --- | --- |
| 目标环境支持 | pass | Windows目标+已有COM绑定，接口cast失败或Runtime不支持则unavailable，不自动安装 |
| 官方分发/构建 | pass | Tauri已有接口与锁定缓存绑定，offline构建；仅提升既有传递依赖为直接依赖 |
| 数据/缓存落点 | pass | 延用项目内开发输出和EXE同级data/logs预算 |
| 忽略/隔离 | pass | 不查询全机同名集合；只由当前WebView环境API提供候选，未知/过期/缺失不补猜 |
| 安全/隐私/网络/凭据 | pass | 只读进程查询句柄，不终止WebView，不读路径/命令行/内容，无上传；共享可能性显式说明 |
| 可重复验证/移除 | pass | adapter/helper与隔离进程fixture，真实COM人测单列；关闭采集无新增查询，删除新增adapter可退回unavailable |
| 集成边界 | pass | 仅诊断模块、UI线程API适配、资源采样与报告说明，不变IPC |
| 相对优势 | pass | 补齐当前缺失的关联环境数据，并提供状态分组；不承诺独占或全组零漏采 |

“仅归属于当前EXE实例全部资源”仍无法从GetProcessInfos证明，不能作为产品承诺。采用明确的webview-environment范围、sharedPossible=true及coverage标记；总和只能标为“已观察主程序+worker+关联环境”，不是无条件的EXE独占占用。

## 有界验证与拒绝条件

用户同意诊断补采作为本轮集成授权；早期门槛通过后进入单Coder实现→稳定快照→独立只读Verifier，真实环境由用户运行。允许显式固定既有绑定、不新增工具；新rc.6仅本地候选。全部生成内容在本任务build目录，无清理旧日志/证据。

关键验证：无全机名称归属、当前环境快照刷新/过期/接口不支持/进程退出/身份复用可识别；保留句柄避免漏短命退出CPU；记录CPU缺失，不用0冒充；查询需UI线程且最多一个未完成请求，采集线程/停止不等待UI响应；共享/漏采/工作集相加非独占等限制充分。不能可靠给出数据时退为partial/unavailable，不降低归属要求。出现需要升级Runtime/新依赖/修改IPC/自动游戏场景的阻塞时停止对应步骤并返回讨论。
