# 0.1.0 正式发布操作清单

状态：人工回归通过；用户已明确“开始正式发布”，本清单获授权执行。

## 已确认的发布对象

- 仓库：`riceWithoutIce/ClickMaster`，分支 `main`。
- 版本 / 标签 / Release 标题：`0.1.0` / `v0.1.0` / `ClickMaster 0.1.0`。
- 发布渠道：GitHub Release 正式版，非 prerelease。
- 已验收 ZIP：`build/staging/TASK-015-RELEASE-PREP/ClickMaster-0.1.0-windows-x64.zip`。
- ZIP SHA-256：`ACEE1AE8045055D01DECE59F56FE7D238FA7A3DDEB9F2C6B2C9447EF008630DD`；3,224,535 字节。
- EXE SHA-256：`9F263B8AC713DDDEF93C929DA38A6804077BAD0FAD82FD21B012894EE0A5DE88`。
- 人工回执：`human-acceptance.md`；发布正文：`release-notes.md`。

## 正式发布时执行

1. 保持已验收 ZIP 字节不变。以该包晋级 `release/current/`，转移而非永久复制一套，并更新发布索引、当前状态说明、来源元数据及人工验收关联。包内保留验收时的准备状态说明，以正式 Release、索引和验收记录标识后续晋级；不借晋级偷偷替换内容。
2. 只提交当前任务的版本、文档、锁文件自有版本和发布记录；检查脱敏、忽略规则及源码关联，推送 `main`。
3. 从正式发布提交创建并推送注释标签 `v0.1.0`，不得覆盖已有同名标签/资产。来源文件区分构建基线与工作区指纹、正式发布提交，明确 Git 换行规范化；不声称可重建出同字节。
4. 发布 `ClickMaster 0.1.0`，上传已验收 ZIP、`SHA256SUMS.txt` 和关联正式提交/人工回执的 `source.json`。后两份元数据可在发布前补齐，不能冒充 ZIP 内的原始构建来源。
5. 核对远端标签指向、Release 资产名称/大小/哈希及本地索引，记录实际 URL 与结果。保留更新前的失败证据，发生冲突或失败不覆盖既有正式版本。

## 历史产物

正式包晋级及远端核对成功后，按现有保留规则记录 rc.9 淘汰。只删除清单内可核验的旧包和过时程序，保留用户 `data/`、必要证据与日志；旧程序运行或占用时跳过并记录，不结束外部进程。已验收的用户测试目录不能整棵删除。

## 当前未执行

执行起点尚未晋级 current、填入正式发布索引、提交/推送正式版、创建标签、上传 Release 或清理 rc.9。实际结果单独记录在 `publication-result.json` / `publication-report.md`，本计划不冒充完成回执。
