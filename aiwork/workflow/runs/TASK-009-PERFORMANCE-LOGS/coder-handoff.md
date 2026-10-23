# TASK-009 Coder交接

日期：2026-10-22。Coder：task009_coder。状态：closure轮次自测、桌面构建与本地候选打包完成，生产源码/配置/版本已冻结，准入独立限定Verifier；未执行普通UI或真实游戏。

## 交付边界

- 实现默认关闭的性能采集、现有诊断区开始/结束入口、60秒自动结束、每秒采样与5秒批量写盘。采集不启动/停止连点，也不修改目标、间隔、持续时间、授权、身份核验、计数或原停止/清理保护。
- 主程序CPU/工作集/专用内存；自有Child句柄复制后追踪短命worker CPU和匿名生命周期、采集边界截断及缺失；有界父/worker阶段墙钟统计、执行上下文和本工具窗口状态。WebView2未建立可靠归属，本版明确unavailable，而非0占用。
- 关闭路径只做开关判断，无采样线程/日志/额外进程枚举。开启时测量点try_lock更新有界内存，争用计丢弃，不向输入线程写盘；独立日志线程落盘，退出最多等待250ms，慢盘/异常退出可能留下不完整记录。
- EXE同级data/logs生成匿名JSONL和TXT，每会话1MiB、最多5套/5MiB，仅轮换已完成且可识别会话；目录独占句柄阻止同目录并发诊断，未知/不完整/锁定文件保留。不可安全腾出预算时拒绝新采集，不回退外部目录。
- worker授权新增可选诊断代次；阶段统计经原私有状态报告传回并按代次匹配，不把先前采集的worker阶段归到新会话。报告增加固定有界数据，上限2048→4096字节，诊断关闭时额外统计字段省略；原校验/写入和读取只诊断语义不变。
- Windows资源查询仅启用已有windows-sys的ProcessStatus feature，无依赖升级或新增第三方库。帮助/版本同步rc.5；hotkey.rs未改，V008-01原失败与处置仍保留，不能因本次通过而声称修复。

## 来源与候选

最终权威来源：`build/verification/TASK-009-PERFORMANCE-LOGS/coder/snapshot.json`、`source-manifest.json`、`source-changes.json`。Git HEAD只作参考，当前dirty工程不是提交。

EXE：`build/verification/TASK-009-PERFORMANCE-LOGS/coder/final/ClickMaster.exe`。ZIP：`build/staging/TASK-009-PERFORMANCE-LOGS/ClickMaster-0.1.0-rc.5-windows-x64.zip`，配套source.json与SHA256SUMS。rc.1～rc.4及用户data不覆盖；652历史项与12项候选EXE/ZIP/manifest按coordinator保留基线核对，全部未变。

- 快照：TASK-009-PERFORMANCE-LOGS-rc.5-coder-final，71源码文件，18项修改、2项新增、无移除；新增performance.rs与perf_native.rs。
- source-manifest SHA256：`2423B0DA7062173A6D92E22B63A2F09340886C22545F044FE3958A444F81A551`。
- EXE：9674240字节，SHA256 `5CDC17F605E27A096E9667DEF08C8CA4C13630A065F069AE0CC4587ADD8AEF93`。
- ZIP：3031308字节，SHA256 `6B8B5397DC401028EC51E043F7CDCA90DF8BAE85BB67DD33E455145C43B11104`。NotSigned，x64 PE，版本字符串0.1.0-rc.5、数值0.1.0.0，实际PE图标匹配。ZIP五条目CRC及解压字节一致，无data/日志/开发残留。

## 自测与方法

从仓库根运行`node build/verification/TASK-009-PERFORMANCE-LOGS/coder/check.mjs fmt typecheck backend frontend desktop`，最后稳定轮次以进程环境`CLICKMASTER_CHECK_RUN=closure`标记。脚本继承scripts/environment.mjs，Cargo offline/locked，缓存/临时/test root均在项目内，日志将根路径替换占位符。不同轮次compile1/integration2/final/freeze/closure保留，不覆盖早期证据。

