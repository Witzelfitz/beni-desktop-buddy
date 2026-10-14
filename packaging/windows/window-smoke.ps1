param([int]$ProcessId)
$ErrorActionPreference = 'Stop'
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class BeniWindows {
    public delegate bool Callback(IntPtr window, IntPtr unused);
    [DllImport("user32.dll")] public static extern bool EnumWindows(Callback callback, IntPtr unused);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
}
'@
$script:Found = $false
[BeniWindows]::EnumWindows({ param($window, $unused)
    [uint32]$owner = 0
    [void][BeniWindows]::GetWindowThreadProcessId($window, [ref]$owner)
    if ($owner -eq $ProcessId -and [BeniWindows]::IsWindowVisible($window)) { $script:Found = $true }
    return $true
}, [IntPtr]::Zero) | Out-Null
if (-not $script:Found) { exit 2 }
