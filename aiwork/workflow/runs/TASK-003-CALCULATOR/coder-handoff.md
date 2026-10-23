# TASK-003-CALCULATOR Coder 交接

日期：2026-10-22。快照：`TASK-003-CALCULATOR-CODER-S1`。Coder 已停止修改，等待独立 Verifier；真实计算器查询、UIA 和消息投递均未执行。

## 实现与范围

唯一新增工具目录：`build/verification/TASK-003-CALCULATOR/coder/`。`run.ps1` 为默认预检/显式单次执行入口；`worker.ps1` 使用已有 Windows PowerShell/.NET；`CalculatorProbe.cs` 实现计算器专用目标筛选、UIA 只读结果、身份门槛、固定单次数字1配对与自动前后观察；`README.md` 给出命令、限制和结果解释。

没有修改产品源码、旧任务脚本、版本、发布产物或任何原有证据；没有启动 GUI/浏览器/服务器、枚举真实窗口、调用 UIA、发送输入、提交或推送。使用已有系统引用，无依赖下载。初次纯编译缺少 System.Xml 引用，补齐本轮 worker 的系统程序集引用后编译和纯测试通过；失败运行的脱敏回执保留。

## 实际检查

工作目录：仓库根。实际命令：

```powershell
& ./build/verification/TASK-003-CALCULATOR/coder/run.ps1 -SelfTest
```

最终运行编号：`20261022T021957053Z-d9cf9ef75abc4759b39b9ebb62b9fe89`。其 `build-result.sanitized.json` 报告编译成功，20项纯断言通过，`DesktopQueries=false`、`InputPosted=false`。

覆盖精确0/1标签与拒绝任意数字文本、主键区扫描码/释放位、API结果与响应判定分离、fake单次配对、按下失败不释放、释放前身份失效不补发、等待异常时finally配对、释放失败标为partial。没有以运行窗口或真实sender测试替代这些纯逻辑断言。

所有受管理输出在本轮目录：`runs/<unique-run-id>/tmp`、阶段回执/纯测试回执/UTC发送截止时间；一次编译诊断的空临时目录 `compile-tmp/` 保留。TEMP/TMP/TMPDIR 在启动编译worker前均重定向到本轮run。没有系统级文件监控，不声明整个操作系统零外写。

## 执行与停止

独立验收冻结清单为同工具目录的 `source-manifest.sha256`，包括四个工具文件。只在 Verifier 没有当前阻塞发现、主协调者核对后，按契约运行一次：

```powershell
& ./build/verification/TASK-003-CALCULATOR/coder/run.ps1 -ExecuteOnce
```

默认不执行真实输入；`-ExecuteOnce` 是显式门槛，任意按键/句柄参数不存在。脚本目标不唯一、进程名不在 Calculator/CalculatorApp 白名单、UIA结果不唯一/不可读/非0、前台属于计算器、身份关系变化均停止且不重试。18秒发送期限、25秒自有worker watchdog；超时不代表未发送，保留最后阶段且不重跑。worker只向原窗口配对一次，原身份失效不向其他实例释放；外部强制终止不保证释放。

实际 UIA 是否提供 CalculatorResults、当前内容进程名是否在白名单、当前计算器是否响应既定消息均待主协调者本轮实测。错误或门槛拒绝不自动兼容、回退输入或改产品。前台/鼠标前后相同不能证明全程隔离；用户并发手动输入1无法排除，响应结论需要相应证据限制。
