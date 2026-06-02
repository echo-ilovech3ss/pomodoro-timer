# Focus Flow Native Windows App Installer
# Installs portably, registers in Start Menu, and places a Desktop Shortcut

$appName = "Focus Flow"
$binaryName = "focus-flow.exe"
$localProgramsDir = "$env:USERPROFILE\AppData\Local\Programs\Focus Flow"
$shortcutDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
$desktopDir = "$env:USERPROFILE\Desktop"

# 1. Create target program directory
Write-Host "Creating Program folder in AppData..." -ForegroundColor Green
New-Item -ItemType Directory -Force -Path $localProgramsDir | Out-Null

# 2. Copy the binary
$sourcePath = Join-Path $PSScriptRoot "target\release\$binaryName"
if (-Not (Test-Path $sourcePath)) {
    Write-Error "Could not find release binary at $sourcePath. Please run 'cargo build --release' first!"
    exit 1
}

$destPath = Join-Path $localProgramsDir $binaryName
Write-Host "Installing standalone binary..." -ForegroundColor Green
Copy-Item -Path $sourcePath -Destination $destPath -Force

# 3. Create shortcuts using WScript COM Object
Write-Host "Creating Start Menu and Desktop shortcuts..." -ForegroundColor Green
$wscript = New-Object -ComObject WScript.Shell

# Start Menu Shortcut
$startMenuLinkPath = Join-Path $shortcutDir "$appName.lnk"
$startMenuLink = $wscript.CreateShortcut($startMenuLinkPath)
$startMenuLink.TargetPath = $destPath
$startMenuLink.WorkingDirectory = $localProgramsDir
$startMenuLink.Description = "Premium Pomodoro Workspace with Procedural Audio"
$startMenuLink.Save()

# Desktop Shortcut
$desktopLinkPath = Join-Path $desktopDir "$appName.lnk"
$desktopLink = $wscript.CreateShortcut($desktopLinkPath)
$desktopLink.TargetPath = $destPath
$desktopLink.WorkingDirectory = $localProgramsDir
$desktopLink.Description = "Premium Pomodoro Workspace with Procedural Audio"
$desktopLink.Save()

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Focus Flow successfully installed as a native app!" -ForegroundColor Cyan
Write-Host "You can now launch it directly from:" -ForegroundColor Yellow
Write-Host " - Your Windows Start Menu ($appName)" -ForegroundColor Yellow
Write-Host " - Your Desktop shortcut" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Cyan
