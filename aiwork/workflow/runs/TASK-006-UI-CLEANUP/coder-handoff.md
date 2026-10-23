# TASK-006-UI-CLEANUP Coder交接

日期：2026-10-22。状态：实现、自测、desktop及便携候选检查完成，源码/配置/锁文件冻结，可进入独立Verifier。无普通UI、真实游戏查询/管道、真实F8注册或桌面输入；无提交、推送、正式晋级、历史清理。

## 本轮修改及原有基线

基于TASK-005已人工验收的dirty源码与57文件manifest实现，Git HEAD仅参考，不代表本轮源码。此前大量未提交/未跟踪源码、产品文档及历史任务均保留，不重置本地状态。

- 新增 `src-tauri/src/duration.rs`：会话永久/限时策略、前后端一致整数边界、单调起点/截止、秒级进度、冻结结束和触发原因；限时1～359999秒，永久没有deadline，不依赖隐藏草稿。时间算术溢出拒绝。
- `input.rs`：准备结束起算，等待中及按下前再次检查截止；到期不新发，已按下仅向原身份配对释放。partial等错误优先，结束冻结耗时与次数，重启新一轮。原消息/随机间隔/原实例及路由保护保留。
- `ipc_runtime.rs`：准备结束起算；连接/写/读取及轮间等待由父端监督截止，授权worker前再检查，不等待原60秒预算才取消；清理仅直接持有Child。执行结束计时和清理门槛分开，失败/超时/清理锁定不改写正常到期；原worker/协议/ACL/客户端核验未重写。
- `input_runtime.rs`、`lib.rs`：新状态、后端有效模式校验、秒级事件和统一停止服务失败原因；运行准入/锁定与F8生命周期沿用，未注册真实F8。单次不接收重复持续时间/间隔，不受非法草稿影响。
- `src/App.vue`、`src/style.css`、`single-input.ts`及新 `execution-time.ts`：精简顶栏，必要参数与歧义路由仍直接可用；诊断放操作栏上方默认折叠，实验预览移入且默认关闭，OVERLAY-001未修。执行页永久/限时和进度，底部统一状态/次数/时间/按钮，关键错误及具体拒绝原因直接显示；结束总结同时表达截止触发与清理结果。
- `scripts/test-target-filter.mjs` 扩展生产helper校验；新增 `version.mjs` 由 `version/VERSION` 同步自有package/Cargo/锁/Tauri，`project.mjs`构建/测试预检，`build.rs`核对VERSION与Cargo。未升级依赖、未新增库。
- 新增只读 `scripts/inspect-pe.mjs`，用于候选架构/导入闭包检查；不执行或加载被验程序。
- 根README、`docs/USAGE.md`、`docs/LIMITATIONS.md`、version/README与CHANGELOG及当前local-data/runtime-distribution说明更新，保留历史回执；写明候选状态、具体兼容边界、共享WebView2、多实例/缓存/残留及NOT RUN环境。

实际文件差异以最终 `source-changes.json` 对照TASK-005 manifest列出；文档不在旧57份清单内的项标记为本清单新增，不意味着历史没有此文件。协调者任务/人测/发布门槛文档非Coder写入。

## 检查、环境和证据

命令均在仓库根，经 `scripts/environment.mjs` 传入项目内TEMP/TMP/TMPDIR、npm/Cargo/Node缓存、Cargo target与路径remap；Cargo `CARGO_NET_OFFLINE=true`，锁定依赖。没有全局安装、依赖下载、.NET、外部目录临时产物。不能据此声称已监控整个系统零外写。

证据位于 `build/verification/TASK-006-UI-CLEANUP/coder/`。最终精确源码为70 passed / 0 failed / 1 ignored（backend-frozen.log）；typecheck、frontend helper和desktop均退出0，图标匹配（同名-frozen.log及final-icon.log）。早期63/68/70 passed均无failed，ignored仅由父测试明确调用的子进程fixture入口，不是遗漏的验收要求。首轮日志使用同名r1被后续自测更新，最终以frozen日志和清单为准，无失败记录被隐藏。

测试覆盖：限时/永久边界、隐藏草稿、准备不计时、时钟溢出、截止早于间隔、不补发、截止前最终检查、原身份释放及partial优先、摘要冻结/再次开始、永久仍有保护、所有IPC阶段的统一截止策略、独立随机非游戏管道阻塞drain时fake clock到期后生产owned-Child清理、停止服务失败原因、中文空格目录移动/替换fixture保留配置。原消息/单帧/first-instance/鉴权/身份/超时/槽位/配置/托盘策略/fake热键紧邻测试保留。

