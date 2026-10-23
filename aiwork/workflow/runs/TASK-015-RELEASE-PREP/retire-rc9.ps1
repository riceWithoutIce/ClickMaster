# Retire only the two recorded rc.9 binaries after verified formal publication.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../..'))
$historyRel = 'aiwork/workflow/runs/TASK-014-WORKSPACE-CLEANUP/historical-versions.json'
$receiptRel = 'aiwork/workflow/runs/TASK-015-RELEASE-PREP/rc9-retirement.json'
$published = Get-Content -LiteralPath (Join-Path $repoRoot 'build/verification/TASK-015-RELEASE-PREP/coordinator/remote-published.json') -Raw | ConvertFrom-Json
if ($published.draft -or $published.prerelease -or -not $published.published_at -or $published.tag_name -ne 'v0.1.0') { throw 'Formal publication not verified' }
$zip = Join-Path $repoRoot 'release/current/ClickMaster-0.1.0-windows-x64.zip'
if ((Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash -ne 'ACEE1AE8045055D01DECE59F56FE7D238FA7A3DDEB9F2C6B2C9447EF008630DD') { throw 'Formal package changed' }
if (Test-Path -LiteralPath (Join-Path $repoRoot $receiptRel)) { throw 'Retirement receipt exists; inspect before another operation' }
$history = Get-Content -LiteralPath (Join-Path $repoRoot $historyRel) -Raw | ConvertFrom-Json
$old = @($history.versions | Where-Object version -eq '0.1.0-rc.9')
if ($old.Count -ne 1 -or $old[0].artifacts.Count -ne 2) { throw 'Unexpected rc9 record' }
$active = @(Get-CimInstance Win32_Process -ErrorAction Stop | Where-Object { $_.ExecutablePath } | ForEach-Object { $_.ExecutablePath })
$allowedRoot = 'build/staging/TASK-013-UI-SKETCH-ALIGN'
$allowed = @("$allowedRoot/ClickMaster.exe", "$allowedRoot/ClickMaster-0.1.0-rc.9-windows-x64.zip")
$results = @()
foreach ($a in $old[0].artifacts) {
    if ($a.path -notin $allowed) { throw 'Not an authorized retirement path' }
    $p = [IO.Path]::GetFullPath((Join-Path $repoRoot $a.path))
    if (-not $p.StartsWith($repoRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Outside workspace' }
    $cursor = $repoRoot
    $link = $false
    foreach ($part in $a.path.Split('/')) {
        $cursor = Join-Path $cursor $part
        if ((Test-Path -LiteralPath $cursor) -and ((Get-Item -LiteralPath $cursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) { $link = $true; break }
    }
    $oldAbs = [IO.Path]::GetFullPath((Join-Path $repoRoot $allowedRoot)) + [IO.Path]::DirectorySeparatorChar
    $isActive = @($active | Where-Object { $_.StartsWith($oldAbs, [StringComparison]::OrdinalIgnoreCase) }).Count -gt 0
    $outcome = 'skipped'
    $reason = 'unknown'
    if ($link) { $reason = 'reparse-point' }
    elseif ($isActive) { $reason = 'active-old-program' }
    elseif (-not (Test-Path -LiteralPath $p)) { $outcome = 'already-absent'; $reason = 'not-deleted-this-run' }
    elseif ((Get-Item -LiteralPath $p -Force).PSIsContainer) { $reason = 'not-a-regular-file' }
    elseif ((Get-Item -LiteralPath $p).Length -ne $a.bytes -or (Get-FileHash -LiteralPath $p -Algorithm SHA256).Hash -ne $a.sha256) { $reason = 'size-or-hash-changed' }
    else {
        try {
            Remove-Item -LiteralPath $p -Force -ErrorAction Stop
            $outcome = 'deleted'; $reason = 'recorded-rc9-artifact-superseded-by-verified-0.1.0'
        } catch { $reason = 'locked-or-access-error' }
    }
    $results += [ordered]@{ path = $a.path; bytes = $a.bytes; sha256 = $a.sha256; outcome = $outcome; reason = $reason }
}
$now = [DateTime]::UtcNow.ToString('o')
$receipt = [ordered]@{ version = '0.1.0-rc.9'; supersededBy = '0.1.0'; finishedAt = $now; releaseUrl = $published.html_url; files = $results; userDataPreserved = $true; processesTerminated = $false }
[IO.File]::WriteAllText((Join-Path $repoRoot $receiptRel), ($receipt | ConvertTo-Json -Depth 10) + "`n", [Text.UTF8Encoding]::new($false))
if (@($results | Where-Object outcome -eq 'skipped').Count -eq 0) {
    $old[0].status = 'removed-after-0.1.0-publication'
    $old[0] | Add-Member -NotePropertyName retiredAt -NotePropertyValue $now -Force
    $old[0] | Add-Member -NotePropertyName retirementReceipt -NotePropertyValue $receiptRel -Force
    [IO.File]::WriteAllText((Join-Path $repoRoot $historyRel), ($history | ConvertTo-Json -Depth 30) + "`n", [Text.UTF8Encoding]::new($false))
}
$receipt | ConvertTo-Json -Depth 10
