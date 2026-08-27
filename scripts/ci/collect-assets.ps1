param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[0-9A-Za-z.+-]+$')]
    [string]$Version,

    [Parameter(Mandatory = $true)]
    [ValidateSet('x64', 'arm64')]
    [string]$AssetArch,

    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[a-z0-9-]+$')]
    [string]$AssetKey
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repoDir = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path

if (-not $env:RUNNER_TEMP) {
    throw 'RUNNER_TEMP is not set.'
}

$searchRoots = @(
    @('target', 'src-tauri/target') |
        ForEach-Object { Join-Path $repoDir $_ } |
        Where-Object { Test-Path -LiteralPath $_ -PathType Container }
)
if ($searchRoots.Count -eq 0) {
    throw 'No target directory was found.'
}

$assetDir = Join-Path $env:RUNNER_TEMP "release-assets-$AssetKey"
$portableDir = Join-Path $env:RUNNER_TEMP "portable-$AssetKey"
if ((Test-Path -LiteralPath $assetDir) -or (Test-Path -LiteralPath $portableDir)) {
    throw "Staging directory already exists for $AssetKey."
}
New-Item -ItemType Directory -Path $assetDir, $portableDir | Out-Null

function Find-UniqueBundle {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Description,

        [Parameter(Mandatory = $true)]
        [string]$BundleDirectory,

        [Parameter(Mandatory = $true)]
        [string]$Extension
    )

    $escapedDirectory = [regex]::Escape("\bundle\$BundleDirectory\")
    $bundleCandidates = @(
        Get-ChildItem -Path $searchRoots -Recurse -File -ErrorAction SilentlyContinue |
            Where-Object {
                $_.FullName -match $escapedDirectory -and
                $_.Extension -eq $Extension -and
                $_.Name.Contains($Version, [System.StringComparison]::Ordinal)
            }
    )
    if ($bundleCandidates.Count -ne 1) {
        $paths = ($bundleCandidates | ForEach-Object FullName) -join [Environment]::NewLine
        throw "Expected exactly one $Description for version $Version, found $($bundleCandidates.Count).`n$paths"
    }
    return $bundleCandidates[0]
}

function Copy-StagedAsset {
    param(
        [Parameter(Mandatory = $true)]
        [System.IO.FileInfo]$Source
    )

    $architecturePattern = if ($AssetArch -eq 'x64') {
        '(x64|amd64|x86_64)'
    } else {
        '(arm64|aarch64)'
    }
    if ($Source.Name -notmatch $architecturePattern) {
        throw "$AssetArch asset name is missing an architecture marker: $($Source.Name)"
    }

    $destination = Join-Path $assetDir $Source.Name
    if (Test-Path -LiteralPath $destination) {
        throw "Duplicate staged asset name: $($Source.Name)"
    }
    Copy-Item -LiteralPath $Source.FullName -Destination $destination
}

$msiBundle = Find-UniqueBundle -Description 'Windows MSI bundle' -BundleDirectory 'msi' -Extension '.msi'
$nsisBundle = Find-UniqueBundle -Description 'Windows NSIS bundle' -BundleDirectory 'nsis' -Extension '.exe'
Copy-StagedAsset -Source $msiBundle
Copy-StagedAsset -Source $nsisBundle

$releaseExecutables = @(
    Get-ChildItem -Path $searchRoots -Recurse -File -Filter 'sealantern.exe' -ErrorAction SilentlyContinue |
        Where-Object {
            $_.FullName -match '\\release\\sealantern\.exe$' -and
            $_.FullName -notmatch '\\bundle\\'
        }
)
if ($releaseExecutables.Count -ne 1) {
    $paths = ($releaseExecutables | ForEach-Object FullName) -join [Environment]::NewLine
    throw "Expected exactly one Windows executable, found $($releaseExecutables.Count).`n$paths"
}

$releaseDir = $releaseExecutables[0].DirectoryName
Copy-Item -LiteralPath $releaseExecutables[0].FullName -Destination $portableDir
Get-ChildItem -Path $releaseDir -File -Filter '*.dll' -ErrorAction SilentlyContinue |
    ForEach-Object { Copy-Item -LiteralPath $_.FullName -Destination $portableDir }
Copy-Item -LiteralPath (Join-Path $repoDir 'LICENSE'), (Join-Path $repoDir 'NOTICE') -Destination $portableDir

$portablePath = Join-Path $assetDir "Sea.Lantern_${Version}_windows_${AssetArch}_portable.zip"
Compress-Archive -Path (Join-Path $portableDir '*') -DestinationPath $portablePath

Write-Host "Staged Windows $AssetArch assets:"
Get-ChildItem -Path $assetDir -File | Sort-Object Name | ForEach-Object FullName
