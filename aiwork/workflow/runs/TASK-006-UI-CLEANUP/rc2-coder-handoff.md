# rc.2 Coder稳定交接

日期：2026-10-22。状态：已实现、自测、构建和便携候选准备，源码冻结。子Coder实现后服务多次断流，协调者接续执行测试/构建/打包与本交接；尚未代替独立Verifier。无普通UI、真实游戏、真实F8或桌面输入，无提交推送/正式发布/历史清理。

## 修改

以rc.1的69文件dirty manifest为基线，本轮18文件变更，无新增/删除清单项，实际见rc2/coder/source-changes.json。IPC四模块将完整单帧写入作为发送成功，读取结果分Pending/Confirmed/Unconfirmed/Failed并附可选Win32码，只作诊断；worker仍仅写一次。父监督观察写入后最多200ms，清理直接持有Child才允许下一轮，无重叠/补发。读取失败不变为传输失败。

父监督已抽取生产supervise供受控fixture使用；观察到worker退出后重读最终report，避免之前读到旧状态；清理后再收集已完成写入，停止/到期/目标失效后仍精确计入已发送，保留原终止原因。清理未确认继续锁定。原固定协议、身份/版本/ACL/first-instance、连接/写入失败停止与热键/托盘/限时逻辑保留。

前端类型、诊断/结束口径与helper同步，次数为消息完整投递或IPC完整单帧写入，不等于玩法响应。版本递增0.1.0-rc.2，自有package/Cargo/锁/Tauri字段同步，无依赖升级。使用/限制/变更文档写明短时观察及慢接收端可能来不及消费，未声称解决全部rc.1根因。按钮改名/泛化诊断优化/覆盖层/高频优化未纳入。

## 命令与证据

仓库根：`CLICKMASTER_CHECK_RUN=r2-final`运行 `node build/verification/TASK-006-UI-CLEANUP/rc2/coder/check.mjs`；脚本使用scripts/environment.mjs，项目内缓存/临时/测试输出，Cargo离线locked。环境写法为说明，PowerShell实际使用进程级环境变量。命令包括cargo fmt、类型检查、全后端、前端helper、desktop构建。最终76 passed / 0 failed / 1 ignored，ignored为父测试显式调用的子入口。记录为*-r2-final.log；类型/helper/构建/图标均退出0。

关键受控fixture：及时及100ms延迟读取精确单帧且无额外帧；连续3轮不读均在短时观察后退出并可继续；读取API错误不阻断、真正写失败计0且停止；最终报告在初次poll后/退出前发布仍收集；写后停止/到期/目标失效计1，写前停止计0；自有Child清理不影响无关Child。管道名称随机且非真实游戏；受控supervise+生产send_frame/helper不是完整真实AppHandle/目标/UI集成。真实游戏/桌面隔离/热键/托盘/双平台/性能NOT RUN。

首轮backend.log失败：fixture错误预期客户端关闭必使Flush失败，Windows实测可成功；改为明确断开服务器管道来可靠触发原生API错误，不放松生产标准。backend-r2.log再次失败为复用旧测试目录的既有portable移动fixture报目录非空；改用全新tests-r2-final目录，无生产修复，最终全通过。原失败日志保留。

打包入口仅本轮 `rc2/coder/package.mjs`，不可由Verifier重跑。包含版本/图标/AMD64 PE、无授权worker拒绝、ZIP CRC/解压逐字节一致、已知本机前缀扫描和旧652文件hash保留检查。ZIP5文件无data/debug/cache，未签名；共享WebView2及原环境限制保留。生成extract副本只限本轮项目内，检查后删除，不动用户data或旧包。旧rc.1包及源清单保留，未晋级release。

## 冻结身份

- snapshot：`TASK-006-UI-CLEANUP-rc.2-coder-final`；HEAD仅参考，dirtySnapshot=true。
- 源清单：`build/verification/TASK-006-UI-CLEANUP/rc2/coder/source-manifest.json`；69文件，SHA256 `1E13433B08DD3509754B5772822B8F1FF8A22812D7986936F98F20B157B94032`。
- EXE：`build/verification/TASK-006-UI-CLEANUP/rc2/coder/final/ClickMaster.exe`；9457664字节；SHA256 `F1C4789F06AF420CCE2633A51B9BA49B795B48FE2F6B2B8144C9BE1A27FE5A38`。
- ZIP：`build/staging/TASK-006-UI-CLEANUP-rc2/ClickMaster-0.1.0-rc.2-windows-x64.zip`；2946202字节；SHA256 `6B3E4ACCF658CFF5EEDCCB21D4677A7CFB3BC088FD9B0255F58935777D73DAD4`。
- source.json/SHA256SUMS在新staging；manifest包含源码、配置、锁、构建入口和产品文档，不含本轮角色记录。独立Verifier检查前后这些被验文件/包不得改写。

尚待独立限定审核与用户rc.2实际玩法/界面体验。读取未确认不阻断是已实现语义，不代表所有帧可靠送达；当前不提供游戏业务回执。
