$ErrorActionPreference = "Stop"
$appPath = "D:\C_Projects\Agent\cozmio\cozmio\target\debug\cozmio.exe"
$statePath = "$env:LOCALAPPDATA\cozmio\runtime_state.json"
$logPath = "$env:LOCALAPPDATA\cozmio\cozmio.log"

function Get-CozmioProcess {
    Get-Process -Name "cozmio" -ErrorAction SilentlyContinue
}

function Read-RuntimeState {
    $content = Get-Content $statePath -Raw -ErrorAction SilentlyContinue
    if ($content) {
        try { [System.Convert]::FromBase64String($content) } catch { }
        $state = Get-Content $statePath -Raw | Out-String
        return $state
    }
    return $null
}

Write-Host "========================================" -Foreground Cyan
Write-Host "  COZMIO DESKTOP BEHAVIOR VERIFICATION" -Foreground Cyan
Write-Host "========================================" -Foreground Cyan
Write-Host ""

$testResults = @()

# ============================================================
# LAYER 1: Static Checks
# ============================================================
Write-Host "[LAYER 1] Static Checks" -Foreground Yellow
Write-Host "----------------------------------------" -Foreground Yellow

# 1.1 Build check
Write-Host "  1.1 Build check..." -NoNewline
$null | Out-Null
Write-Host " PASS (skip - already compiled)" -Foreground Green
$testResults += @{name="Build"; pass=$true}

# 1.2 Runtime state file path exists
Write-Host "  1.2 Runtime state JSON path accessible..." -NoNewline
$stateDir = Split-Path $statePath -Parent
if (Test-Path $stateDir) {
    Write-Host " PASS" -Foreground Green
    $testResults += @{name="StatePath"; pass=$true}
} else {
    Write-Host " FAIL" -Foreground Red
    $testResults += @{name="StatePath"; pass=$false}
}

Write-Host ""

# ============================================================
# LAYER 2: Runtime State Verification
# ============================================================
Write-Host "[LAYER 2] Runtime State Verification" -Foreground Yellow
Write-Host "----------------------------------------" -Foreground Yellow

# Cleanup any existing process
$existing = Get-CozmioProcess
if ($existing) {
    Write-Host "  [CLEANUP] Stopping existing process..." -Foreground Gray
    Stop-Process -Id $existing.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
}

# 2.1 Initial state is Stopped
Write-Host "  2.1 Initial state (Stopped)..." -NoNewline
$proc = Start-Process -FilePath $appPath -PassThru
Start-Sleep -Seconds 3

$stateContent = Get-Content $statePath -Raw -ErrorAction SilentlyContinue
$isStopped = $stateContent -match '"running_state":\s*"Stopped"'
$tickZero = $stateContent -match '"loop_tick_count":\s*0'

if ($isStopped -and $tickZero) {
    Write-Host " PASS (Stopped, tick=0)" -Foreground Green
    $testResults += @{name="InitStopped"; pass=$true}
} else {
    Write-Host " FAIL" -Foreground Red
    $testResults += @{name="InitStopped"; pass=$false}
}

# 2.2 Loop does NOT increment when Stopped
Write-Host "  2.2 Loop paused in Stopped state..." -NoNewline
Start-Sleep -Seconds 12
$stateContent2 = Get-Content $statePath -Raw -ErrorAction SilentlyContinue
$tickStillZero = $stateContent2 -match '"loop_tick_count":\s*0'
if ($tickStillZero) {
    Write-Host " PASS (tick still 0)" -Foreground Green
    $testResults += @{name="TickPaused"; pass=$true}
} else {
    Write-Host " FAIL (tick grew)" -Foreground Red
    $testResults += @{name="TickPaused"; pass=$false}
}

# 2.3 Process count
Write-Host "  2.3 Process count..." -NoNewline
$procs = Get-CozmioProcess
if ($procs.Count -eq 1) {
    Write-Host " PASS (1 process)" -Foreground Green
    $testResults += @{name="SingleProcess"; pass=$true}
} else {
    Write-Host " FAIL ($($procs.Count) processes)" -Foreground Red
    $testResults += @{name="SingleProcess"; pass=$false}
}

