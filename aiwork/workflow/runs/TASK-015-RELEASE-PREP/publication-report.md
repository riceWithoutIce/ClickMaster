# 0.1.0 正式发布完成

- 发布地址：https://github.com/riceWithoutIce/ClickMaster/releases/tag/v0.1.0
- 发布时间：2026-10-23T08:21:34Z。
- 发布源码提交：`d14a214d149bd380295b8ea6be7c404ef5c294a8`；注释标签 `v0.1.0` 指向该提交，已和远端剥离后的标签目标核对。
- `main` 与标签先原子推送，再创建草稿 Release，核对三项资产后正式公开；不是 prerelease。发布完成回执另行提交，标签不移动。

## 交付与验证

| 资产 | 字节数 | SHA-256 |
| --- | --- | --- |
| ClickMaster-0.1.0-windows-x64.zip | 3,224,535 | ACEE1AE8045055D01DECE59F56FE7D238FA7A3DDEB9F2C6B2C9447EF008630DD |
| SHA256SUMS.txt | 182 | 4E0A3EBAC6A0D138357D8C57C4E3A6B2AD1A57ACDDAE83AF7F4B49738005DC02 |
| source.json | 16,658 | B5DD1CB2896EBE7DEAB1A2044E12CD5A61FED202F9C8B51B88618CFA42520870 |

GitHub API 在草稿及公开后均返回对应的资产大小和 SHA-256 digest，逐项与本地匹配。ZIP 与人工验收包字节不变，当前位于 `release/current/`，暂存目录不另留重复 ZIP。来源文件包括构建基线、84 项验收时指纹、发布提交、文档后续变化和人工通过记录；不承诺重建字节一致。

仓库内 README/使用/限制文档已标正式版；ZIP 内保留人工验收时的准备状态文字，Release 说明明确了后续正式晋级关系。未暗中重新构建或替换用户已验收 ZIP。

构建、包检查和人工结果见 preparation-report/human-acceptance；原始后端入口启动失败与隔离测试副本 100 通过、0 失败、2 忽略分别保留。本轮发布未启动普通产品或发送游戏输入，没有新增 Windows 11、缺失/旧 Runtime、长期资源或独立 Verifier 证据。

## 收尾

- 发布索引已关联实际 sourceCommit、公开 URL、发布时间和三项远端资产指纹，详情见 `publication-result.json`。
- rc.9 两项旧二进制经大小/SHA-256 核验后删除，共 12,865,801 字节；原始版本/来源/验收记录及用户 data 保留，详见 rc9-retirement.json。未删除活动的 0.1.0 EXE 或停止任何外部进程。
- GitHub CLI 首次只读 jq 查询因 Windows 命令包装丢失引号失败；改用结构化 JSON 读取后核验成功，未重复创建 Release 或覆盖资产。
- OVERLAY-001、F-01、性能后续项及 V008-01 测试同步问题按约定延期，未扩大审核或修复范围。

本任务达到发布交付范围，停止继续检查或优化；后续反馈另开任务。
