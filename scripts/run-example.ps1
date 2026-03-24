param(
    [string]$Example = "hello_world_counter.js",
    [switch]$PrintOnly
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$examplePath = if ([System.IO.Path]::IsPathRooted($Example)) {
    $Example
} elseif (Test-Path (Join-Path $repoRoot $Example)) {
    Join-Path $repoRoot $Example
} else {
    Join-Path (Join-Path $repoRoot "examples") $Example
}

if (-not (Test-Path $examplePath)) {
    throw "Example not found: $examplePath"
}

$resolvedExamplePath = (Resolve-Path $examplePath).Path

Push-Location $repoRoot
try {
    if ($PrintOnly) {
        Write-Output "cargo run -- `"$resolvedExamplePath`""
        return
    }

    & cargo run -- $resolvedExamplePath
} finally {
    Pop-Location
}