- compile1后端86通过、0失败、1子进程fixture忽略，类型通过；仅初始功能，不充当最终快照证据。
- integration2与final后端各90通过、0失败、2隔离fixture忽略，类型/helper通过。此前结果不替代freeze轮次。
- freeze与最终closure后端各91通过、0失败、2隔离fixture忽略；closure覆盖最后的退出收尾顺序与CPU查询缺失后成功基线不重复累计。fmt/type/helper通过。desktop构建后核对PE图标、实际版本资源及签名状态；独立无授权worker入口只退出，不启动UI。
- 最终closure desktop/版本/图标、无授权worker退出64且无输出/不创建data、ZIP CRC/逐字节、已知本机前缀脱敏检查通过。证据为desktop-closure.log、pe-version-signature.json、worker-entry.json、zip-validation.json、sanitization.json、preservation.json与candidate-preservation.json。本机前缀扫描非敏感信息穷尽证明。
- 新测试覆盖CPU归一/缺失、匿名固定词汇、默认关闭、重复入口/手动结束/短预算自动结束/有界shutdown、目录互斥、未知与不完整记录保留、完成记录轮换、会话字节预算、真实原Child退出CPU及首边界截断、结束边界不终止worker、生产日志模块非游戏私有管道开关比较。
- 60秒边界通过生产end_reason纯helper的可控时间验证，另用同一capture循环的20ms测试预算验证自动收尾；没有将短测声称实际跑满60秒。慢盘用busy状态模拟收尾未响应验证250ms等待预算，不声称实机慢盘注入。

非游戏比较使用本轮单元测试EXE、实际Collector/原生管道与send_frame、父/子各哈希同一测试EXE，每次一帧，完成后100ms额外等待，24帧/24worker。接收器为测试驱动进程，仅随机ClickMaster-OfflineFixture管道，读取固定4字节；退出后通过原句柄累计每个worker CPU。开/关使用同样接收策略和额外待命时间，最后稳定轮次额外等待3000ms，确保执行真实5秒批写路径，之后手动结束采集。输出位于`tests-closure/performance-integration/comparison.json`及on/logs/匿名文件。`node build/verification/TASK-009-PERFORMANCE-LOGS/coder/validate-logs.mjs`复核真实输出的CPU公式、24匿名worker明细与原句柄CPU总量相等、发送总量、已知前缀/敏感字段与会话字节预算；证据为log-validation.json。

比较范围：真实诊断模块、进程/文件hash/原生管道/worker启动与CPU；未执行真实游戏身份/程序集/授权链/窗口/普通UI/WebView，不代表完整产品开销。单次顺序、未隔离外部负载、CPU约15.6ms分辨率；开启测得更低也不能称负开销或零开销。阶段计时为嵌套墙钟，不能当CPU份额，缺失/跨边界/被中断worker统计分别标记。

最终同24帧/24worker，关闭CPU合计0.421875秒、耗时6.383秒；开启0.406250秒、6.309秒，仅差一个CPU量子，不支持性能改善或零开销结论。两边0错误，真实日志6个样本、24个匿名worker、已发送24、CPU总量与原句柄一致、JSONL+TXT11018字节。已知前缀/禁止身份字段0，WebView2明确unavailable，数据/限制见log-validation.json。

## 尚未执行与验收入口

真实UI/游戏、真实F8/托盘、实际60秒体验、长期性能、跨Windows/Runtime均NOT RUN。无游戏固定管道、真实输入、焦点操作、上传、提交推送、正式晋级或历史清理。

Verifier应先核对最终来源/产物hash及文件冻结，按acceptance.md限定只读检查。性能可复核固定数据和真实collector归属；不应把fixture开关开销或本版WebView未测推广为完整产品资源结论。全部被验源码保持冻结后才准入，协调文件及独立测试输出除外。

Coder生产写入已停止，无其他待编译改动。后续只允许独立验证输出与协调记录；修复需依据用户finding处置另行进入Coder轮次。
