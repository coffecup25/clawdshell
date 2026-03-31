# ClawdShell installer for Windows PowerShell
# Usage: irm https://clawdshell.dev/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Version = if ($env:CLAWDSHELL_VERSION) { $env:CLAWDSHELL_VERSION } else { "latest" }
$Repo = if ($env:CLAWDSHELL_REPO) { $env:CLAWDSHELL_REPO } else { "coffecup25/clawdshell" }

$Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }
$BinaryName = "clawdshell-windows-${Arch}.exe"

if ($Version -eq "latest") {
    $DownloadUrl = "https://github.com/${Repo}/releases/latest/download/${BinaryName}"
} else {
    $DownloadUrl = "https://github.com/${Repo}/releases/download/${Version}/${BinaryName}"
}

$InstallDir = "$env:USERPROFILE\.local\bin"
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$Dest = Join-Path $InstallDir "clawdshell.exe"

Write-Host ""
Write-Host "  clawdshell " -ForegroundColor DarkYellow -NoNewline
Write-Host "installer"
Write-Host "  windows-${Arch}" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  Downloading..." -ForegroundColor DarkGray -NoNewline

try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $Dest -UseBasicParsing
    Write-Host "`r  Downloaded to $Dest          " -ForegroundColor DarkYellow
} catch {
    Write-Host "`r  Download failed.                    " -ForegroundColor Red
    Write-Host "  $DownloadUrl" -ForegroundColor DarkGray
    exit 1
}

# Add to PATH for this session
if ($env:PATH -notlike "*$InstallDir*") {
    $env:PATH = "$InstallDir;$env:PATH"
}

# Add to user PATH permanently
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$InstallDir;$UserPath", "User")
    Write-Host "  Added $InstallDir to PATH" -ForegroundColor DarkGray
}

Write-Host ""

# Run the interactive installer
& $Dest --install
