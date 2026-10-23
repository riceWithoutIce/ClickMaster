param([Parameter(ValueFromRemainingArguments = $true)][string[]]$NpmArguments)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
$locations = @{ TEMP='build/tmp'; TMP='build/tmp'; TMPDIR='build/tmp'; npm_config_cache='build/cache/npm'; NODE_COMPILE_CACHE='build/cache/node'; CARGO_HOME='build/cache/cargo-home'; CARGO_TARGET_DIR='build/cargo' }
foreach ($entry in $locations.GetEnumerator()) {
  $resolved = Join-Path $projectRoot $entry.Value
  New-Item -ItemType Directory -Path $resolved -Force | Out-Null
  [Environment]::SetEnvironmentVariable($entry.Key, $resolved, 'Process')
}
$env:npm_config_update_notifier='false'
$env:npm_config_fund='false'
$env:npm_config_audit='false'
$env:npm_config_logs_max='0'
Push-Location $projectRoot
try { & npm.cmd @NpmArguments; exit $LASTEXITCODE } finally { Pop-Location }
