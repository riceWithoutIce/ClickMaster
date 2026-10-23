# TASK-006-UI-CLEANUP 独立验收

日期：2026-10-22。Verifier：独立只读审核任务。结论：**约定源码、离线测试及本地候选包检查 PASS；本轮未发现 BLOCKER 或需提交用户裁定的 finding。** 这不表示真实 UI、游戏、跨系统或正式发布已通过。

## 被验快照

对象为 `TASK-006-UI-CLEANUP-rc.1-coder-final` dirty快照，不以Git HEAD代替当前源码。

- 69文件清单：`build/verification/TASK-006-UI-CLEANUP/coder/source-manifest.json`，SHA256 `B9198A948057C638206A536FFD4DF5C4B0C141962CD9FEC293ED7F650E9FEEA6`。
- EXE：`coder/final/ClickMaster.exe`（相对于本任务验证目录），9,454,080字节，SHA256 `369E6DC757757FB5D001083D5834699591220A2B0A23DA576E0D1C1BF2323F6D`。
- ZIP：`build/staging/TASK-006-UI-CLEANUP/ClickMaster-0.1.0-rc.1-windows-x64.zip`，2,952,449字节，SHA256 `27A8141BA4864F8832E12157D7AC6F5097DBFC89069EDA584B4D2E8F05EBE46A`。
- 独立审核前后69文件及EXE/ZIP哈希一致；652份保留基线逐项一致；最终EXE目录未出现data。证据：`verifier/freeze-before.json`、`freeze-after.json`。

本报告中的 `verifier/` 均相对于 `build/verification/TASK-006-UI-CLEANUP/`。只写本轮Verifier辅助产物及本报告，未改被验源、配置、锁文件或产品文档，未重跑Coder打包/最终构建入口。

## 验收结果

| 编号 | 方法与结论 | 结果/证据 |
| --- | --- | --- |
| AC-01 | 核对App.vue正常导航、筛选/绑定、歧义路由、通道和执行页；间隔/持续时间直接提供，底部集中操作 | 源码PASS；真实视觉/交互NOT RUN |
| AC-02 | details无open，诊断默认折叠；实验预览默认关且移入诊断。关键失败/partial/timeout/cleanup-failed及按下/释放失败原因在折叠区外展示；展开不调用执行 | 源码PASS；OVERLAY-001仍延期 |
| AC-03 | 前端durationError与后端Request覆盖永久/限时、1与359999秒、空/分数/负/非有限/越界/零拒绝；永久忽略隐藏草稿，单次命令不带重复草稿；运行锁定参数 | 类型/helper/后端PASS |
| AC-04 | 准备后单调起点；等待中及prepare后按下前复核截止；完成后独立等待，不追赶；原身份配对释放，partial优先 | 后端fake clock/adapter及源码PASS |
| AC-05 | IPC准备/传输/等待统一监督截止；stdin授权worker前复核；阻塞drain非游戏随机管道fixture调用生产截止策略与owned-Child cleanup | 后端/源码PASS；完整AppHandle/游戏IPC未运行 |
| AC-06 | 默认永久无deadline仍有保护；冻结耗时/原因，新轮重置。手动/到期停止保留触发原因，partial/timeout/cleanup-failed不改写成功；清理未确认保持busy锁定 | 后端/helper/源码PASS |
| AC-07 | 单次、固定/随机、身份和路由、单帧/鉴权/超时、槽位边界、配置、托盘策略、fake热键紧邻回归；按钮/F8/托盘/退出走原统一取消链 | 离线测试/源码PASS；物理F8/托盘/退出NOT RUN |
| AC-08 | VERSION/package/Cargo/锁/Tauri均0.1.0-rc.1；PE字符串相同，数值0.1.0.0为SemVer映射；AMD64 PE32+，图标资源与源ICO一致 | 独立version/icon/PE检查PASS |
| AC-09 | ZIP仅EXE、README、USAGE、LIMITATIONS、CHANGELOG；CRC与逐字节源文件一致，无data/debug/cache；文本及二进制已知路径前缀无匹配 | 独立包/有界脱敏检查PASS |
| AC-10 | README/使用/限制与Coder记录区分旧人工回执、当前fixture、NOT RUN及NOT PROVEN；未签名说明随包，无正式晋级/上传，release索引仍空 | 声明边界/静态检查PASS；环境/性能项保留未验 |
| AC-11 | 本版新UI、永久/短限时、真实游戏及结束反馈 | NOT RUN，用户按human-test验收 |

## 独立执行证据

仓库根执行 `node build/verification/TASK-006-UI-CLEANUP/verifier/run-checks.mjs`。实际子命令为vue-tsc `--noEmit`、前端生产helper脚本、`cargo test --release --locked --offline --manifest-path src-tauri/Cargo.toml`、只读图标、VERSION与PE检查。结果见 `run-results.json` 及同名日志。

后端 **70 passed / 0 failed / 1 ignored**；ignored为父测试显式调用的私有子进程fixture入口。类型及7组前端helper检查通过。未新增依赖或下载，复用项目内Cargo缓存/target；本轮fixture、TEMP/TMP/TMPDIR、Node缓存位于Verifier目录。Rust路径重映射沿用projectEnvironment。不将此设置推断成全系统外写监控。

PE签名/版本首试使用Windows PowerShell，安全模块无法自动加载；该日志已保留，其退出0不作为成功证据。使用已有pwsh重新只读检查成功，见 `pe-version-r2.json`：字符串版本0.1.0-rc.1，数值0.1.0.0，签名NotSigned。此为工具兼容问题，无产品修改。

`python -B build/verification/TASK-006-UI-CLEANUP/verifier/check-package.py` 在内存中读取ZIP，无额外解压副本，验证5文件CRC/字节/哈希。`package-sanitization.json`仅记录相对路径与匹配数；检查当前清单文本和包内内容/EXE的已知仓库及用户前缀UTF-8/UTF-16，均0匹配。静态导入/延迟导入仅系统DLL，无私有随包DLL或动态WebView2Loader导入；共享Evergreen仍为外部依赖。此扫描不证明穷尽隐私、动态依赖或所有系统环境可运行。

## Findings与停止边界

本次范围内未发现 findings；未向Coder追加工作或修改被验源。

本版普通UI未启动；未注册真实F8、查询游戏、建立游戏管道或发送桌面输入。非游戏fixture的截止决策与cleanup组合测试不冒称完整真实应用端到端验收。中文/空格移动升级是纯文件fixture证据，不能替代WebView启动。

新UI体验、真实限时/永久及游戏响应、完整工作输入隔离、真实退出/目标失效/热键冲突等仍留用户；Windows 10/11双平台、缺失/过旧Runtime及性能未运行；最低WebView2所有功能组合NOT PROVEN。当前为本地RC体验候选，这些限制已明示，不将未来正式发布门槛当成本轮已验能力。OVERLAY-001、多实例无新增互斥、缓存可增长及异常槽位不自动清扫等原限制保持。

约定独立检查通过、无BLOCKER，当前审核停止。后续人工结果及正式晋级由协调者/用户分别记录；本报告不授权提交、推送、上传、发布晋级或历史清理。
