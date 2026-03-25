$ErrorActionPreference = "Stop"
$Port = 3000

function Test-CommandAvailable {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command not found: $Name"
    }
}

function Wait-ForApi {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Url,
        [Parameter(Mandatory = $true)]
        [System.Diagnostics.Process]$Process,
        [Parameter(Mandatory = $true)]
        [string]$StdOutLog,
        [Parameter(Mandatory = $true)]
        [string]$StdErrLog
    )

    for ($attempt = 0; $attempt -lt 60; $attempt++) {
        $Process.Refresh()

        if ($Process.HasExited) {
            $stdout = if (Test-Path $StdOutLog) { Get-Content $StdOutLog -Raw } else { "" }
            $stderr = if (Test-Path $StdErrLog) { Get-Content $StdErrLog -Raw } else { "" }
            throw "Login app API exited before becoming ready.`nSTDOUT:`n$stdout`nSTDERR:`n$stderr"
        }

        try {
            Invoke-WebRequest -Uri $Url -UseBasicParsing -TimeoutSec 1 | Out-Null
            return
        } catch {
            Start-Sleep -Milliseconds 500
        }
    }

    $stdout = if (Test-Path $StdOutLog) { Get-Content $StdOutLog -Raw } else { "" }
    $stderr = if (Test-Path $StdErrLog) { Get-Content $StdErrLog -Raw } else { "" }
    throw "Timed out waiting for login app API at $Url.`nSTDOUT:`n$stdout`nSTDERR:`n$stderr"
}

Test-CommandAvailable -Name "bun"
Test-CommandAvailable -Name "cargo"

$repoRoot = Split-Path -Parent $PSScriptRoot
$apiDir = Join-Path $repoRoot "examples\login-app\rest-api"
$appEntry = Join-Path $repoRoot "examples\login-app\app\main.js"
$apiUrl = "http://127.0.0.1:$Port/"
$stdoutLog = Join-Path $env:TEMP "rustyjs-login-app-api-$PID.out.log"
$stderrLog = Join-Path $env:TEMP "rustyjs-login-app-api-$PID.err.log"
$apiProcess = $null

Write-Host "Starting login app API on $apiUrl"

try {
    $startInfo = @{
        FilePath = "bun"
        ArgumentList = @("run", "start")
        WorkingDirectory = $apiDir
        PassThru = $true
        RedirectStandardOutput = $stdoutLog
        RedirectStandardError = $stderrLog
    }

    $apiProcess = Start-Process @startInfo
    Wait-ForApi -Url $apiUrl -Process $apiProcess -StdOutLog $stdoutLog -StdErrLog $stderrLog

    Write-Host "API ready. Launching RustyJS-UI app..."
    Push-Location $repoRoot
    try {
        & cargo run -- $appEntry
    } finally {
        Pop-Location
    }
} finally {
    if ($apiProcess -and -not $apiProcess.HasExited) {
        Stop-Process -Id $apiProcess.Id -Force
    }

    if (Test-Path $stdoutLog) {
        Remove-Item $stdoutLog -Force
    }

    if (Test-Path $stderrLog) {
        Remove-Item $stderrLog -Force
    }
}
