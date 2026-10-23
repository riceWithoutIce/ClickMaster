# TASK-008非游戏性能基线

日期：2026-10-22。状态：rc.3适配fixture基线完成；真实游戏、Tauri/WebView2、完整产品性能未测。生产源码未修改。原始结果在 `build/verification/TASK-008-IPC-EFFICIENCY/performance/baseline/`。

## 方法与等价范围

冻结rc.3的6个IPC相关源码和lib：`performance/baseline-source/manifest.json`。独立Rust harness直接编译冻结的ipc_native.rs副本，依赖版本与项目已有缓存一致、离线构建，无新安装。仅将目标EXE/窗口类替换为独立fixture身份，程序集路径替换为项目内synthetic文件，预期hash从测试进程环境传入。fixture接收器创建离屏、NOACTIVATE/TOOLWINDOW且可见的16×16窗口，不获取真实游戏身份或抢焦点；仅连接随机私有测试管道。

复用生产native的真实进程/桌面枚举、Lease全量校验、当前用户管道ACL/first-instance、overlapped connect/write与Flush。父端代表性监督按10ms读取状态并执行完整Lease校验，轮间按50ms完整校验；每轮独立worker，完整4字节固定帧，接收器连接后固定等待90ms再读，无批量或补发。每次完整传输后再等待2000或100ms，不能按等待值推算实际频率。

每帧父/子各读取和SHA256校验rc.3正式EXE（9456640字节），子端Lease::open读取SHA256校验固定4MiB synthetic assembly。后者大小不是实际游戏程序集大小，不将其成本冒称游戏原成本。实际harness EXE小于正式EXE；未复用完整worker父子私有授权、Tauri AppHandle、200ms未确认分支、热键、后台UI轮询和WebView。状态文件是短标签而非完整JSON，这部分文件解析/发布成本不等价。因此结果是检查/短命worker/管道的受控基线，不是rc.3原包完整性能或各函数精确占比。

CPU通过GetProcessTimes读取主发送端及每个直接拥有的worker句柄，worker退出后仍累计其CPU，避免按进程列表轮询漏掉短命进程。接收器单列，16逻辑处理器，整机百分比=累计CPU秒/实际秒/16。每秒采样发送端波动；CPU计时分辨率约15.6ms，结果0表示低于分辨率，非绝对零开销。三个30秒场景顺序执行，单次样本，未隔离全系统外部负载。未声称统计显著或长期资源无增长。

## 测量结果

| 场景 | 实际秒 | 帧数/worker数 | 父完整校验次数 | 发送端CPU秒 | worker累计CPU秒 | 二者整机CPU均值 | 接收器CPU秒 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fixture待命 | 30.032 | 0/0 | 0 | <0.0156 | 0 | <0.004% | <0.0156 |
| 完成后等待2000ms | 30.052 | 14/14 | 525 | 8.250 | 1.406 | 2.008% | 0.0156 |
| 完成后等待100ms | 30.107 | 87/87 | 896 | 15.109 | 8.266 | 4.852% | <0.0156 |

100ms场景实际约2.89帧/秒，不是10帧/秒。发送端CPU每秒样本约2.60～4.04%整机；worker累计单列约1.716%，总均值4.852%。2000ms场景每帧平均发送端加worker CPU约0.690秒，100ms约0.269秒；长等待期间持续完整枚举使“每帧成本”口径受等待长度影响，后续比较必须在同一间隔下进行。

附带25个一秒内存快照（覆盖运行场景但未精确逐场分段）：发送端工作集最高6.93MiB、专用内存2.35MiB；receiver8.55/1.54MiB；捕获7个worker快照最高6.27/1.30MiB。短命worker采样可能漏峰值，仅辅助观察，不是完整峰值/长期泄漏结论。WebView2未启动，记不适用而非0占用。

## 对第一阶段的支持与限制

基线确认独立fixture中间隔缩短时CPU及worker创建次数上升，等待和监督期间分别出现525/896次昂贵全量校验，支持优先把全局枚举从10/50ms循环移出、改为直接原实例检查并保留发送前完整核验。不能仅凭累计CPU证明枚举、哈希、进程创建分别占多少，也不能据此承诺真实产品降低具体百分比。

推荐比较目标：相同接收90ms策略、相同固定间隔和输入数量定义下，首阶段CPU每帧明显下降且不减少发送量，worker不重叠/无残留、停止/目标保护通过。是否定量承诺由协调者在基线后冻结；尚未运行优化版，不写预期比例为事实。

后续A/B保持同一native适配、rc.3正式EXE双次哈希、4MiB assembly、90ms读取、发送后额外等待。优化若减少等待导致帧数增加，同时报告单位时间CPU和每帧CPU，必要增加等帧数比较；UI cache收益不在此fixture范围。第一阶段候选的真实窗口交互/玩法/资源仍由用户验收。

## 运行证据

仓库根先运行 `node build/verification/TASK-008-IPC-EFFICIENCY/performance/prepare.mjs` 冻结并构建，再运行 `node build/verification/TASK-008-IPC-EFFICIENCY/performance/run.mjs baseline`。prepare会写baseline-source，冻结后不得在生产改动后重跑。两秒smoke输出在smoke目录，正式结果在baseline/result-{0,2000,100}.json；内存辅助样本memory-sample.json。所有项目缓存/临时及输出通过scripts/environment.mjs留项目内，日志已重映射本机路径。

