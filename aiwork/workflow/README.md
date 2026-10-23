# 协作工作流

本目录记录已确认的主协调者、Coder、Verifier 协作规则。流程已约定不等于当前开发已启动；每个任务按用户授权的范围推进。

## 阅读入口

| 文档 | 职责 |
| --- | --- |
| [任务对齐](task-alignment.md) | 任务契约、角色职责、授权与调度 |
| [Coder 开发](coder.md) | 范围内实现、自测、稳定快照与交接 |
| [Verifier 验收](verifier.md) | 独立检查、问题分级、修复复审与停止条件 |
| [收尾与交付](closure.md) | 证据状态、目标实测、版本交付与记录 |
| [文件与文档脱敏](sanitization.md) | 本机路径、敏感内容、日志截图及交付检查 |
| [开发与测试产物落点](generated-data.md) | 临时文件、依赖缓存、测试及验收输出限定在当前项目内 |
| [任务契约模板](templates/task-contract.md) | 开始前统一目标与验收边界 |
| [Coder 交接模板](templates/coder-handoff.md) | 提交稳定快照和自测证据 |
| [Verifier 报告模板](templates/verifier-report.md) | 一次性汇总结论和 findings |

## 标准流程

1. 主协调者与用户对齐任务契约，用户明确开始。
2. Coder 在约定范围内实现、自测并提供稳定快照。
3. 主协调者核对交接完整性，冻结验收对象，再交独立 Verifier。
4. Verifier 按契约验收；主协调者将结果一次性汇总给用户。
5. 用户决定当前修复、延期或接受风险；获准的修复由 Coder 完成。
6. Verifier 仅复查原 finding 及紧邻回归；达到停止条件后结束本轮。
7. 分别记录目标实测与交付状态；仅在相应授权内推进发布。

文档整理等简单任务可由主协调者在授权范围内直接完成，明确记录实际执行角色，不虚构独立验收。本次仅落地 workflow 文档，没有启动 Coder、Verifier 或应用开发。

## 规则来源

- 功能范围以 [冻结需求](../requirements.md) 为准，阶段以 [roadmap](../roadmap.md) 为准。
- 后台兼容性按 [验证方案](../validation.md) 单独取证。
- 输出与归档遵循 [build](../../build/README.md)、[release](../../release/README.md) 和 [version](../../version/README.md) 的职责；本目录不重复维护保留数量。
- 用户当前指令优先；范围变化先更新任务契约和确认记录，再执行受影响工作。
- 项目文件、文档和附带证据统一遵循 [脱敏约定](sanitization.md)，不得带入本机路径或敏感内容。