非游戏drain测试是“生产截止决策 + 生产cleanup + 独立原生阻塞Child”的组合，不冒称完整真实AppHandle/游戏IPC运行。中文目录/移动是文件fixture，不是普通UI启动或WebView有效运行路径证明。

## 未执行和产品限制

真实rc.1 UI渲染/交互、永久和短限时游戏响应、物理F8/冲突/托盘/退出、目标失效和完整工作输入隔离：NOT RUN，留人工。Windows双平台、缺失/低版本WebView2、启动和待命/运行性能：NOT RUN；不修改外部环境补证。当前锁定组合所有功能最低WebView2版本未独立证明，不将存在Runtime当兼容证据。

多实例/共享data没有新增互斥能力，正常IPC单槽位无逐帧历史，异常残留不启动扫描清理；WebView缓存不承诺固定大小。以上使用/限制说明随包。OVERLAY-001仍延期，不扩新能力。

## 稳定快照和本地候选

最终manifest与包核验完成，独立Verifier尚未启动；不发布或上传。便携ZIP只含EXE、简短README、使用/限制/变更说明，不含data/debug/cache；staging不覆盖旧包。


| 最终检查 | 结果/证据 |
| --- | --- |
| 精确源码后端 | 70 passed / 0 failed / 1 fixture ignored；backend-frozen.log |
| 前端/类型 | PASS；typecheck-frozen.log、frontend-frozen.log；含持续时间边界/冻结总结 |
| 正式模式构建 | PASS；desktop-frozen.log，Cargo离线locked、无安装器 |
| 图标 | PASS；源ICO与最终PE资源匹配，final-icon.log |
| 版本 | VERSION/package/Cargo/锁/Tauri均0.1.0-rc.1；PE FileVersion/ProductVersion字符串均0.1.0-rc.1，数值均0.1.0.0（Tauri SemVer映射，不独立表示rc） |
| PE/签名 | AMD64 PE32+；静态/延迟导入无随包私有DLL要求，无动态WebView2Loader DLL导入，仍依赖共享Evergreen；NotSigned，包说明已告知系统可能提示 |
| ZIP闭包/洁净 | 5文件：EXE、README、docs/USAGE、docs/LIMITATIONS、CHANGELOG；CRC及解压后逐字节一致，无data/debug/cache。只移除本任务新建的临时解压副本，未清理历史 |
| worker拒绝 | 最终包仅运行无授权--ipc-worker与额外参数，两者退出64，无stdout/stderr/data；无普通UI |
| 脱敏 | 当前源码/文档文本和二进制UTF-8/UTF-16已知仓库/用户前缀0匹配；不声称穷尽隐私或OS监控 |
| 原证据 | 协调者652历史文件逐项hash均未变；preservation.json |

版本同步只更新自有元数据，依赖锁定未升级。`hotkey.rs`相对TASK-005有cargo fmt格式变化，未改变其热键行为。`release-checks.json`逐项区分静态PASS、fixture、NOT RUN和NOT PROVEN；缺失/过旧Runtime、双平台、性能、人机隔离不伪造通过。发布索引`version/releases.json`保持空；没有晋级release槽位。

快照：`TASK-006-UI-CLEANUP-rc.1-coder-final`，HEAD `d3d8f58ba8490212a16f33faecb4b5533d79bd5d`仅参考，dirtySnapshot=true。

- 69文件源码/配置/锁/产品文档manifest：`build/verification/TASK-006-UI-CLEANUP/coder/source-manifest.json`；SHA-256 `B9198A948057C638206A536FFD4DF5C4B0C141962CD9FEC293ED7F650E9FEEA6`。
- EXE：`build/verification/TASK-006-UI-CLEANUP/coder/final/ClickMaster.exe`；9,454,080字节；SHA-256 `369E6DC757757FB5D001083D5834699591220A2B0A23DA576E0D1C1BF2323F6D`。
- ZIP：`build/staging/TASK-006-UI-CLEANUP/ClickMaster-0.1.0-rc.1-windows-x64.zip`；2,952,449字节；SHA-256 `27A8141BA4864F8832E12157D7AC6F5097DBFC89069EDA584B4D2E8F05EBE46A`。
- staging配套`source.json`和`SHA256SUMS.txt`；不复制用户data。`snapshot.json`记录上述来源；manifest差异见source-changes.json。

最终构建入口 `coder/final-build.mjs`，打包检查入口 `coder/package.mjs`；均只限本任务辅助产物，不是正式发布工具。package.mjs会写Coder输出并拒绝覆盖现有ZIP，Verifier不要重跑，只读清单与重新计算hash。开始/结束须核对69文件及包hash；交接后Coder停止修改被验源码。最终包以当前manifest归属，早期测试或HEAD不代替该快照。
