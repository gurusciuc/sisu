param([Parameter(Mandatory)][string]$Exe,[Parameter(Mandatory)][string]$Out)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class TabPaintTest{
 public struct Rect{public int l,t,r,b;}
 [DllImport("user32.dll")]public static extern bool GetWindowRect(IntPtr h,out Rect r);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)]public static extern IntPtr FindWindowEx(IntPtr p,IntPtr a,string cls,string name);
 [DllImport("user32.dll")]public static extern IntPtr GetDlgItem(IntPtr h,int id);
 [DllImport("user32.dll",EntryPoint="SendMessageW")]public static extern IntPtr Send(IntPtr h,int m,IntPtr w,IntPtr l);
 [DllImport("user32.dll")]public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
 [DllImport("user32.dll")]public static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll")]static extern uint GetWindowThreadProcessId(IntPtr h,out uint pid);
 public delegate bool EnumProc(IntPtr h,IntPtr l);
 [DllImport("user32.dll")]static extern bool EnumWindows(EnumProc callback,IntPtr l);
 public static int VisibleWindows(int pid){
   int count=0;EnumWindows((h,l)=>{uint owner;GetWindowThreadProcessId(h,out owner);if(owner==pid&&IsWindowVisible(h))count++;return true;},IntPtr.Zero);return count;
 }
}
'@
$null=[TabPaintTest]::SetThreadDpiAwarenessContext([IntPtr](-4))
$env:BROWSER_TEST_HOME=Join-Path ([IO.Path]::GetTempPath()) ('sisu-tab-paint-'+[guid]::NewGuid())
$null=New-Item -ItemType Directory -Force $Out
$app=Start-Process $Exe -PassThru
function Tabs {
 $after=[IntPtr]::Zero
 while(($after=[TabPaintTest]::FindWindowEx($hw,$after,'Button','New tab')) -ne [IntPtr]::Zero){
   # The + button has the same accessible label. Tab controls use ID 100.
   $rect=New-Object TabPaintTest+Rect;$null=[TabPaintTest]::GetWindowRect($after,[ref]$rect)
   if($rect.r-$rect.l -gt 80){$after}
 }
}
function Check-Pixels($name,$selected) {
 Start-Sleep -Milliseconds 200
 # The keyboard helper verifies foreground ownership before sending input.
 & "$PSScriptRoot/window.ps1" -BrowserPid $app.Id -Action keys -Value '{ESC}'
 $rect=New-Object TabPaintTest+Rect;$null=[TabPaintTest]::GetWindowRect($hw,[ref]$rect)
 $bitmap=New-Object Drawing.Bitmap ($rect.r-$rect.l),($rect.b-$rect.t)
 $g=[Drawing.Graphics]::FromImage($bitmap)
 try {
   $g.CopyFromScreen($rect.l,$rect.t,0,0,$bitmap.Size)
   $index=0
   foreach($tab in @(Tabs)){
     $r=New-Object TabPaintTest+Rect;$null=[TabPaintTest]::GetWindowRect($tab,[ref]$r)
     $color=$bitmap.GetPixel([int](($r.l+$r.r)/2)-$rect.l,$r.t-$rect.t+6)
     $expected=if($index -eq $selected){@(43,47,47)}else{@(14,15,17)}
     if($color.R -ne $expected[0] -or $color.G -ne $expected[1] -or $color.B -ne $expected[2]){throw "$name tab $index has a stale selection fill ($color)"}
     if($index -ne $selected){
       $oldClose=$bitmap.GetPixel($r.r-$rect.l-17,[int](($r.t+$r.b)/2)-$rect.t)
       if($oldClose.R -ne 14 -or $oldClose.G -ne 15 -or $oldClose.B -ne 17){throw "$name has a stale close-button rectangle ($oldClose)"}
     }
     $index++
   }
   $bitmap.Save((Join-Path $Out "$name.png"),[Drawing.Imaging.ImageFormat]::Png)
 } finally {$g.Dispose();$bitmap.Dispose()}
}
try{
 for($i=0;$i -lt 80;$i++){$app.Refresh();if($app.MainWindowHandle -ne [IntPtr]::Zero){break};Start-Sleep -Milliseconds 100}
 $hw=$app.MainWindowHandle
 Start-Sleep -Milliseconds 500
 foreach($layout in 'top','left'){
   if($layout -eq 'left'){& "$PSScriptRoot/control.ps1" -BrowserPid $app.Id -Action button -Value 'Change tab placement'}
   & "$PSScriptRoot/control.ps1" -BrowserPid $app.Id -Action button -Value 'New tab'
   Check-Pixels "$layout-after-create" 1
   $tabs=@(Tabs);$null=[TabPaintTest]::Send($tabs[0],245,[IntPtr]::Zero,[IntPtr]::Zero)
   Check-Pixels "$layout-after-select" 0
   & "$PSScriptRoot/control.ps1" -BrowserPid $app.Id -Action button -Value 'Close active tab'
   Check-Pixels "$layout-after-close" 0
 }
 $baseline=[TabPaintTest]::VisibleWindows($app.Id)
 $null=[TabPaintTest]::Send($hw,273,[IntPtr]16,[IntPtr]::Zero)
 Start-Sleep -Milliseconds 700
 $address=& "$PSScriptRoot/control.ps1" -BrowserPid $app.Id -Action address
 if($address -ne 'chrome://extensions/'){throw "Extension manager is outside the selected Sisu tab ($address)"}
 if([TabPaintTest]::VisibleWindows($app.Id) -ne $baseline){throw 'Extension manager opened an extra top-level window'}
 $before=@(Tabs).Count
 $null=[TabPaintTest]::Send($hw,273,[IntPtr]16,[IntPtr]::Zero)
 if(@(Tabs).Count -ne $before){throw 'Existing extension manager was duplicated'}
 & "$PSScriptRoot/window.ps1" -BrowserPid $app.Id -Action capture -OutputFile (Join-Path $Out 'extension-manager.png')
 [ordered]@{method='Foreground screen pixels';layouts=@('top','left');inactiveFill=$true;activeFill=$true;noStaleCloseRectangle=$true;create=$true;select=$true;close=$true;managerInTab=$true;managerReused=$true;noExtraManagerWindow=$true}|ConvertTo-Json|Set-Content (Join-Path $Out 'tab-paint.json')
} finally {
 if(!$app.HasExited){$null=$app.CloseMainWindow();$null=$app.WaitForExit(10000)}
}
