# Exact, authorized TASK-014 cleanup. Never recurse for deletion or follow links.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../..'))
$recordRel = 'aiwork/workflow/runs/TASK-014-WORKSPACE-CLEANUP'
$outputRel = 'build/verification/TASK-014-WORKSPACE-CLEANUP/execution'
$latestRel = 'build/staging/TASK-013-UI-SKETCH-ALIGN'
function Full([string]$relative) {
    if ([string]::IsNullOrWhiteSpace($relative) -or $relative.Contains('\') -or
        [IO.Path]::IsPathRooted($relative) -or $relative.Contains(':') -or
        ($relative.Split('/') | Where-Object { $_ -in @('', '.', '..') })) { throw 'Invalid relative path' }
    $full = [IO.Path]::GetFullPath((Join-Path $repoRoot $relative))
    if (-not $full.StartsWith($repoRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Path outside repository' }
    return $full
}
function Hash([string]$relative) {
    $stream = [IO.File]::OpenRead((Full $relative))
    try { return [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($stream)) }
    finally { $stream.Dispose() }
}
function LinkFree([string]$relative) {
    $current = $repoRoot
    if (([IO.File]::GetAttributes($current) -band [IO.FileAttributes]::ReparsePoint) -ne 0) { return $false }
    foreach ($part in $relative.Split('/')) {
        $current = Join-Path $current $part
        try { $attr = [IO.File]::GetAttributes($current) }
        catch [IO.FileNotFoundException] { return $true }
        catch [IO.DirectoryNotFoundException] { return $true }
        if (($attr -band [IO.FileAttributes]::ReparsePoint) -ne 0) { return $false }
    }
    return $true
}
function Matches($entry) {
    $p = Full $entry.path
    return ((LinkFree $entry.path) -and [IO.File]::Exists($p) -and
        ([IO.FileInfo]::new($p).Length -eq $entry.bytes) -and ((Hash $entry.path) -eq $entry.sha256))
}
function ActiveRoots {
    $roots = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($proc in Get-CimInstance Win32_Process -ErrorAction Stop) {
        $exe = $proc.ExecutablePath
        if ($exe -and $exe.StartsWith($repoRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
            $rel = [IO.Path]::GetRelativePath($repoRoot, [IO.Path]::GetDirectoryName($exe)).Replace('\', '/')
            [void]$roots.Add($rel)
        }
    }
    return ,$roots
}
function IsActive([string]$relative) {
    foreach ($active in $script:activeRoots) {
        if ($active -eq '.' -or $relative.StartsWith($active + '/', [StringComparison]::OrdinalIgnoreCase)) { return $true }
    }
    return $false
}
function WriteJson([string]$relative, $value) {
    [IO.File]::WriteAllText((Full $relative), ($value | ConvertTo-Json -Depth 20) + "`n", [Text.UTF8Encoding]::new($false))
}
$planRel = 'build/verification/TASK-014-WORKSPACE-CLEANUP/preparation/file-plan.json'
$historyRel = "$recordRel/historical-versions.json"
if ((Hash $planRel) -ne 'ED804C298D45AA08F477BCDB52C2D94BCC641E19DFC88587472EE2CFDD9ADC0D') { throw 'Frozen manifest mismatch' }
if ((Hash $historyRel) -ne 'AD98A6CAC0B53D5D7D2E4A15C5DB8C8E94AEA01A689DCA7A752BAAECEC6EF469') { throw 'Archive record mismatch' }
if (Test-Path -LiteralPath (Full $outputRel)) { throw 'Execution directory already exists; inspect receipt before any new attempt' }
$plan = Get-Content -LiteralPath (Full $planRel) -Raw | ConvertFrom-Json
$history = Get-Content -LiteralPath (Full $historyRel) -Raw | ConvertFrom-Json
$protected = Get-Content -LiteralPath (Full "$recordRel/protected-sources.json") -Raw | ConvertFrom-Json
$tracked = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
$gitFiles = & git -C $repoRoot ls-files
if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect Git tracked files' }
foreach ($p in $gitFiles) { [void]$tracked.Add($p) }
$seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
$deletes = @($plan.entries | Where-Object action -eq 'delete')
foreach ($e in $plan.entries) {
    $null = Full $e.path
    if (-not $seen.Add($e.path)) { throw 'Duplicate manifest path' }
}
foreach ($e in $deletes) {
    if (($e.path -ne '0.1.0-rc.8' -and $e.path -notmatch '^build/(staging|verification|tmp)/') -or
        $e.path.StartsWith($latestRel + '/', [StringComparison]::OrdinalIgnoreCase) -or
        $e.path.StartsWith('build/verification/TASK-014-WORKSPACE-CLEANUP/') -or
        $tracked.Contains($e.path) -or [IO.Path]::GetFileName($e.path) -eq '.gitkeep') { throw 'Forbidden deletion candidate' }
}
foreach ($e in $protected.entries) { if (-not (Matches $e)) { throw 'Protected source baseline changed' } }
$latestArtifacts = @($history.versions | Where-Object version -eq '0.1.0-rc.9' | ForEach-Object artifacts)
foreach ($e in $latestArtifacts) { if (-not (Matches $e)) { throw 'Latest accepted artifact mismatch' } }
$script:activeRoots = ActiveRoots
# Snapshot only non-active retained evidence; active WebView files are intentionally volatile.
$evidence = @($plan.entries | Where-Object { $_.action -eq 'keep-evidence' })
foreach ($e in $evidence) { if (-not (Matches $e)) { throw 'Retained evidence changed before execution' } }
$null = New-Item -ItemType Directory -Path (Full $outputRel)
$started = [DateTime]::UtcNow.ToString('o')
$summary = [ordered]@{
    startedAt = $started; status = 'running'; manifestSha256 = (Hash $planRel)
    historySha256Before = (Hash $historyRel); candidates = $deletes.Count
    processed = 0; deleted = 0; deletedBytes = [long]0; missing = 0; skipped = 0
    emptyDirectoriesRemoved = 0; emptyDirectorySkips = 0
    activeExecutableRoots = @($script:activeRoots)
    retainedLink = 'build/verification/TASK-003/verifier/source/node_modules'
}
WriteJson "$outputRel/progress.json" $summary
$writer = [IO.StreamWriter]::new((Full "$outputRel/files.jsonl"), $false, [Text.UTF8Encoding]::new($false))
$parents = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
try {
    foreach ($e in $deletes) {
        if ($summary.processed % 2000 -eq 0) { $script:activeRoots = ActiveRoots }
        $outcome = 'skipped'; $reason = 'unclassified'; $p = Full $e.path
        try {
            if (-not (LinkFree $e.path)) { $reason = 'reparse-point-in-path' }
            elseif (IsActive $e.path) { $reason = 'active-executable-directory' }
            elseif (-not (Test-Path -LiteralPath $p)) { $outcome = 'missing'; $reason = 'already-absent' }
            elseif (-not [IO.File]::Exists($p)) { $reason = 'no-longer-regular-file' }
            elseif (-not (Matches $e)) { $reason = 'size-or-hash-changed' }
            else {
                # Full() and LinkFree() above check final absolute target and all ancestors.
                Remove-Item -LiteralPath $p -Force -ErrorAction Stop
                $outcome = 'deleted'; $reason = 'matched-approved-manifest'
                $summary.deleted++; $summary.deletedBytes += [long]$e.bytes
                $parent = [IO.Path]::GetDirectoryName($e.path).Replace('\', '/')
                while ($parent -match '^build/(staging|verification|tmp)/') {
                    [void]$parents.Add($parent)
                    $parent = [IO.Path]::GetDirectoryName($parent).Replace('\', '/')
                }
            }
        } catch { $reason = 'access-locked-or-io-error' }
        if ($outcome -eq 'missing') { $summary.missing++ }
        if ($outcome -eq 'skipped') { $summary.skipped++ }
        $writer.WriteLine(([ordered]@{ path = $e.path; outcome = $outcome; reason = $reason; bytes = $e.bytes } | ConvertTo-Json -Compress))
        $summary.processed++
        if ($summary.processed % 1000 -eq 0) {
            $writer.Flush(); WriteJson "$outputRel/progress.json" $summary
            Write-Output ("Processed {0}/{1}; deleted {2}; skipped {3}" -f $summary.processed, $summary.candidates, $summary.deleted, $summary.skipped)
        }
    }
} finally { $writer.Dispose() }
$directoryWriter = [IO.StreamWriter]::new((Full "$outputRel/directories.jsonl"), $false, [Text.UTF8Encoding]::new($false))
try {
    foreach ($relative in ($parents | Sort-Object { $_.Length } -Descending)) {
        try {
            if (-not (LinkFree $relative) -or (IsActive "$relative/")) { continue }
            $p = Full $relative
            if (-not [IO.Directory]::Exists($p)) { continue }
            if ([IO.Directory]::EnumerateFileSystemEntries($p).GetEnumerator().MoveNext()) { continue }
            # Empty directories only; false forbids recursive deletion, including new files.
            [IO.Directory]::Delete($p, $false)
            $summary.emptyDirectoriesRemoved++
            $directoryWriter.WriteLine((@{ path = $relative; outcome = 'empty-directory-removed' } | ConvertTo-Json -Compress))
        } catch { $summary.emptyDirectorySkips++ }
    }
} finally { $directoryWriter.Dispose() }
$errors = [Collections.Generic.List[string]]::new()
foreach ($e in $protected.entries) { if (-not (Matches $e)) { $errors.Add("protected-source-mismatch:$($e.path)") } }
foreach ($e in $latestArtifacts) { if (-not (Matches $e)) { $errors.Add("latest-artifact-mismatch:$($e.path)") } }
foreach ($e in $evidence) { if (-not (Matches $e)) { $errors.Add("retained-evidence-mismatch:$($e.path)") } }
$remaining = @($deletes | Where-Object { Test-Path -LiteralPath (Full $_.path) })
$summary['remainingCandidates'] = $remaining.Count
$summary['protectedSourceChecks'] = @($protected.entries).Count
$summary['retainedEvidenceChecks'] = $evidence.Count
$summary['latestArtifactChecks'] = $latestArtifacts.Count
$summary['verificationErrors'] = $errors.ToArray()
$summary['finishedAt'] = [DateTime]::UtcNow.ToString('o')
$summary.status = if ($errors.Count) { 'verification-failed' } elseif ($summary.skipped -or $remaining.Count) { 'completed-with-skips' } else { 'completed' }
WriteJson "$outputRel/progress.json" $summary
WriteJson "$recordRel/execution-result.json" $summary
$summary | ConvertTo-Json -Depth 10
if ($errors.Count) { exit 1 }
