# IPC-EXE 独立 Verifier 审核

日期：2026-10-22。结论：本次独立测试包的冻结源码/交付静态审核与最终 EXE 纯自检 PASS，无 BLOCKER，无需交付前处置的 finding。允许主协调者提供包路径，由用户自行启动和点击验收；本报告不构成正常 UI、真实热键、游戏或正式产品验收。

## 核心流程审核

普通无参数入口只初始化实际 EXE 同级 data 与中文窗口；没有目标查询、F8 注册、管道或发送调用。窗口构造保持停止；单次/低频入口必须由用户事件触发。长时间查询、监督、等待和原生 drain 不放 UI 线程；UI await 期间保留停止、WM_HOTKEY 与关闭处理。

单次点击创建新 GUID 会话，原子阶段 attempt；状态机及 working 禁重入。只有当前单次完整读取、正常关闭、监督确认退出且未 kill、热键成功释放后，才允许当前人工反馈；新明确 yes 通过落盘的 run/三文件 hash/时间门槛解锁低频，仍需用户再次点击。新单次或进程重启创建的新状态不恢复旧反馈；repeat 每会话最多一次，不能跟随目标重启/换窗口。

F8 绑定窗口句柄，UI 线程注册、WndProc 处理、finally 释放；冲突不启动 worker。停止/F8/关闭请求取消，监督只通过直接持有且引用校验的 Process 对象 Kill(false)，不杀树、不按名称/PID扫描终止。UI 关闭先等当前操作，再检查自有 worker 结束与热键释放；未确认时拒绝退出并保留停止/关闭重试入口。普通完成、OS 回收与未确认分别留证，在途帧不宣称可撤回。

worker 为同 EXE 的受控参数入口，要求环境票据、当前 GUID 会话/阶段/run、EXE/DLL hash 与一次 claim；普通启动不会进入该路径。数据由实际 EXE 位置推导，不依赖 CWD；data 不可写时停止，无用户目录回退。最多 10 会话并提示自行处理，不自动删除。父子 TEMP/TMP/TMPDIR 放 data 内。

继承的单帧协议、同客户端/原目标实例与窗口复核、精确 EXE/已知程序集 hash、唯一 UnityWndClass 后台可见非最小化/有效客户区、first-instance/max1/current-user ACL/reject-remote 均保留。single 一帧，repeat 至多五帧且前帧读取后至少两秒，不追赶；连接20秒、写2秒、逐帧 drain10秒、父总监督60秒。原子逐帧回执、保留 UTC 七位小数的单调监督及新帧 timer 重置符合契约。未发现全局键鼠、PostMessage、焦点/鼠标移动、进程内存、注入/驱动/调试器或游戏/产品写入路径。

## 独立执行与冻结证据

仅执行最终包 `ClickMaster.IpcTest.exe --self-test <本任务内新目录>`，ProcessStartInfo 隐藏启动并直接持有自检进程，有界等待正常退出，退出码0。使用独立 `verifier/self-test/`，94项 PASS：51核心与43桌面/路径/授权/时间检查。自检入口在任何 UI 初始化前路由，源码与结果均显示没有原生调用、目标查询或 worker 启动。未启动正常 UI、F8、管道或发送。

审核前后16源码/配置/图标/说明和5个包文件全部匹配冻结 manifest；183个旧工具/证据和产品文件重新计算 hash 均未变。`verifier/checks.json` 保存核对结果。最终包仍只有5文件，无 data/PDB；独立对各文件 UTF8/UTF16 扫描已知仓库/用户路径，0匹配。build.ps1 AST错误0。

核对 csproj、清空包源的 NuGet.Config、构建脚本和已声明离线成功证据：framework-dependent net10.0-windows/win-x64、禁PDB、源码 PathMap、缓存/obj/bin/CLI home 在任务内。没有为补证重新构建或下载依赖。运行配置需要 Microsoft.NETCore.App 与 Microsoft.WindowsDesktop.App 10.0.8（或兼容更新10.0补丁）；中文说明明确 .NET Desktop10 x64 依赖、必要同目录文件及非自包含边界。EXE图标资源匹配项目icon的已有PE检查证据与当前包hash绑定，窗口使用同一嵌入ICO；实际显示仍待人工。

## 交付边界

主协调者只交付可点击路径，不自动启动正常UI或恢复脚本实测。UI渲染/缩放、真实F8冲突/送达/注销、真实停止/关闭的worker结束、原生写/drain、游戏效果、精确次数和全程桌面隔离留给用户验收。纯测试不替代上述证据；读取成功不证明玩法响应，五帧不证明长期稳定。10/60秒受调度和最多2秒退出等待影响，仍是名义阈值。
