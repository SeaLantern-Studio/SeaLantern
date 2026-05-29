param(
    [string]$CliPath = "",
    [string]$WorkspaceRoot = "",
    [string]$ServerName = "sl-smoke-paper",
    [string]$McVersion = "1.21.1",
    [string]$Core = "paper",
    [string]$Image = "itzg/minecraft-server",
    [string]$ImageTag = "latest",
    [string]$DataDir = "",
    [string]$ComposeOut = "",
    [int]$GamePort = 25565,
    [switch]$SkipStart,
    [switch]$FullStackCompose
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Resolve-RepoRoot {
    param([string]$ExplicitRoot)

    if (-not [string]::IsNullOrWhiteSpace($ExplicitRoot)) {
        return (Resolve-Path -LiteralPath $ExplicitRoot).Path
    }

    return (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
}

function Resolve-CliPath {
    param([string]$ExplicitCli, [string]$RepoRoot)

    if (-not [string]::IsNullOrWhiteSpace($ExplicitCli)) {
        return (Resolve-Path -LiteralPath $ExplicitCli).Path
    }

    $candidates = @(
        (Join-Path $RepoRoot 'target\debug\sea-lantern.exe'),
        (Join-Path $RepoRoot 'src-tauri\target\debug\sea-lantern.exe'),
        (Join-Path $RepoRoot 'src-tauri\target\debug\sea-lantern.exe')
    )

    foreach ($candidate in $candidates) {
        if (Test-Path -LiteralPath $candidate) {
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }

    throw "Sea Lantern CLI executable was not found. Pass -CliPath explicitly."
}

function Invoke-CheckedCli {
    param(
        [string]$Cli,
        [string[]]$Arguments,
        [switch]$AllowNonZero
    )

    Write-Host "> $Cli $($Arguments -join ' ')"
    & $Cli @Arguments
    $exitCode = $LASTEXITCODE
    if (-not $AllowNonZero -and $exitCode -ne 0) {
        throw "CLI invocation failed with exit code: $exitCode"
    }
    return $exitCode
}

function Ensure-DockerReady {
    $versionOutput = & docker version 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Docker daemon is unavailable. Output: $($versionOutput -join [Environment]::NewLine)"
    }

Write-Host "Docker daemon is reachable. Consider pre-pulling the target image first."
    Write-Host "Example: sealantern docker pull $($Image):$($ImageTag)"
}

$repoRoot = Resolve-RepoRoot -ExplicitRoot $WorkspaceRoot
$cli = Resolve-CliPath -ExplicitCli $CliPath -RepoRoot $repoRoot

if ([string]::IsNullOrWhiteSpace($DataDir)) {
    $DataDir = Join-Path $repoRoot ("tmp\docker-smoke\{0}" -f $ServerName)
}
if ([string]::IsNullOrWhiteSpace($ComposeOut)) {
    $ComposeOut = Join-Path $repoRoot ("tmp\docker-smoke\{0}.compose.yaml" -f $ServerName)
}

$null = New-Item -ItemType Directory -Force -Path $DataDir
$null = New-Item -ItemType Directory -Force -Path ([System.IO.Path]::GetDirectoryName($ComposeOut))

Write-Host "RepoRoot : $repoRoot"
Write-Host "CliPath  : $cli"
Write-Host "DataDir  : $DataDir"
Write-Host "Compose  : $ComposeOut"
Write-Host "Image    : $($Image):$($ImageTag)"

Ensure-DockerReady

if (-not (Test-Path -LiteralPath $cli)) {
    throw "CLI executable does not exist: $cli"
}

Invoke-CheckedCli -Cli $cli -Arguments @('docker', 'doctor')

$createArgs = @(
    'server',
    $ServerName,
    '--runtime', 'docker',
    '--mc', $McVersion,
    '--core', $Core,
    '--image', $Image,
    '--image-tag', $ImageTag,
    '--data-dir', $DataDir,
    '--port', $GamePort,
    '--command-mode', 'rcon'
)

if ($SkipStart) {
    $createArgs += '--create-only'
}
else {
    $createArgs += '--detach'
}

Invoke-CheckedCli -Cli $cli -Arguments $createArgs

Invoke-CheckedCli -Cli $cli -Arguments @('server', 'inspect', $ServerName)
Invoke-CheckedCli -Cli $cli -Arguments @('server', 'status', $ServerName)
Invoke-CheckedCli -Cli $cli -Arguments @('server', 'logs', $ServerName, '--lines', '20')

if (-not $SkipStart) {
    Invoke-CheckedCli -Cli $cli -Arguments @('server', 'send', $ServerName, 'say', 'smoke-check')
    Invoke-CheckedCli -Cli $cli -Arguments @('server', 'restart', $ServerName)
    Invoke-CheckedCli -Cli $cli -Arguments @('server', 'status', $ServerName)
}

$composeArgs = @('compose', 'generate', $ServerName, '--output', $ComposeOut)
if ($FullStackCompose) {
    $composeArgs += @('--full-stack', '--sealantern-data', '/app/data/servers', '--http-port', '3000')
}
Invoke-CheckedCli -Cli $cli -Arguments $composeArgs

Write-Host "Smoke verification completed."
Write-Host "Compose output: $ComposeOut"
