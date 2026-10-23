# TASK-010：WebView环境统计与状态对照

日期：2026-10-22。用户同意日志分析后的下一步：补齐可归属WebView统计，比较待命/连点与显示/收起，先定位再优化。现执行诊断补采的有界实现、项目内离线测试与rc.6本地候选；真实游戏操作仍归用户。技术核对和环境共享语义见[screening](screening.md)。

进度：Coder已提供冻结的73文件rc.6快照及完整[交接](coder-handoff.md)，[独立限定验收](verifier-report.md)完成，无新增findings。独立后端96通过、0失败、2隔离入口按设计忽略；日志、状态分组、类型/helper及候选检查通过。协调者收尾核对源码、manifest、EXE、ZIP及652项历史证据、15项旧候选、2份用户原日志全部一致。本轮开发与限定审核结束，rc.6交用户真实采集；实际COM/UI/游戏仍NOT RUN，性能问题尚未定位或宣称解决。

## 范围

- 仅从当前Tauri WebView的官方Environment8.GetProcessInfos取得关联集合；匿名分组CPU、内存、进程数量与类型、覆盖/缺失/变动/采样新鲜度。
- 字段明示范围是同数据目录WebView环境、可能共享、crashpad不含，不能称为EXE实例独占。无法核对归属或接口不可用则partial/unavailable，不全机按名字/命令行补猜。
- 沿用60秒、默认关闭、每秒采样/批量落盘及1MiB/5套5MiB；关闭时无新COM查询/枚举/采样线程。UI线程异步适配最多一个待完成请求，超时/过期数据标缺失，不阻塞采集线程/输入/退出。
- 记录资源和四种状态的对照汇总，状态来自本工具后端实际任务/窗口事件，不依赖用户手记。状态切换穿过采样区间时标mixed/排除严格分组，不用末端状态冒充整个区间。未知状态保留。游戏及窗口状态切换由用户操作，不自动安排任务或隐藏/激活窗口。
- 自动摘要列出每组有效时长、发送量、CPU分项、内存样本、覆盖限制；未覆盖的组标not-observed，不填0。不把共享环境/重复进程内存之和说成独占物理内存。

## 写入边界与排除

允许performance/perf_native新增WebView适配与统计、必要lib生命周期/UI诊断提示及类型、测试/帮助/自有版本。IPC模块仅测量点所需最小调整，不改变原传输、身份、授权、停止/退出/计数/读取语义。hotkey原V008-01未处置不修；不修覆盖层、不改UI布局、不做任务级worker/长连接/哈希缓存，不升级依赖/Runtime，不开新全局日志。

可显式依赖已锁定webview2-com=0.39.1和windows-core=0.62.2，锁文件只调整根直接依赖，不改其他包版本/来源。若需其他能力先返回协调者。

## 流程与交付

协调者→单Coder→稳定快照→独立只读Verifier→用户真实采集。Coder单写生产，协调者维护task/screening/acceptance/human-test/devlog/roadmap。沿用scripts/environment.mjs、Cargo offline/locked和项目内独立输出。保持dirty状态、rc.1～rc.5及原用户日志/data；不自动轮换用户提供日志做测试。

输出根build/verification/TASK-010-WEBVIEW-METRICS，rc.6候选build/staging/TASK-010-WEBVIEW-METRICS；快照/manifest/EXE/ZIP/hash及完整自测交接后冻结。无自动普通UI/真实COM环境/游戏/F8操作，无提交推送、正式发布、上传或历史清理。
