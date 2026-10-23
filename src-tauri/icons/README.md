# 应用图标

`source.png` 是用户提供并授权用于本项目的黄色背景蓝色指针原图副本，原始外部素材只读且未改写。项目不记录其机器路径。

执行 `node scripts/create-icon.mjs` 从仓库内 source 重建 `icon.png`、`icon.ico` 与前端 `src/assets/app-icon.png`；只保留必需PNG图像块，不恢复旧紫色C图标。当前源图为48×48 RGBA，ICO包裹同一PNG，未添加第三方依赖或远程素材。Tauri默认窗口图标、EXE资源和托盘使用平台资源，侧栏和关于使用前端资源；实际显示由人工验收。

`build.rs` 显式跟踪平台ICO/PNG，避免增量构建沿用旧PE资源；统一桌面构建入口额外对照最终EXE的实际图标组与引用载荷，不能仅因源码图标正确就判定EXE已更新。
