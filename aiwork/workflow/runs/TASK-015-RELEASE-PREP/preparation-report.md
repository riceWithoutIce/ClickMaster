# 0.1.0 发布准备交接

状态：版本、文档、构建和主协调者包检查完成，等待用户人工回归；尚未正式发布。

## 本轮范围与来源

- 主协调者执行版本/文档整理及项目内构建、测试、包检查；本轮没有独立子 agent 验收，不冒充独立 Verifier。
- 基线提交 `e094ca897743ffd08ada44a5d61b722a13c5aeac`，当前为包含正式版元数据与文档的未提交快照。
- 版本源和 package/Cargo/锁文件自有包/Tauri 字段均为 `0.1.0`；依赖内容除自有版本外与基线一致。
- `src/`、`src-tauri/src/` 和 `scripts/` 相对基线无变更；正式 EXE 重新构建，没有直接改名旧 rc.9 产物。
- `source.json` 当前指向 84 项工作区文件指纹，按 Git 换行规范化前的文件字节记录；不宣称已有正式版提交或可重建出相同字节。

## 产物

| 项目 | 结果 |
| --- | --- |
| ZIP | `build/staging/TASK-015-RELEASE-PREP/ClickMaster-0.1.0-windows-x64.zip` |
| ZIP 大小 / SHA-256 | 3,224,535 字节；`ACEE1AE8045055D01DECE59F56FE7D238FA7A3DDEB9F2C6B2C9447EF008630DD` |
| 内嵌 EXE 大小 / SHA-256 | 9,774,592 字节；`9F263B8AC713DDDEF93C929DA38A6804077BAD0FAD82FD21B012894EE0A5DE88` |
| EXE FileVersion / ProductVersion | `0.1.0` / `0.1.0`，AMD64 |
| 包内容 | EXE、README、CHANGELOG、USAGE、LIMITATIONS，共 5 文件，无用户 data、依赖缓存或历史工程 |
| 签名 | NotSigned，随包说明已注明 |
| 来源 / 校验 | ZIP 同级 `source.json`、`SHA256SUMS.txt` |

## 执行证据

- 仓库根 `node scripts/version.mjs --sync` 与 `--check`：自有版本同步、校验通过，未升级依赖。
- `node scripts/project.mjs typecheck`、`test-targets`：类型及现有 UI 逻辑测试通过。
- `node scripts/project.mjs test`：编译通过，但原始库测试 EXE 因 `0xc0000139 / STATUS_ENTRYPOINT_NOT_FOUND` 未启动，不能记原始入口通过。
- 沿 TASK-011 已保留方法，在本任务独立目录复制测试 EXE，仅为测试副本嵌入 Common Controls v6 清单，再串行执行：100 passed、0 failed、2 ignored。两项 ignored 是驱动测试调用的内部子进程入口；未修改生产代码或生产 EXE。
- 桌面命令首次误用内部子命令 `build:desktop`，在构建前被拒绝；正确命令 `node scripts/project.mjs desktop` 随后完成前端和桌面构建、PE 图标检查，退出 0。
- 对生产 EXE 只读导出清单确认已有 Common Controls v6；本轮测试入口问题不推定为生产 EXE 启动失败。
- `package.ps1` 创建 ZIP 并逐条读取比较源文件大小/哈希；`verify.mjs` 再核对来源、源码差异、依赖版本、架构、图标、rc.9 和受限脱敏扫描，全部通过。
- 已知本机前缀及常见凭据特征未命中，不宣称穷尽所有隐私信息或证明全系统零外写。
- 构建环境为 Windows 10 22H2 x64（19045）；不能将构建成功替代产品在该环境的真实启动与输入验收。

项目内证据目录：`build/verification/TASK-015-RELEASE-PREP/coordinator/`，含 initial-checks、results、backend.log、package-content、source-manifest、package-result、closure 和生产清单。原始入口失败与隔离测试通过分别保存。

## 未执行与延期

- 新包普通 GUI、真实游戏、F8/托盘、焦点隔离、目标退出、数据与重启：待用户按 `human-test.md` 验收；未自动启动产品或发送游戏输入。
- Windows 11、缺失/旧 Runtime、长期资源及全系统外写监控：NOT RUN。
- OVERLAY-001、F-01、性能后续项及 V008-01 同步测试问题保持延期；本次测试通过不抹去历史失败或代表已修复。
- rc.9 EXE/ZIP 哈希保持一致，当前运行数据不处理，未结束其两个运行实例。
- `version/releases.json` 仍为空；未提交、推送正式版，未建标签、发布 Release 或淘汰 rc.9。

人工回归通过后再确认正式提交与对外发布。若交付内容有变化，须重新冻结相应包和指纹，不覆盖已完成验收的产物冒充同一包。
