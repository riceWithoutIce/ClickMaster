# 开发与发布脚本

当前入口：`npm.ps1` 在依赖安装前设置项目内 npm/Cargo/临时缓存；`environment.mjs` 统一下游环境；`project.mjs` 执行开发、检查、构建和后端测试；`create-icon.mjs` 从仓库内用户提供的源图重建平台和前端图标。命令见根 README。版本同步、正式发布和清理仍留后续任务。

实现时遵循：

- 从仓库位置推导路径，不写入机器专属绝对路径。
- 构建按进程设置输出位置，不改用户级工具配置。
- 清理默认先列出候选及保留理由，执行时再次校验仓库内路径、清单和哈希；不使用全盘通配删除。
- 不将 Git 忽略等同于允许删除，不修改 version 与 aiwork 的历史证据。
- 不以清理为由结束外部进程；文件被占用时报告并停止对应操作。

依赖版本由两个锁文件固定。首次初始化可通过包装入口执行 install；已有锁文件的还原使用 ci。验收副本同样使用自己的包装入口，所有路径由副本根解析。普通日志不自动留存；npm 禁用诊断日志留存，错误需按脱敏规则摘录。

桌面构建成功后自动执行 `verify-pe-icon.mjs`，离线读取EXE的 `RT_GROUP_ICON` 与所引用的 `RT_ICON`，严格对照当前ICO的尺寸描述、数量和图像载荷；不加载或执行EXE，资源不一致即整个构建入口失败。可从仓库根手动执行 `node scripts/verify-pe-icon.mjs <exe-path> [ico-path]`。该检查不代替桌面实际图标显示验收。

后端测试通过统一入口设置 `CLICKMASTER_TEST_ROOT`，默认本轮 `build/verification/TASK-002/coder/tests/`；可在运行前指定本项目内其他隔离输出，脚本拒绝项目外路径。旧配置/目录测试也使用此根，避免复跑时污染前轮夹具。只执行可注入逻辑及文件测试，不启动应用或调用真实窗口枚举。

TASK-002 R1及后续复审必须显式使用各自新测试根，例如 `build/verification/TASK-002/coder/r1/tests/`，不得复用旧夹具。`./scripts/npm.ps1 run test:targets`离线编译并执行生产筛选/展示辅助函数，验证EXE/标题/PID部分匹配、大小写、空词、计数及选择排除，不启动UI或依赖新测试库。

TASK-003-OVERLAY使用`build/verification/TASK-003-OVERLAY/coder/`或`verifier/`下的新测试根。`test:targets`同时验证生产坐标草稿的有限JSON转换；Rust覆盖层测试仅纯几何/状态策略及fake renderer，不调用原生窗口创建、真实目标查询或GUI。

TASK-003使用本任务coder/verifier目录下新测试根；`test:targets`同时检查单键下拉/默认、倒计时取整及部分投递/错误码文案。Rust单次输入测试使用人工时钟数值、fake sender/query，仅构造消息与执行注入状态机，不调用真实PostMessage或查询目标。构建原生适配层不等于已验证实际后台接收。