首次工具编译失败为测试harness的cast比较括号与缺少WNDCLASS所需既有Windows feature，已修正；非产品缺陷，不放松任何测试标准。无游戏固定管道、真实F8、桌面全局输入或焦点切换，无生产/版本修改或历史删除。

## 优化版同fixture比较（2026-10-22）

最终测量源码独立保存在 `performance/optimized-final-source/`，manifest SHA256 为 `F1A78E2136AB215685D822345E5BBF2A9067981EE9FE2E6E19E5D4165009765C`；native SHA256 为 `713A6C9029449C95353037524E0071C8D3DAFBFAD82E147E7574CBD47888FA21`。测完与当时生产 native 做逐字节 diff 无差异。baseline manifest 复核仍为 `2C02A2F82C17A84642AE4F7F2E1B5407FA396C5519A69093FAD18A013C964E1F`，未重建或覆盖历史基线。

候选同样离线编译原生代码副本，只做同目标身份/程序集 fixture 替换。父端 Lease 在测量开始前完整准入一次，监督与等待仅 `validate_fast`；worker 每帧 Lease open 保留 assembly hash，连接后一次 fast+真实管道客户端核验，写前一次 `validate_for_send` 完整枚举+客户端核验，再唯一四字节写入。每帧父/子依然各读取同一个9456640字节rc.3正式EXE作hash，不拿候选体积替换工作量；保留相同4MiB assembly、90ms接收延迟和完成后额外等待。未实现持续worker，也未减少帧数/改协议。

| 场景 | 版本 | 实际秒 | 帧/worker | 父完整/fast次数 | 发送端CPU秒 | worker CPU秒 | 二者整机CPU均值 | 二者CPU秒/帧 | 接收器CPU秒 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 待命 | baseline | 30.032 | 0/0 | 0/0 | <0.0156 | 0 | <0.004% | 不适用 | <0.0156 |
| 待命 | optimized-final | 30.036 | 0/0 | 0/0 | <0.0156 | 0 | <0.004% | 不适用 | 0.0156 |
| 等待2000ms | baseline | 30.052 | 14/14 | 525/0 | 8.250 | 1.406 | 2.008% | 0.6897 | 0.0156 |
| 等待2000ms | optimized-final | 30.012 | 14/14 | 0/771 | 0.188 | 0.672 | 0.179% | 0.0614 | <0.0156 |
| 等待100ms | baseline | 30.107 | 87/87 | 896/0 | 15.109 | 8.266 | 4.852% | 0.2687 | <0.0156 |
| 等待100ms | optimized-final | 30.023 | 115/115 | 0/2036 | 1.516 | 5.359 | 1.431% | 0.0598 | 0.0156 |

优化worker完整枚举为2000ms场14次、100ms场115次（各一帧一次，另有测量外父准入一次）。受控fixture中，2000ms同14帧CPU总量9.656→0.859秒，约下降91.1%；100ms固定时间CPU均值约下降70.5%，发送量87→115帧，实际约3.83帧/秒。后者每帧CPU约下降77.7%，不将115帧误称每秒10帧。

为排除吞吐差异，再执行100ms候选**固定87帧**：23.197秒、87个worker、父1550次fast，发送端CPU1.219秒、worker4.078秒，合计5.297秒（0.0609秒/帧），接收器0.0156秒；相对baseline同87帧23.375 CPU秒约下降77.3%。baseline为固定30秒循环，包含少量尾端等待；此尾差远小于6.9秒耗时差但仍明确披露。固定帧候选总预算60秒，到87帧即停，无补发或延长baseline。

这支持“将全局枚举移出热循环后，相同输入fixture单位工作CPU显著下降”的工程结论。**不代表完整ClickMaster或真实游戏下降相同比例**：本测量没有UI/WebView、兼容性缓存轮询或实际游戏程序集，也未隔离全机外部负载、未做统计重复。保留逐帧worker与hash后，100ms场worker CPU仍占合计大部分；是否继续优化需真实产品人工体验后另议。

同方法补25个一秒内存辅助快照：候选发送端工作集最高5.86MiB/专用1.27MiB，接收器8.55/1.55MiB，9个捕获worker快照最高6.21/1.26MiB。baseline发送端6.93/2.35MiB、接收器8.55/1.54MiB、7个worker6.27/1.30MiB。未精确场景分段且可漏短命worker分配峰值，不能据此承诺内存降幅或无泄漏。初候选曾捕获14.49/10.15MiB worker一帧快照，更说明低频采样不适合给真实峰值结论。

最终结果在 `performance/optimized-final/result-{0,2000,100}.json`、`optimized-final-equal87/result-100.json`、`comparison.json`；内存样本 `optimized-final/memory-sample.json`。命令为：`node build/verification/TASK-008-IPC-EFFICIENCY/performance/prepare-optimized.mjs optimized-final`（已冻结，不得覆盖）；`node build/verification/TASK-008-IPC-EFFICIENCY/performance/run-optimized.mjs optimized-final 0,2000,100 30 0 optimized-final`；`node build/verification/TASK-008-IPC-EFFICIENCY/performance/run-optimized.mjs optimized-final-equal87 100 60 87 optimized-final`；`node build/verification/TASK-008-IPC-EFFICIENCY/performance/compare.mjs`。

`optimized/`和`optimized-equal87/`保留初候选观测，少一次连接后fast/client检查，未用作最终比较。最终候选加回这一步后重新测量三场与等帧，未覆盖初测。测量结束所有fixture进程正常退出，无游戏输入。真实游戏与UI交互、完整产品CPU、托盘/WebView、真实停止与异常保护仍由对应功能测试和人工验收独立确认。
