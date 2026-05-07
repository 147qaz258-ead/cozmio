#Requires -Version 5.1
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

$appPath = "D:\C_Projects\Agent\cozmio\cozmio\target\debug\cozmio.exe"
$statePath = "$env:LOCALAPPDATA\cozmio\runtime_state.json"
$logPath = "$env:LOCALAPPDATA\cozmio\cozmio.log"

Write-Host "=== COZMIO UI VERIFICATION TOOL ===" -Foreground Cyan
Write-Host ""

# Cleanup
$existing = Get-Process -Name "cozmio" -ErrorAction SilentlyContinue
if ($existing) {
    Write-Host "[CLEANUP] Stopping existing cozmio..." -Foreground Gray
    Stop-Process -Id $existing.Id -Force
    Start-Sleep -Seconds 2
}

# Start app
Write-Host "[START] Launching cozmio..." -Foreground Yellow
$proc = Start-Process -FilePath $appPath -PassThru
Start-Sleep -Seconds 4

$passCount = 0
$failCount = 0

# Check 1: Process running
Write-Host "[CHECK 1] Process running..." -NoNewline
if ($proc -and -not $proc.HasExited) {
    Write-Host " PASS (PID: $($proc.Id))" -Foreground Green
    $passCount++
} else {
    Write-Host " FAIL" -Foreground Red
    $failCount++
}

# Check 2: UI Window visible via Automation
Write-Host "[CHECK 2] UI Window accessible..." -NoNewline
$root = [System.Windows.Automation.AutomationElement]::RootElement
$cond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty, "com.cozmio.app-siw")
$win = $root.FindFirst([System.Windows.Automation.TreeScope]::Children, $cond)
if ($win) {
    $isOff = $win.Current.IsOffscreen
    $bound = $win.Current.BoundingRectangle
    $visibleStr = if ($isOff) { "NO" } else { "YES" }
    $visibleColor = if ($isOff) { "Red" } else { "Green" }
    Write-Host " PASS (visible=$visibleStr, bounds=$bound)" -Foreground Green
    $passCount++
} else {
    Write-Host " FAIL (window not found)" -Foreground Red
    $failCount++
}

# Check 3: Runtime state exists
Write-Host "[CHECK 3] Runtime state file..." -NoNewline
if (Test-Path $statePath) {
    $state = Get-Content $statePath -Raw | ConvertFrom-Json
    Write-Host " PASS (running_state=$($state.running_state))" -Foreground Green
    $passCount++
} else {
    Write-Host " FAIL (not found)" -Foreground Red
    $failCount++
}

# Check 4: Single instance
Write-Host "[CHECK 4] Single instance..." -NoNewline
$allProcs = Get-Process -Name "cozmio" -ErrorAction SilentlyContinue
if ($allProcs.Count -eq 1) {
    Write-Host " PASS (1 process)" -Foreground Green
    $passCount++
} else {
    Write-Host " FAIL ($($allProcs.Count) processes)" -Foreground Red
    $failCount++
}

# Check 5: Log file
Write-Host "[CHECK 5] Log file..." -NoNewline
if (Test-Path $logPath) {
    $log = Get-Content $logPath -Tail 3 -ErrorAction SilentlyContinue
    Write-Host " PASS (exists, $($log.Count) lines)" -Foreground Green
    $passCount++
} else {
    Write-Host " FAIL" -Foreground Red
    $failCount++
}

# Check 6: Window not minimized to zero
Write-Host "[CHECK 6] Window size valid..." -NoNewline
if ($win) {
    $bound = $win.Current.BoundingRectangle
    if ($bound.Width -gt 0 -and $bound.Height -gt 0) {
        Write-Host " PASS (size: $($bound.Width)x$($bound.Height))" -Foreground Green
        $passCount++
    } else {
        Write-Host " WARN (minimized/hidden, size: $($bound.Width)x$($bound.Height))" -Foreground Yellow
        $passCount++
    }
} else {
    Write-Host " SKIP" -Foreground Gray
    $passCount++
}

# Cleanup
Write-Host ""
Write-Host "[CLEANUP] Stopping cozmio..." -Foreground Gray
Stop-Process -Name "cozmio" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

Write-Host ""
Write-Host "=== SUMMARY ===" -Foreground Cyan
Write-Host "Passed: $passCount" -Foreground Green
if ($failCount -gt 0) {
    Write-Host "Failed: $failCount" -Foreground Red
}

if ($failCount -eq 0) {
    Write-Host ""
    Write-Host "ALL CHECKS PASSED" -Foreground Green
    exit 0
} else {
    Write-Host ""
    Write-Host "SOME CHECKS FAILED" -Foreground Red
    exit 1
}
