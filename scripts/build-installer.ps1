# Package a Windows `Streaker.exe` into an Inno Setup installer.
#
# Usage: scripts/build-installer.ps1 <version> <output-dir> <path-to-Streaker.exe>
#
# Run from the repo root. Installs Inno Setup via Chocolatey if it isn't already present.
param(
    [Parameter(Mandatory)][string]$Version,
    [Parameter(Mandatory)][string]$OutDir,
    [Parameter(Mandatory)][string]$Exe
)
$ErrorActionPreference = 'Stop'

$iscc = "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe"
if (-not (Test-Path $iscc)) {
    choco install innosetup --no-progress -y
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

# ISCC resolves relative paths against the .iss file, so pass absolute ones.
$exePath = (Resolve-Path $Exe).Path
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$outPath = (Resolve-Path $OutDir).Path

& $iscc /Q "/DAppVersion=$Version" "/DSourceExe=$exePath" "/O$outPath" scripts\windows-installer.iss
exit $LASTEXITCODE
