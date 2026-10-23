# TASK-012 Coder 稳定交接

状态：Coder 完成，等待独立 Verifier。未提交、未推送、未正式发布。

## 变更
- src/App.vue：导航固定为窄图标栏，移除展开/收起控件；目标窗口候选显示精简为标题与 EXE，技术信息保留在按需详情；状态头部和底部操作栏使用文字与颜色。
- src/style.css：窄导航、状态色、折叠详情、紧凑 footer 样式。
- version/VERSION、版本元数据与 version/CHANGELOG.md 更新为 0.1.0-rc.8。
- 未修改 IPC 协议、worker、输入语义、性能逻辑或前端依赖。

## 自测
- npm run typecheck：通过。
- npm run build：通过。
- npm run build:desktop：通过；PE 图标核验通过。
- 未运行后端测试；未进行人工 UI、真实游戏、托盘、F8 或性能验收。

## 快照与候选
- 源码 manifest：build/verification/TASK-012-UI-USER-FACING/coder/source-manifest.json；89 文件；SHA256 99E0E5A8CD610FDD90605A6F5BF0ABE3F54EEB35A93D2B0465C0EF2FBB79A61C
- EXE：build/staging/TASK-012-UI-USER-FACING/ClickMaster.exe；9772544 bytes；SHA256 7EB0E7661BF4700C622416A7FE0F813717DD978CCB571862C9262DB74E4BE61E
- ZIP：build/staging/TASK-012-UI-USER-FACING/ClickMaster-0.1.0-rc.8-windows-x64.zip；3221700 bytes；SHA256 44E1CC66818311653E5469A9D2857ED894E1C482AEC9675367FBA29047AC88B8
- source.json 与 SHA256SUMS.txt 位于同一候选目录；未签名。

## 未运行项
人工界面体验、真实游戏兼容、性能、长期运行和正式发布均未宣称通过。
