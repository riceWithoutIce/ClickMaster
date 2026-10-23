# TASK-007 Coder稳定交接

日期：2026-10-22。状态：实现、自测、desktop构建及独立rc.3便携包准备完成，生产源码冻结，可进入限定独立Verifier。无普通UI、真实游戏/固定游戏管道、真实F8或桌面输入；无提交、推送、正式发布或历史清理。

## 本轮修改

- `src-tauri/src/repeat.rs`：IPC与消息的重复间隔均验证100～60000毫秒整数，保留原调用接口、拒绝非法值/倒置、不夹边和随机采样/单调调度逻辑。扩展原边界测试，两个通道均接受100和60000、拒绝99/0/负数/60001/小数/NaN/Infinity/缺失/倒置。
- `src/input-channel.ts`与`src/App.vue`：校验/输入控件下限统一100；默认草稿仍2000/2000，最大60000、运行中锁定保持。界面说明明确间隔是完成后的额外等待，实际频率取决于传输耗时。
- `scripts/test-target-filter.mjs`：生产helper对两个通道的同边界检查。
- VERSION与自有package/Cargo/锁/Tauri元数据递增0.1.0-rc.3；根README、使用/限制、版本说明/CHANGELOG同步。无依赖升级。

相对rc.2的69文件清单：15文件变化，无新增/删除项，实际见`source-changes.json`。全部IPC模块、worker、duration/hotkey及其他停止逻辑哈希未变（`transport-preservation.json`），不重构传输/调度或读取诊断，不新增持久化。rc.1/rc.2唯一包及用户data保留；此前dirty/未跟踪状态未重置。

## 验证与证据

仓库根运行 `node build/verification/TASK-007-IPC-INTERVAL/coder/check.mjs`。该入口调用scripts/environment.mjs，项目内TEMP/TMP/TMPDIR、npm/Cargo/Node缓存及Cargo target，Cargo离线locked，新`tests-final`测试根；不写系统临时目录、不安装下载或改全局环境。实际检查：cargo fmt、typecheck、全backend、前端helper、desktop。无法据此宣称监控了操作系统全部内部写入。

| 检查 | 结果 |
| --- | --- |
| 后端全量 | 76 passed / 0 failed / 1 ignored；backend-final.log。ignored仅由父测试显式调用的子进程入口 |
| 前端生产helper | PASS；两个通道100/60000边界与非法值、原读取诊断/持续时间策略均通过；frontend-final.log |
| 类型检查 | PASS；typecheck-final.log |
| desktop/图标 | PASS；desktop-final.log和final-icon.log |
| 版本/架构 | 0.1.0-rc.3；AMD64 PE32+，PE字符串版本一致，数值0.1.0.0为Tauri转换，不单独表达rc身份 |
| ZIP | 5文件；CRC/解压逐字节一致，无data/debug/cache；zip-validation.json |
| 无授权worker入口 | 两种无授权调用均退出64、无输出/数据；worker-entry.json，不启动普通UI |
| 脱敏 | 已知仓库/用户前缀文本及二进制UTF8/UTF16零匹配；sanitization.json。非穷尽隐私保证 |
| 历史保留 | 旧652项hash未变；rc.1/rc.2的EXE/ZIP/源清单6项hash未变；preservation.json及prior-packages-preservation.json |

构建/测试首次全通过，无本轮失败需重试。真实游戏100ms体验、实际频率/性能、完整桌面隔离、UI/F8/托盘/目标失效/双平台环境未执行，均NOT RUN；原源码/fixture回归不能替代用户运行验收。读取未确认不阻断仍是rc.2行为，本轮没有新增对该行为的真实游戏证据。

## 稳定快照与包

- 快照：`TASK-007-IPC-INTERVAL-rc.3-coder-final`；HEAD仅参考，dirtySnapshot=true。
- 源码清单：`build/verification/TASK-007-IPC-INTERVAL/coder/source-manifest.json`；69文件；SHA256 `3ED1B7AA576DAC85E49FA4141FE4955C7BD84245416785E71DE56E65929C0FB7`。
- EXE：`build/verification/TASK-007-IPC-INTERVAL/coder/final/ClickMaster.exe`；9456640字节；SHA256 `71C5B6806F590460DCA8271D96000681BB9294B2EAA2BBC645C0688918E36AC3`。
- ZIP：`build/staging/TASK-007-IPC-INTERVAL/ClickMaster-0.1.0-rc.3-windows-x64.zip`；2953942字节；SHA256 `AC1AF361AD80323326B0C6A590610758DA48D7CD0DDFE36A1F5DAEEC2642A5B8`。
- 新staging配套`source.json`、`SHA256SUMS.txt`；只本地候选，NotSigned，依赖已有共享WebView2，未正式发布/晋级release。

本轮打包辅助入口`coder/package.mjs`会写产物且拒绝覆盖ZIP，Verifier不得重跑。只读核对清单/包hash，使用独立项目内测试目录；不执行fmt或版本同步。交接后Coder停止修改被验源码与候选包。
