# 独立 IPC EXE Coder 稳定交接

日期：2026-10-22。新源码和交付物冻结于 `build/verification/TAPTAPLOOT-IPC-EXE/coder/`。没有修改产品、旧工具或旧验收证据；未启动普通UI或游戏实测。请独立 Verifier 完成只读审核后，由主协调者交用户自行双击/点击测试。

## 最终包

`coder/final/` 共5文件，目录总计约246KB，包含：

- `ClickMaster.IpcTest.exe`，便携应用入口；项目图标已写入原生PE图标资源。
- `ClickMaster.IpcTest.dll`、`ClickMaster.IpcTest.deps.json`、`ClickMaster.IpcTest.runtimeconfig.json`，必须与EXE保留同目录。
- `使用说明.txt`，中文操作、证据及依赖说明。

framework-dependent Windows x64，要求 .NET Desktop Runtime **10.0.8或兼容更新的10.0补丁版本**（本机具备）。不是单文件/自包含包，不自动安装或下载。最终包没有data、PDB、测试fixture或原始票据，不晋级release。

## 用户流程与实现

普通启动先确认实际EXE同级data可写并设置其temp；无法写则停止，无其他目录回退。启动窗口保持停止，不查游戏、不注册F8、不创建worker/管道。中文窗口提供单次、低频、停止/F8、准备倒计时、写/读进度和单次“有响应/无响应/没看清”。使用 WinForms 仅作本次独立测试UI，不改正式Tauri技术选型。

用户明确点击单次才创建一个新会话，data/sessions内最多10个会话，达到上限只提示，不自动删除。一个会话含single及最多一次repeat；阶段attempt用CreateNew、worker.claim另一次原子声明，阶段失败不自动重试。新单次重置当前反馈；重新打开不恢复绑定/有效确认。UI与共用InteractionState共同禁止重入。单次只有读取成功、正常结束且新明确“有响应”才解锁本会话低频，仍须用户再点。反馈文件绑定本次single最终attempt/worker/supervisor文件hash与run，不改写原回执。

复制适配原REPEAT C#协议、身份、窗口、ACL、逐帧调度及阶段门槛；没有重写协议。worker使用**同EXE内部入口**，只接受EXE位置推导的data会话GUID与single/repeat，环境一次票据、当前session/stage/run、EXE与DLL hash核验后才进入核心，普通启动不会变worker。仅Hash身份落盘；不落真实PID/HWND/路径。F8仅本阶段在UI线程注册，WndProc处理WM_HOTKEY，冲突不创建worker；结束finally注销。UI await后台监督，native drain只在隔离worker，不阻塞UI。

single数量1一次；repeat逐帧数量1最多5，上一帧读取成功后至少2秒，不追赶、不批量数量5。每帧前重验原客户端、保留查询句柄对应的原实例/窗口、程序集hash及后台可见非最小化条件。first-instance/max1/current-user DACL/reject-remote原管道保护保留。准备5秒、连接最多20秒、write2秒、drain10秒、父总监督60秒，worker自身非阻塞阶段令牌58秒。逐帧write-attempted/complete/drain阶段原子替换，父读取允许Delete共享；UTC直接按明确偏移DateTimeOffset处理，保留7位小数后映射单调时钟。

停止/F8/关闭请求取消，后台监督只Kill(false)直接创建并持有的自有Process引用，最多2秒WaitForExit，不按名字/PID结束外部进程，不杀树；正常Dispose、OS回收、未确认分别记录。在途帧不能撤回。若未确认worker或F8释放，UI禁用新开始并保留停止/关闭重试入口，关闭不会默默留下后台发送。没有键鼠模拟、hook、抢焦点、注入、驱动、进程内存读取或游戏/产品文件修改。

## 构建

命令（从仓库根目录）：

```powershell
& ./build/verification/TAPTAPLOOT-IPC-EXE/coder/build.ps1 `
  -WorkDirectory ./build/verification/TAPTAPLOOT-IPC-EXE/coder/work-r1
