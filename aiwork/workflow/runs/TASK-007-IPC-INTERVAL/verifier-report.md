# TASK-007独立限定验收报告

日期：2026-10-22。Verifier：task007_verifier。结论：本轮冻结范围 **PASS**，未发现BLOCKER或FOLLOW-UP；可以交用户体验rc.3。真实游戏100ms、实际频率和运行环境验收仍NOT RUN，本结论不等于正式发布。

## 验收对象

快照 `TASK-007-IPC-INTERVAL-rc.3-coder-final`；69文件manifest SHA256 `3ED1B7AA576DAC85E49FA4141FE4955C7BD84245416785E71DE56E65929C0FB7`。EXE SHA256 `71C5B6806F590460DCA8271D96000681BB9294B2EAA2BBC645C0688918E36AC3`；ZIP SHA256 `AC1AF361AD80323326B0C6A590610758DA48D7CD0DDFE36A1F5DAEEC2642A5B8`。审核前后源清单/全部被验文件及EXE/ZIP一致，旧652证据及rc.1/rc.2的6份EXE/ZIP/manifest均未变。

证据目录：`build/verification/TASK-007-IPC-INTERVAL/verifier/`。仓库根执行本轮 `check.mjs`，只运行类型、前端helper、后端离线locked测试、版本检查和图标检查；没有执行fmt、同步或Coder打包脚本。使用scripts/environment.mjs并将临时/测试/Node缓存定向本轮项目内目录；Cargo缓存和target沿用项目内已有目录。未安装下载、未操作普通UI/游戏/真实F8/桌面输入，未修改生产源码。

| 条目 | 方法与结果 | 证据 |
| --- | --- | --- |
| IPC前后端边界 | 源码复核加生产helper/全后端复跑；两个通道均接受100/60000，拒绝99/0/负数/60001/小数/NaN/Infinity/缺失/倒置，无夹边 | frontend.log、backend.log |
| 默认值及UI | App草稿仍2000/2000；输入min100/max60000/step1，运行中仍锁定，原启动入口后端validate仍接入 | src/App.vue、src-tauri/src/lib.rs只读检查 |
| 传输与停止边界 | IPC四模块、storage、duration/hotkey相对rc.2哈希不变，生产随机采样/调度方法未改变；原非游戏读取诊断、停止/到期和自有Child测试复跑通过 | freeze-before/after.json、backend.log |
| 类型/前端 | 独立退出0，生产helper PASS | typecheck.log、frontend.log、results.json |
| 后端 | 独立76 passed / 0 failed / 1 ignored，ignored为父测试调用的子进程fixture入口 | backend.log |
| 候选归属 | 69文件/EXE/ZIP/hash前后一致；版本及PE字符串均0.1.0-rc.3，NotSigned，图标与源ICO相符 | freeze-before/after.json、version.log、pe-version.json、icon.log |
| ZIP洁净性 | 独立CRC通过，5文件逐字节对应源EXE/产品文档，包内无data/debug/cache，无解压或启动 | package-check.json |
| 文档/脱敏 | 使用与限制说明100～60000、默认2000、额外等待不等于实际频率；核对Coder已知机器前缀扫描为零，新增Verifier文件仅相对路径/脱敏输出 | docs/USAGE.md、docs/LIMITATIONS.md、coder/sanitization.json |

## Findings与边界

本次冻结范围内未发现findings。未变更或修复代码，未向Coder追加任务。

真实游戏100ms体验、实际频率/性能、完整桌面隔离、普通UI/F8/托盘/目标退出和Windows双平台均未实测，不用源码或fixture替代用户验收。rc.2用户循环成功的历史反馈保留，不作为rc.3的100ms实测通过。旧rc.2独立审核曾服务中断的历史不改写；本轮只对TASK-007变化及紧邻回归提供独立结论。

满足本轮限定审核停止条件；不扩展为全面发布审计，无提交推送、正式发布、远程上传或历史清理授权。
