Add-Type -AssemblyName System.Windows.Forms, System.Drawing

$signature = @'
[DllImport("user32.dll")]
public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
[DllImport("user32.dll")]
public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);
[DllImport("user32.dll")]
public static extern bool IsWindowVisible(IntPtr hWnd);
[DllImport("user32.dll")]
public static extern bool SetForegroundWindow(IntPtr hWnd);
'@
$type = Add-Type -MemberDefinition $signature -Name "Win32Utils" -Namespace "Win32" -PassThru

$iconPath = "$env:USERPROFILE\AppData\Local\Programs\Focus Flow\logo_v3.ico"
if (-not (Test-Path $iconPath)) {
    $iconPath = "logo.ico"
}

$icon = New-Object System.Drawing.Icon($iconPath)
$tray = New-Object System.Windows.Forms.NotifyIcon
$tray.Icon = $icon
$tray.Text = "Focus Flow - Pomodoro Workspace"
$tray.Visible = $true

$menu = New-Object System.Windows.Forms.ContextMenu
$showItem = New-Object System.Windows.Forms.MenuItem("Show Focus Flow")
$hideItem = New-Object System.Windows.Forms.MenuItem("Hide to Tray")
$exitItem = New-Object System.Windows.Forms.MenuItem("Exit Application")

$menu.MenuItems.Add($showItem) | Out-Null
$menu.MenuItems.Add($hideItem) | Out-Null
$menu.MenuItems.Add($exitItem) | Out-Null
$tray.ContextMenu = $menu

$toggleWindow = {
    $hwnd = [Win32.Win32Utils]::FindWindow($null, "Focus Flow")
    if ($hwnd -ne [IntPtr]::Zero) {
        $visible = [Win32.Win32Utils]::IsWindowVisible($hwnd)
        if ($visible) {
            [Win32.Win32Utils]::ShowWindow($hwnd, 0) # SW_HIDE
        } else {
            [Win32.Win32Utils]::ShowWindow($hwnd, 9) # SW_RESTORE
            [Win32.Win32Utils]::SetForegroundWindow($hwnd) | Out-Null
        }
    }
}

$showWindow = {
    $hwnd = [Win32.Win32Utils]::FindWindow($null, "Focus Flow")
    if ($hwnd -ne [IntPtr]::Zero) {
        [Win32.Win32Utils]::ShowWindow($hwnd, 9) # SW_RESTORE
        [Win32.Win32Utils]::SetForegroundWindow($hwnd) | Out-Null
    }
}

$hideWindow = {
    $hwnd = [Win32.Win32Utils]::FindWindow($null, "Focus Flow")
    if ($hwnd -ne [IntPtr]::Zero) {
        [Win32.Win32Utils]::ShowWindow($hwnd, 0) # SW_HIDE
    }
}

$exitApp = {
    $hwnd = [Win32.Win32Utils]::FindWindow($null, "Focus Flow")
    if ($hwnd -ne [IntPtr]::Zero) {
        [Win32.Win32Utils]::ShowWindow($hwnd, 9)
        Stop-Process -Name "focus-flow" -Force -ErrorAction SilentlyContinue
    }
    $tray.Visible = $false
    $tray.Dispose()
    exit
}

$showItem.add_Click($showWindow)
$hideItem.add_Click($hideWindow)
$exitItem.add_Click($exitApp)
$tray.add_DoubleClick($toggleWindow)

while ($true) {
    Start-Sleep -Seconds 2
    $process = Get-Process -Name "focus-flow" -ErrorAction SilentlyContinue
    if (-not $process) {
        $tray.Visible = $false
        $tray.Dispose()
        exit
    }
}
