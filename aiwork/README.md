# ClickMaster 工作记录

本目录保存 Windows 后台连点工具的需求、路线、开发日志和 AI 调试记录。项目当前处于第一版需求已冻结、尚未实现阶段。

## 文档入口

| 文件 | 用途 |
| --- | --- |
| [requirements.md](requirements.md) | 第一版冻结需求、排除项与待定参数 |
| [roadmap.md](roadmap.md) | 开发阶段、交付物与完成条件 |
| [workflow/README.md](workflow/README.md) | 主协调者、Coder、Verifier 流程、职责边界与交接模板 |
| [technology-selection.md](technology-selection.md) | 参考产品技术调查、候选比较与待定选型 |
| [ui-technology-selection.md](ui-technology-selection.md) | 圆角与动效要求下的 Tauri、WPF 等候选比较 |
| [project-layout.md](project-layout.md) | 源码、构建、版本、发布目录及有限归档规则 |
| [devlog.md](devlog.md) | 按日期追加的开发记录 |
| [validation.md](validation.md) | 后台兼容性验证步骤与验收证据 |
| [debug/README.md](debug/README.md) | AI 调试记录规则 |
| [debug/_template.md](debug/_template.md) | 单次问题调查模板 |

## 使用约定

- 需求以 `requirements.md` 为准；变更应记录原因和用户确认，避免实现过程默默扩展范围。
- 路线中的完成状态必须有对应记录或证据。编译成功、消息投递成功和游戏实际响应分别记录。
- 开发日志按时间追加，调试调查按问题单独建文件。
- 文件、文档及证据遵循 [脱敏约定](workflow/sanitization.md)：路径使用仓库相对路径或占位符，命令注明相对执行位置，不保存本机路径、用户标识、凭据或无关个人内容。
- 当前已建立工作文档与目录基线；需求冻结不代表工具已实现或后台兼容性已通过。
