param([Parameter(Mandatory)][int]$BrowserPid,[Parameter(Mandatory)][string]$OutputFile,[int]$Samples=10,[int]$IntervalMs=1000)
$ErrorActionPreference='Stop'
$records=@()
$clock=[Diagnostics.Stopwatch]::StartNew()
for ($sample=0;$sample -lt $Samples;$sample++) {
    $begin=$clock.ElapsedMilliseconds
    $attempts=0
    do {
    $attempts++
    $procs=@(Get-CimInstance Win32_Process -Filter "Name='desktop_browser.exe'" -Property ProcessId,ParentProcessId,CommandLine,CreationDate,KernelModeTime,UserModeTime)
    $tree=@([uint32]$BrowserPid)
    for ($i=0;$i -lt $tree.Count;$i++) {
        $tree+=@($procs|Where-Object { $_.ParentProcessId -eq $tree[$i] -and $_.ProcessId -notin $tree }|ForEach-Object ProcessId)
    }
    $metrics=@(Get-CimInstance Win32_PerfFormattedData_PerfProc_Process -Filter "Name LIKE 'desktop_browser%'" -Property IDProcess,WorkingSetPrivate,PrivateBytes)
    $rows=@(foreach($proc in $procs) {
        if($proc.ProcessId -notin $tree){continue}
        $row=$metrics|Where-Object IDProcess -eq $proc.ProcessId |Select-Object -First 1
        $role=if($proc.ProcessId -eq $BrowserPid){'browser'}elseif($proc.CommandLine -match '--type=([^ "]+)'){$matches[1]}else{'unknown'}
        [pscustomobject]@{pid=$proc.ProcessId;parent=$proc.ParentProcessId;role=$role;measured=[bool]$row;workingSetPrivateBytes=$row.WorkingSetPrivate;privateCommitBytes=$row.PrivateBytes;cpu100ns=([uint64]$proc.KernelModeTime+[uint64]$proc.UserModeTime)}
    })
    $complete=($rows.Count -ge 2 -and @($rows|Where-Object {-not $_.measured}).Count -eq 0)
    if(!$complete){Start-Sleep -Milliseconds 600}
    } until ($complete -or $attempts -ge 5)
    if(!$complete){throw 'Incomplete process tree sample after retries'}
    $records+=@([pscustomobject]@{sample=$sample;atMs=$begin;samplingMs=$clock.ElapsedMilliseconds-$begin;attempts=$attempts;processes=$rows})
    if($sample -lt $Samples-1){Start-Sleep -Milliseconds $IntervalMs}
}
[IO.File]::WriteAllText($OutputFile,($records|ConvertTo-Json -Depth 5))