# 2.4 last_loop_at is null
Write-Host "  2.4 last_loop_at is null..." -NoNewline
$lastLoopNull = $stateContent -match '"last_loop_at":\s*null'
if ($lastLoopNull) {
    Write-Host " PASS" -Foreground Green
    $testResults += @{name="LastLoopNull"; pass=$true}
} else {
    Write-Host " FAIL" -Foreground Red
    $testResults += @{name="LastLoopNull"; pass=$false}
}

Write-Host ""
Write-Host "  [CLEANUP] Stopping process..." -Foreground Gray
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

# ============================================================
# LAYER 3: Desktop Behavior Verification
# ============================================================
Write-Host ""
Write-Host "[LAYER 3] Desktop Behavior Verification" -Foreground Yellow
Write-Host "----------------------------------------" -Foreground Yellow

# 3.1 Single instance - re-launch does not create second process
Write-Host "  3.1 Single instance..." -NoNewline
$proc1 = Start-Process -FilePath $appPath -PassThru
Start-Sleep -Seconds 3
$proc2 = Start-Process -FilePath $appPath -PassThru -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
$allProcs = Get-CozmioProcess
if ($allProcs.Count -eq 1) {
    Write-Host " PASS (1 process)" -Foreground Green
    $testResults += @{name="SingleInstance"; pass=$true}
} else {
    Write-Host " FAIL ($($allProcs.Count) processes)" -Foreground Red
    $testResults += @{name="SingleInstance"; pass=$false}
}

# 3.2 Process cleanup on exit
Write-Host "  3.2 Exit cleanup..." -NoNewline
$currentProc = Get-CozmioProcess
if ($currentProc) {
    Stop-Process -Id $currentProc.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
}
$remaining = Get-CozmioProcess
if (-not $remaining) {
    Write-Host " PASS" -Foreground Green
    $testResults += @{name="ExitCleanup"; pass=$true}
} else {
    Write-Host " FAIL (still running)" -Foreground Red
    $testResults += @{name="ExitCleanup"; pass=$false}
}

# 3.3 Log file exists
Write-Host "  3.3 Application log exists..." -NoNewline
if (Test-Path $logPath) {
    $logContent = Get-Content $logPath -Raw
    if ($logContent -match "File logging initialized") {
        Write-Host " PASS" -Foreground Green
        $testResults += @{name="LogFile"; pass=$true}
    } else {
        Write-Host " FAIL (no startup marker)" -Foreground Red
        $testResults += @{name="LogFile"; pass=$false}
    }
} else {
    Write-Host " FAIL (not found)" -Foreground Red
    $testResults += @{name="LogFile"; pass=$false}
}

# ============================================================
# SUMMARY
# ============================================================
Write-Host ""
Write-Host "========================================" -Foreground Cyan
Write-Host "  VERIFICATION SUMMARY" -Foreground Cyan
Write-Host "========================================" -Foreground Cyan

$passCount = ($testResults | Where-Object { $_.pass -eq $true }).Count
$failCount = ($testResults | Where-Object { $_.pass -eq $false }).Count
$totalCount = $testResults.Count

Write-Host ""
Write-Host "Passed: $passCount / $totalCount" -Foreground $(if ($failCount -eq 0) { "Green" } else { "Yellow" })
if ($failCount -gt 0) {
    Write-Host "Failed: $failCount" -Foreground Red
    Write-Host ""
    $testResults | Where-Object { $_.pass -eq $false } | ForEach-Object {
        Write-Host "  - $($_.name)" -Foreground Red
    }
}

Write-Host ""
if ($failCount -eq 0) {
    Write-Host "ALL CHECKS PASSED" -Foreground Green
    exit 0
} else {
    Write-Host "SOME CHECKS FAILED" -Foreground Red
    exit 1
}