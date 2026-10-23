# IPC 分阶段验证记录

日期：2026-10-22。用户已批准[契约](task.md)。状态：开发及独立审核完成，single传输读取完成；用户回复“刚才没看清，先暂停”，实测按要求暂停，repeat未执行。

| 项目 | 状态 |
| --- | --- |
| Coder稳定快照及纯验证 | PASS：编译、51核心+21时间链+2attempt检查，七源码hash冻结 |
| 独立Verifier | PASS：独立重跑74项、AST零错误，七源码hash一致，无BLOCKER |
| 新single写/drain/资源释放 | PASS：唯一一帧写/drain完成，目标客户端身份确认，正常Dispose及worker退出 |
| 新single人工响应 | 未看清，用户要求暂停；不记成功或失败 |
| repeat准入（新single完成且人工确认） | 未满足：缺本次有效人工反馈；未生成有效feedback门槛文件 |
| 最多5次、至少2秒低频 | 未执行 |
| F8原生注册/注销与实际取消 | 本次注册/注销均成功；未按F8，真实送达/取消未验 |
| 目标失效停止、60秒总限 | 纯策略通过；本次正常约8.893秒结束，未触发真实失效/总限 |
| 工作输入/焦点/光标完整隔离 | 未确认 |

本轮不以旧人工成功替代新single反馈；原IPC-DRAIN计时修复与旧attempt保留。未授权产品集成或发布。

## single唯一实测与人工暂停

主协调者核对独立审核及七源码hash、确认本任务runtime未使用后，从仓库根执行一次 `& ./build/verification/TAPTAPLOOT-IPC-REPEAT/coder/probe.ps1 -RunOnce -Stage single`。发送前提示用户暂停键鼠、保持游戏可见非最小化且非前台，可按F8停止；准备阶段至少5秒。

北京时间13:28:17左右发送数量1的一帧，WriteCompleted=true、DrainCompleted=true，目标客户端身份通过；13:28:20左右监督结束，正常Dispose、worker已退出、未请求Kill、监督查询对象释放，F8注册与注销均成功。真实读取证据本轮已取得，但不能代替游戏解析/事件或可见效果。回执：`build/verification/TAPTAPLOOT-IPC-REPEAT/runtime/single/worker-result.json` 与 `supervisor-result.json`。

主协调者询问本次人工响应后，用户选择“刚才没看清，先暂停”。按要求立即停止推进，不重新发送、不执行repeat、不请求F8操作。单次原始回执的human-pending不改写，补充 `human-observation.json` 记录未看清及暂停；没有生成 `human-feedback.json`，repeat门槛不满足，repeat运行目录不存在。single attempt已消耗，恢复时不能直接重跑它。

## 保留与未验

主协调者复核99个旧证据/产品文件hash全部未变，见 `coordinator/after-single.json`；独立Verifier另复核110文件未变。新增任务文档和两份真实回执的定向机器路径扫描无匹配；检查不等于全系统零外写证明。未改游戏/产品、未发键鼠/PostMessage、未提交推送发布或清理旧结果。

repeat持续响应/帧间隔真实证据、F8真实取消、目标退出真实停止、60秒真实超时及完整工作输入/焦点/光标隔离均未验。当前暂停来自用户的明确选择，不是读取失败。后续等用户恢复后对齐观察方式与新的必要尝试，不擅自绕过固定attempt或将旧轮人工成功搬入本轮。
