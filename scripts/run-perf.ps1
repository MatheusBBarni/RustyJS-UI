param(
    [switch]$Release
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot

Push-Location $repoRoot
try {
    if ($Release) {
        & cargo run --bin perf_harness --release
    } else {
        & cargo run --bin perf_harness
    }
} finally {
    Pop-Location
}
