# ClickMaster

轻量级 Windows 后台重复输入工具，采用 Tauri 2、Rust、Vue 3 和 TypeScript。**0.1.0 首个正式版**采用 Windows x64 免安装 ZIP 分发，已完成人工回归。

下载与发布说明：[ClickMaster 0.1.0](https://github.com/riceWithoutIce/ClickMaster/releases/tag/v0.1.0)。

支持选择不同应用窗口，以窗口消息发送鼠标左键、右键或单个键盘按键；另有针对已核验 TapTapLoot 版本的专用 IPC。消息成功入队不等于玩法响应：TapTapLoot 消息键鼠无效，计算器内容子窗口有旧版单次键盘通过回执，TapTapLoot 专用 IPC 有旧版响应、重复和停止回执，不能宣传任意游戏兼容。

连续执行默认永久，可设1秒～99时59分59秒限时。3秒准备后计时，每次完整操作后按最小/最大毫秒区间等待。按钮、F8和托盘统一停止；显示秒级进度和结束总结，底部更多菜单按需打开诊断。`0.1.0` 验收包已获用户人工回归通过，未将该结果推广至其他系统、任意游戏或长期性能。

IPC完整写入一帧即记为“已发送”。传输层读取确认只作诊断，不阻断后续执行、不重发当前帧；连接及写入失败仍停止。单次短时观察后清理，重复复用连接直到结束；已发送不保证目标消费或玩法响应。

专用IPC允许游戏前台或后台且非最小化，人工操作游戏UI不再因焦点变化停止；人工与自动效果可能叠加。重复运行复用本轮已授权worker与连接，完整唯一实例/窗口及实际客户端仍在每帧写入前核验；单次保持原路径。间隔支持1～60000毫秒，1毫秒只供极限测试，不承诺1kHz；快于目标消费时可能背压超时，已发帧停止后仍可能被处理。受控非游戏测量与真实软件、游戏资源验收分别记录。

“诊断信息”可手动开启60秒性能采集，记录主程序、自有worker和当前WebView环境资源，以及待命/执行、显示/收起四状态对照，在同级data/logs输出有界脱敏JSONL和TXT摘要；默认关闭，不改变连点状态。WebView环境可能共享，不含crashpad；缺失不当作0，工作集求和不等于独占物理内存。

- [使用说明](docs/USAGE.md)：启动、绑定、通道、持续时间与停止。
- [已知限制](docs/LIMITATIONS.md)：运行环境、兼容性及未验证项目。
- [变更记录](version/CHANGELOG.md)；[版本来源](version/VERSION)。

Windows x64 便携 ZIP 解压到可写目录后运行 ClickMaster.exe，数据放在 EXE 同级 `data/`。无本工具安装器、.NET 分发或自动更新；需要兼容的共享 Evergreen WebView2。Windows 10 22H2 / Windows 11 是支持目标，不表示均已实测。

| 目录 | 职责 |
| --- | --- |
| src / src-tauri | 前端及 Rust/Tauri 后端 |
| scripts | 项目环境、构建、校验入口 |
| docs | 使用与限制说明 |
| build | 缓存、测试与本地候选暂存 |
| version / release | 文本版本记录 / 有限发布槽位 |
| aiwork | 需求、路线、workflow及验收证据 |

开发需现有 Node.js 22.12+、npm 10+、Rust 1.90+、MSVC x64/Windows SDK 和 WebView2。在仓库根使用包装入口，将缓存和临时产物留在项目内：

```powershell
./scripts/npm.ps1 ci
./scripts/npm.ps1 run typecheck
./scripts/npm.ps1 run dev
./scripts/npm.ps1 run test:backend
./scripts/npm.ps1 run build:desktop
```

依赖还原需相应网络授权；已有依赖可设置 `CARGO_NET_OFFLINE=true` 离线验收。版本维护 `version/VERSION`，用 `node scripts/version.mjs --sync` 同步自有元数据，`--check`核对，不升级依赖。桌面输出 `build/cargo/release/clickmaster.exe`，不打安装器。

协作按 [workflow](aiwork/workflow/README.md)，产物遵守[项目落点](aiwork/workflow/generated-data.md)及[脱敏规则](aiwork/workflow/sanitization.md)。候选、审核、人测、正式晋级及上传分别授权记录。

仓库：[riceWithoutIce/ClickMaster](https://github.com/riceWithoutIce/ClickMaster)。