```

使用既有 .NET SDK 10.0.300、Desktop 10.0.8 ref/runtime/apphost；NuGet.Config清空所有源，没有新依赖或下载。脚本将CLI home、NuGet packages/HTTP/plugins、TEMP/TMP/TMPDIR、obj/bin全部指向本任务；关闭遥测/首次体验/workload广告和解析。禁PDB、PathMap源码路径。首轮脚本参数拼接错误在 `work/` 保留，修正后 `work-r1/build-result.json` 与脱敏 `build-log.txt` 为离线restore/publish成功。没有删改旧产物。

Verifier可用新的本任务内WorkDirectory与OutputDirectory独立构建，脚本拒绝覆盖已有发布目录；不运行普通UI。

## 纯自检与证据

执行最终包EXE的 `--self-test <本任务内全新证据目录>`，由直接持有Process对象等待退出；本轮输出 `coder/self-test/`，退出码0。该入口在任何UI初始化前路由，要求程序位于本项目任务树，产物只在指定任务目录，不允许把发布后的用户文件夹当任意自检落点。未调用真实UI/F8、目标查询、worker、管道或发送。

`self-test/result.json`：共 **94项检查PASS**（51核心+43桌面/路径/时间/授权）：

- 原共用调度器/协议/门槛/停止/10秒与60秒策略、无票据拒绝。
- 实际DataStore会话路径不依赖CWD、10上限不删除、阶段CreateNew拒绝覆盖、GUID路径校验；InteractionState当前反馈/新单次/重启/重入/一次repeat门槛。
- 实际WorkerGate的session、票据、EXE/DLL hash门槛；实际反馈落盘→single文件门槛，no/unclear不解锁、yes解锁。
- 实际Policy.Save→DataStore.ReadWorker→SupervisionClock链，历史UTC、7位小数、明确偏移、104ms保真、9.999/10秒、原子完成、逐帧重置。

`icon-verification.json`：调用仓库既有纯PE资源检查器，不执行被检程序；EXE图标与项目icon完全匹配，1个图像。窗口图标从同一ICO嵌入资源加载，实际窗口显示仍由用户验收。

`package-path-scan.json`：对全部最终文件UTF8/UTF16扫描本仓库/用户目录已知本机路径，0匹配；无PDB。`build.ps1` AST解析0错误。`preserved-baseline.json` / `preservation-check.json`：183个旧IPC工具/证据和产品/工程文件hash前后不变。最终包运行纯自检后仍无data。

## SHA256冻结

16个源码/配置/图标/说明文件逐项hash见 `coder/source-sha256.json`；该清单SHA256：`A6024CFF05F001DD291ACE8F98D3A46EAB7BC0E7E0733835BFE3B09AFF8716E1`。最终包逐项清单见 `coder/package-sha256.json`：

| 最终文件 | SHA256 |
| --- | --- |
| `ClickMaster.IpcTest.exe` | `67A7BFDD5109B506EAECC16850D0899DC14C90E792C372748D9DCC5F4CBAEA5B` |
| `ClickMaster.IpcTest.dll` | `C536BAC52EFC721E5551D6D075E112F5A43B9E433106D24AD21073DAC191BF65` |
| `ClickMaster.IpcTest.deps.json` | `CF607FC6865FF2C46220B46F62857EF3F2556F106E94E6A79CE9E7A68D157319` |
| `ClickMaster.IpcTest.runtimeconfig.json` | `95AEB50687D5060FD21CB16B2D2417F6DB63BFA1C14E3821A7297E0203C8F0EE` |
| `使用说明.txt` | `301E490EB2F8C4B61151F3AE7CE3F20BC9D69DDAA5B92FF018BFB43C98EA67A7` |

## 未执行项与交付边界

**没有自动启动正常UI、游戏或任何真实测试。** 仅离线构建与EXE无GUI纯自检。真实窗口渲染/缩放、F8注册冲突/送达/注销、停止关闭时真实worker结束、原生连接/写/drain、当前游戏效果均由用户验收，不能用纯策略测试替代。采样无法证明所有瞬时状态或完整桌面输入隔离；10/60秒受OS调度/最多2秒退出等待影响，不承诺硬实时。读取成功不等同玩法成功、5次也不等同长期稳定。

等待独立Verifier稳定快照审核；问题一次性返回用户决定。主协调者只交付可点击EXE路径，不自动打开或恢复脚本实测；本任务没有提交、推送、正式发布或旧内容清理。
