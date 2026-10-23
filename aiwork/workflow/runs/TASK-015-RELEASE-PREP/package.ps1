# Prepare a ZIP for human release acceptance; never launch the product or publish.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$taskRoot = $PSScriptRoot
$repoRoot = [IO.Path]::GetFullPath((Join-Path $taskRoot '../../../..'))
$stageRel = 'build/staging/TASK-015-RELEASE-PREP'
$outRel = 'build/verification/TASK-015-RELEASE-PREP/coordinator'
function Full([string]$rel) {
    $p = [IO.Path]::GetFullPath((Join-Path $repoRoot $rel))
    if (-not $p.StartsWith($repoRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Outside project output' }
    return $p
}
function HashFile([string]$rel) { return (Get-FileHash -LiteralPath (Full $rel) -Algorithm SHA256).Hash }
function Json([string]$rel, $value) {
    [IO.File]::WriteAllText((Full $rel), ($value | ConvertTo-Json -Depth 20) + "`n", [Text.UTF8Encoding]::new($false))
}
$env:TEMP = Full 'build/tmp'
$env:TMP = $env:TEMP
$env:TMPDIR = $env:TEMP
if (Test-Path -LiteralPath (Full $stageRel)) { throw 'Staging slot exists; do not replace an acceptance package' }
if ((Get-Content -LiteralPath (Full 'version/VERSION') -Raw).Trim() -ne '0.1.0') { throw 'Version mismatch' }
$exeRel = 'build/cargo/release/clickmaster.exe'
$v = [Diagnostics.FileVersionInfo]::GetVersionInfo((Full $exeRel))
if ($v.FileVersion -ne '0.1.0' -or $v.ProductVersion -ne '0.1.0') { throw 'PE version mismatch' }
$signature = (Get-AuthenticodeSignature -LiteralPath (Full $exeRel)).Status.ToString()
if ($signature -ne 'NotSigned') { throw 'Review signature description before packaging' }
$null = New-Item -ItemType Directory -Path (Full $stageRel)
$null = New-Item -ItemType Directory -Path (Full $outRel) -Force
$readme = @'
# ClickMaster 0.1.0

首个正式版的待发布验收包，尚未正式发布。Windows x64 免安装 ZIP，完整解压到可写目录后运行 ClickMaster.exe；需要已有兼容的共享 Evergreen WebView2，不自动安装。

先阅读 docs/USAGE.md 和 docs/LIMITATIONS.md。默认永久、2000 毫秒间隔、3 秒准备；按钮/F8/托盘停止。数据保存在 EXE 同级 data/，升级请保留。目标保持非最小化；已发送不等于游戏响应，停止不能撤回已写入的缓冲。

本包未签名，Windows 可能提示。实验坐标预览、绑定标签和性能后续项仍有已知限制。不自动回退全局输入、注入或驱动。ZIP 不带用户数据。

SHA256SUMS.txt 与 source.json 位于 ZIP 同级，用于核对当前验收包及来源；未创建正式发布标签，待用户人工回归。
'@
[IO.File]::WriteAllText((Full "$stageRel/README.md"), $readme + "`n", [Text.UTF8Encoding]::new($false))
$files = [ordered]@{
    'ClickMaster.exe' = $exeRel
    'README.md' = "$stageRel/README.md"
    'CHANGELOG.md' = 'version/CHANGELOG.md'
    'docs/USAGE.md' = 'docs/USAGE.md'
    'docs/LIMITATIONS.md' = 'docs/LIMITATIONS.md'
}
$zipRel = "$stageRel/ClickMaster-0.1.0-windows-x64.zip"
Add-Type -AssemblyName System.IO.Compression
$zip = [IO.Compression.ZipFile]::Open((Full $zipRel), [IO.Compression.ZipArchiveMode]::Create)
try {
    foreach ($entry in $files.GetEnumerator()) {
        $null = [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, (Full $entry.Value), $entry.Key, [IO.Compression.CompressionLevel]::Optimal)
    }
} finally { $zip.Dispose() }
$content = [Collections.Generic.List[object]]::new()
$zip = [IO.Compression.ZipFile]::OpenRead((Full $zipRel))
try {
    if ($zip.Entries.Count -ne $files.Count) { throw 'Unexpected package file count' }
    foreach ($entry in $zip.Entries) {
        if (-not $files.Contains($entry.FullName)) { throw 'Unexpected package entry' }
        $stream = $entry.Open()
        try { $sha = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($stream)) }
        finally { $stream.Dispose() }
        if ($sha -ne (HashFile $files[$entry.FullName])) { throw 'Embedded file mismatch' }
        $content.Add([ordered]@{ path = $entry.FullName; bytes = $entry.Length; sha256 = $sha })
    }
} finally { $zip.Dispose() }
Json "$outRel/package-content.json" $content.ToArray()
$oldManifest = Get-Content -LiteralPath (Full 'build/verification/TASK-013-UI-SKETCH-ALIGN/coder/source-manifest.json') -Raw | ConvertFrom-Json
$manifest = @($oldManifest.path | Sort-Object -Unique | ForEach-Object {
    [ordered]@{ path = $_; bytes = (Get-Item -LiteralPath (Full $_)).Length; sha256 = (HashFile $_) }
})
Json "$outRel/source-manifest.json" $manifest
$base = (& git -C $repoRoot rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0) { throw 'No source baseline' }
$source = [ordered]@{
    id = 'TASK-015-RELEASE-PREP-0.1.0-human-candidate'
    version = '0.1.0'
    baseCommit = $base
    dirtySnapshot = $true
    sourceFileCount = $manifest.Count
    sourceManifest = "$outRel/source-manifest.json"
    sourceManifestSha256 = (HashFile "$outRel/source-manifest.json")
    sourceTextHashMode = 'working-tree-bytes-before-Git-EOL-normalization'
    executable = [ordered]@{ path = $exeRel; packageEntry = 'ClickMaster.exe'; bytes = (Get-Item -LiteralPath (Full $exeRel)).Length; sha256 = (HashFile $exeRel) }
    zip = [ordered]@{ path = $zipRel; bytes = (Get-Item -LiteralPath (Full $zipRel)).Length; sha256 = (HashFile $zipRel) }
    signature = $signature
    testedBuildHost = [ordered]@{ os = 'Windows 10 22H2'; build = '19045'; architecture = 'x64' }
    checks = [ordered]@{ version = 'PASS'; typecheck = 'PASS'; uiHelpers = 'PASS'; desktopBuild = 'PASS'; peIcon = 'PASS'; packageContent = 'PASS'; rawBackendEntry = 'FAILED-TO-START-0xc0000139'; isolatedBackendWithCommonControlsManifest = '100-passed-0-failed-2-ignored' }
    humanAcceptance = 'NOT RUN'
    windows11 = 'NOT RUN'
    missingOrOldRuntime = 'NOT RUN'
    independentVerifier = 'NOT RUN-coordinator-checks-only'
    release = $false
}
Json "$stageRel/source.json" $source
$checksums = "$(HashFile $exeRel)  ClickMaster.exe`n$(HashFile $zipRel)  ClickMaster-0.1.0-windows-x64.zip`n"
[IO.File]::WriteAllText((Full "$stageRel/SHA256SUMS.txt"), $checksums, [Text.UTF8Encoding]::new($false))
Json "$outRel/package-result.json" ([ordered]@{ version = '0.1.0'; zip = $source.zip; executable = $source.executable; entries = $content.ToArray(); humanAcceptance = 'NOT RUN'; release = $false })
$source | ConvertTo-Json -Depth 10
