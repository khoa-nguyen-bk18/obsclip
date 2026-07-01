#Requires -Version 5.1
$ErrorActionPreference = 'Stop'

$Repo = 'khoa-nguyen-bk18/obsclip'
$AppName = 'Obsclip'

function Write-Info([string]$Message) {
    Write-Host "==> $Message"
}

function Write-Err([string]$Message) {
    Write-Error $Message
    exit 1
}

if (-not $IsWindows -and $env:OS -ne 'Windows_NT') {
    Write-Err 'Obsclip install is supported on macOS and Windows only.'
}

$version = $env:OBSCLIP_VERSION
if ($version) {
    $apiUrl = "https://api.github.com/repos/$Repo/releases/tags/v$version"
} else {
    $apiUrl = "https://api.github.com/repos/$Repo/releases/latest"
}

Write-Info 'resolving release…'
try {
    $release = Invoke-RestMethod -Uri $apiUrl -Headers @{ 'User-Agent' = 'obsclip-installer' }
} catch {
    Write-Err "failed to fetch release metadata from $apiUrl"
}

if (-not $version) {
  $version = $release.tag_name -replace '^v', ''
}
if (-not $version) {
    Write-Err 'could not determine release version'
}

Write-Info "installing $AppName v$version (x64)"

$assetName = "Obsclip_${version}_x64_en-US.msi"
$asset = $release.assets | Where-Object { $_.name -eq $assetName } | Select-Object -First 1
if (-not $asset) {
    Write-Err "no asset matching $assetName — see https://github.com/$Repo/releases"
}

$workDir = Join-Path $env:TEMP 'obsclip-install'
New-Item -ItemType Directory -Force -Path $workDir | Out-Null
$msiPath = Join-Path $workDir $assetName

Write-Info "downloading $assetName…"
try {
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $msiPath -UseBasicParsing
} catch {
    Write-Err "download failed: $($asset.browser_download_url)"
}

Write-Info 'running MSI installer…'
$msiArgs = @('/i', $msiPath, '/quiet', '/norestart')
$proc = Start-Process -FilePath 'msiexec.exe' -ArgumentList $msiArgs -Wait -PassThru
if ($proc.ExitCode -ne 0) {
    Write-Err "msiexec failed with exit code $($proc.ExitCode). Try running without /quiet or check UAC/SmartScreen."
}

Remove-Item -Force $msiPath -ErrorAction SilentlyContinue

$candidates = @(
    (Join-Path $env:ProgramFiles "$AppName\obsclip.exe"),
    (Join-Path ${env:ProgramFiles(x86)} "$AppName\obsclip.exe"),
    (Join-Path $env:LOCALAPPDATA "Programs\$AppName\obsclip.exe")
)

$exePath = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1

if ($exePath) {
    Write-Info "launching $AppName…"
    try {
        Start-Process -FilePath $exePath
    } catch {
        Write-Warning "installed but could not launch — open $AppName from the Start Menu."
        exit 0
    }
} else {
    Write-Warning 'installed but could not find obsclip.exe — open Obsclip from the Start Menu.'
    exit 0
}

Write-Host ''
Write-Host 'If Windows blocked the app, click More info → Run anyway.'
Write-Info "done — $AppName v$version is installed and running."
