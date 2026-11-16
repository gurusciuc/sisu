param([Parameter(Mandatory)][string]$Exe,[Parameter(Mandatory)][string]$Out)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class HoverTest{
 public struct Rect {public int l,t,r,b;}
 public struct Point {public int x,y;}
 [DllImport("user32.dll")]public static extern bool GetWindowRect(IntPtr h,out Rect r);
 [DllImport("user32.dll")]public static extern IntPtr GetDlgItem(IntPtr h,int id);
 [DllImport("user32.dll",EntryPoint="SendMessageW")]public static extern IntPtr Send(IntPtr h,int msg,IntPtr w,IntPtr l);
 [DllImport("user32.dll")]public static extern bool SetCursorPos(int x,int y);
 [DllImport("user32.dll")]public static extern bool GetCursorPos(out Point p);
 [DllImport("user32.dll")]public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr c);
 [DllImport("user32.dll")]public static extern bool RedrawWindow(IntPtr h,IntPtr rect,IntPtr region,uint flags);
 [DllImport("user32.dll")]public static extern bool UpdateWindow(IntPtr h);
 [DllImport("dwmapi.dll")]public static extern int DwmFlush();
}
'@
$null=[HoverTest]::SetThreadDpiAwarenessContext([IntPtr](-4))
$previous=New-Object HoverTest+Point
$null=[HoverTest]::GetCursorPos([ref]$previous)
$env:BROWSER_TEST_HOME=Join-Path ([IO.Path]::GetTempPath()) ('sisu-hover-'+[guid]::NewGuid())
$null=New-Item -ItemType Directory -Force $Out
$app=Start-Process $Exe -PassThru
function Rect($h){
 $r=New-Object HoverTest+Rect;$null=[HoverTest]::GetWindowRect($h,[ref]$r);return $r
}
function Point-At($h){
 $r=Rect $h;$null=[HoverTest]::SetCursorPos([int](($r.l+$r.r)/2),[int](($r.t+$r.b)/2))
 $script:anchor=$h;$script:anchorX=[int](($r.l+$r.r)/2);$script:anchorY=[int](($r.t+$r.b)/2)
 $null=[HoverTest]::Send($h,512,[IntPtr]::Zero,[IntPtr]0x00100010)
}
function Assert-Color($h,$hover,$label){
 $null=[HoverTest]::SetCursorPos($script:anchorX,$script:anchorY)
 if($script:anchor -ne [IntPtr]::Zero){$null=[HoverTest]::Send($script:anchor,512,[IntPtr]::Zero,[IntPtr]0x00100010);$null=[HoverTest]::RedrawWindow($script:anchor,[IntPtr]::Zero,[IntPtr]::Zero,0x121)}
 # Complete an already-pending paint. This does not invalidate the old button,
 # so a missing invalidation still leaves the stale pixels visible.
 $null=[HoverTest]::UpdateWindow($h)
 $null=[HoverTest]::DwmFlush()
 $r=Rect $h
 $bitmap=New-Object Drawing.Bitmap 1,1
 $g=[Drawing.Graphics]::FromImage($bitmap)
 try{
  $g.CopyFromScreen($r.l+9,[int](($r.t+$r.b)/2),0,0,$bitmap.Size)
  $c=$bitmap.GetPixel(0,0)
  $expected=if($hover){@(58,58,58)}else{@(11,11,11)}
  if($c.R -ne $expected[0] -or $c.G -ne $expected[1] -or $c.B -ne $expected[2]){
    $pointer=New-Object HoverTest+Point;$null=[HoverTest]::GetCursorPos([ref]$pointer)
    throw "$label rendered $c; pointer $($pointer.x),$($pointer.y); button $($r.l),$($r.t),$($r.r),$($r.b)"
  }
 }finally{$g.Dispose();$bitmap.Dispose()}
}
function Point-At-Page {
 $r=Rect $hw;$null=[HoverTest]::SetCursorPos($r.l+300,$r.t+180)
 $script:anchor=[IntPtr]::Zero;$script:anchorX=$r.l+300;$script:anchorY=$r.t+180
 Start-Sleep -Milliseconds 40
}
try{
 for($i=0;$i -lt 80;$i++){$app.Refresh();if($app.MainWindowHandle -ne [IntPtr]::Zero){break};Start-Sleep -Milliseconds 100}
 $hw=$app.MainWindowHandle
 & "$PSScriptRoot/window.ps1" -BrowserPid $app.Id -Action keys -Value '{ESC}'
 $min=[HoverTest]::GetDlgItem($hw,30);$max=[HoverTest]::GetDlgItem($hw,31)
 Point-At $min
 Assert-Color $min $true 'Initial hover'
 # A new control's move can precede the previous control's leave notification.
 $null=[HoverTest]::Send($max,512,[IntPtr]::Zero,[IntPtr]0x00100010)
 $null=[HoverTest]::Send($min,675,[IntPtr]::Zero,[IntPtr]::Zero)
 Point-At $max
 Assert-Color $min $false 'Previous button after hover handoff'
 Assert-Color $max $true 'Current button after hover handoff'
 Point-At-Page
 Assert-Color $max $false 'Button after pointer leaves'
 foreach($state in 'maximize','restore'){
  Point-At $max
  $null=[HoverTest]::Send($max,245,[IntPtr]::Zero,[IntPtr]::Zero)
  Start-Sleep -Milliseconds 350
  Point-At-Page
  Assert-Color $max $false "Button after $state"
 }
 Point-At $max
 $null=[HoverTest]::Send($max,31,[IntPtr]::Zero,[IntPtr]::Zero)
 Point-At-Page
 Assert-Color $max $false 'Button after capture cancellation'
 & "$PSScriptRoot/window.ps1" -BrowserPid $app.Id -Action capture -OutputFile (Join-Path $Out 'hover-cleared.png')
 [ordered]@{method='Foreground screen pixels';hoverVisible=$true;oldHoverCleared=$true;pointerLeave=$true;maximize=$true;restore=$true;captureCancellation=$true}|ConvertTo-Json|Set-Content (Join-Path $Out 'hover.json')
}finally{
 if(!$app.HasExited){$null=$app.CloseMainWindow();$null=$app.WaitForExit(10000)}
 $null=[HoverTest]::SetCursorPos($previous.x,$previous.y)
}
