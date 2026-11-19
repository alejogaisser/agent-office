<#
.SYNOPSIS
  Measures CPU and private memory of a process and all its descendants (WebView2 included).
.EXAMPLE
  pwsh scripts/measure-resources.ps1 -Name agent-office -Seconds 60
  Prints JSON: {cpuPercent, privateMB, hostPrivateMB, processCount}
.EXAMPLE
  pwsh scripts/measure-resources.ps1 -Name agent-office -Seconds 120 -Network
  Also samples TCP connections of every process in the tree every 2 s and adds
  remoteConnections (count of samples with a non-loopback remote) and remotes (distinct "ip:port (process)").
  Start it right after launching the app so early connections are seen. Loopback, 0.0.0.0, :: are ignored.
.NOTES
  CPU% = total processor-time delta / (wall seconds x logical cores) x 100.
  Descendants are found through Win32_Process.ParentProcessId.
#>
param(
  [string]$Name = "agent-office",
  [int]$Seconds = 60,
  [switch]$Network
)

$ErrorActionPreference = "Stop"

function Get-Tree([int]$RootPid) {
  $all = Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, CreationDate
  $born = @{}
  foreach ($p in $all) { $born[[int]$p.ProcessId] = $p.CreationDate }
  $ids = [System.Collections.Generic.HashSet[int]]::new()
  [void]$ids.Add($RootPid)
  $added = $true
  while ($added) {
    $added = $false
    foreach ($p in $all) {
      # a real child is younger than its parent (a reused parent PID would make it older)
      if ($ids.Contains([int]$p.ParentProcessId) -and -not $ids.Contains([int]$p.ProcessId) -and $p.CreationDate -ge $born[[int]$p.ParentProcessId]) {
        [void]$ids.Add([int]$p.ProcessId)
        $added = $true
      }
    }
  }
  return $ids
}

function Sample([int]$RootPid) {
  $cpu = @{}
  $priv = 0.0
  $hostPriv = 0.0
  foreach ($id in (Get-Tree $RootPid)) {
    $p = Get-Process -Id $id -ErrorAction SilentlyContinue
    if ($null -eq $p) { continue }
    $cpu[$id] = $p.TotalProcessorTime.TotalSeconds
    $priv += $p.PrivateMemorySize64
    if ($id -eq $RootPid) { $hostPriv = $p.PrivateMemorySize64 }
  }
  return @{ Cpu = $cpu; Priv = $priv; HostPriv = $hostPriv }
}

$host_ = Get-Process -Name $Name -ErrorAction SilentlyContinue | Sort-Object StartTime | Select-Object -First 1
if ($null -eq $host_) {
  Write-Error "process '$Name' not found"
  exit 1
}
$rootPid = $host_.Id
$cores = [Environment]::ProcessorCount

function Test-Ignored([string]$addr) {
  return ($addr -eq "::1" -or $addr -eq "::" -or $addr -eq "0.0.0.0" -or $addr.StartsWith("127.") -or $addr -eq "::ffff:127.0.0.1")
}

$remoteSamples = 0
$remotes = [System.Collections.Generic.HashSet[string]]::new()
function Sample-Network([int]$RootPid) {
  $ids = Get-Tree $RootPid
  $conns = Get-NetTCPConnection -ErrorAction SilentlyContinue | Where-Object { $ids.Contains([int]$_.OwningProcess) }
  $found = 0
  foreach ($c in $conns) {
    if (Test-Ignored $c.RemoteAddress) { continue }
    $found++
    $pn = (Get-Process -Id $c.OwningProcess -ErrorAction SilentlyContinue).ProcessName
    [void]$remotes.Add("$($c.RemoteAddress):$($c.RemotePort) ($pn)")
  }
  return $found
}

$s0 = Sample $rootPid
$t0 = Get-Date
if ($Network) {
  while (((Get-Date) - $t0).TotalSeconds -lt $Seconds) {
    if ((Sample-Network $rootPid) -gt 0) { $remoteSamples++ }
    Start-Sleep -Seconds 2
  }
} else {
  Start-Sleep -Seconds $Seconds
}
$s1 = Sample $rootPid
$wall = ((Get-Date) - $t0).TotalSeconds

$delta = 0.0
foreach ($id in $s1.Cpu.Keys) {
  $before = 0.0
  if ($s0.Cpu.ContainsKey($id)) { $before = $s0.Cpu[$id] }
  $delta += [Math]::Max(0.0, $s1.Cpu[$id] - $before)
}

$result = [ordered]@{
  cpuPercent    = [Math]::Round($delta / ($wall * $cores) * 100, 2)
  privateMB     = [Math]::Round($s1.Priv / 1MB, 1)
  hostPrivateMB = [Math]::Round($s1.HostPriv / 1MB, 1)
  processCount  = $s1.Cpu.Count
}
if ($Network) {
  $result.remoteConnections = $remoteSamples
  $result.remotes = @($remotes)
}
[pscustomobject]$result | ConvertTo-Json -Compress
