param([Parameter(Mandatory)][string]$Exe,[Parameter(Mandatory)][string]$Out)
$ErrorActionPreference='Stop'
Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public class BrowserFrameTest{
 public struct Rect{public int l,t,r,b;}
 [DllImport("user32.dll")]public static extern IntPtr GetDlgItem(IntPtr h,int id);
 [DllImport("user32.dll")]public static extern bool GetWindowRect(IntPtr h,out Rect rect);
 [DllImport("user32.dll")]public static extern bool IsZoomed(IntPtr h);
 [DllImport("user32.dll")]public static extern bool IsIconic(IntPtr h);
 [DllImport("user32.dll")]public static extern bool ShowWindow(IntPtr h,int command);
 [DllImport("user32.dll",EntryPoint="SendMessageW")]public static extern IntPtr Send(IntPtr h,int m,IntPtr w,IntPtr l);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)]public static extern int GetWindowText(IntPtr h,StringBuilder title,int max);
 public static bool HasFont(IntPtr h){return Send(h,49,IntPtr.Zero,IntPtr.Zero)!=IntPtr.Zero;}
}
'@
$env:BROWSER_TEST_HOME=Join-Path ([IO.Path]::GetTempPath()) ('sisu-frame-'+[guid]::NewGuid())
$null=New-Item -ItemType Directory -Force $Out
$app=Start-Process $Exe -PassThru
try{
 for($i=0;$i -lt 80;$i++){$app.Refresh();if($app.MainWindowHandle -ne [IntPtr]::Zero){break};Start-Sleep -Milliseconds 100}
 Start-Sleep -Milliseconds 500
 $hw=$app.MainWindowHandle
 $title=[Text.StringBuilder]::new(100);$null=[BrowserFrameTest]::GetWindowText($hw,$title,100)
 if($title.ToString() -ne 'Sisu'){throw 'Unexpected window title'}
  $hasFont=[BrowserFrameTest]::HasFont([BrowserFrameTest]::GetDlgItem($hw,7))
 if(!$hasFont){throw "Address font was not assigned"}
 $rect=New-Object BrowserFrameTest+Rect;$tab=New-Object BrowserFrameTest+Rect;$button=New-Object BrowserFrameTest+Rect
 $null=[BrowserFrameTest]::GetWindowRect($hw,[ref]$rect)
 $null=[BrowserFrameTest]::GetWindowRect([BrowserFrameTest]::GetDlgItem($hw,100),[ref]$tab)
 $null=[BrowserFrameTest]::GetWindowRect([BrowserFrameTest]::GetDlgItem($hw,30),[ref]$button)
 if([Math]::Abs(($tab.t+$tab.b-$button.t-$button.b)/2) -gt 2){throw 'Caption control is not aligned with tab'}
 $point=[IntPtr]((($rect.t+24) -shl 16) -bor ($rect.l+600))
 if([BrowserFrameTest]::Send($hw,132,[IntPtr]::Zero,$point).ToInt32() -ne 2){throw 'Tab-row drag region was not a caption'}
 $max=[BrowserFrameTest]::GetDlgItem($hw,31)
 $null=[BrowserFrameTest]::Send($max,245,[IntPtr]::Zero,[IntPtr]::Zero)
 if(![BrowserFrameTest]::IsZoomed($hw)){throw 'Maximize failed'}
 $null=[BrowserFrameTest]::Send($max,245,[IntPtr]::Zero,[IntPtr]::Zero)
 if([BrowserFrameTest]::IsZoomed($hw)){throw 'Restore failed'}
 $null=[BrowserFrameTest]::Send([BrowserFrameTest]::GetDlgItem($hw,30),245,[IntPtr]::Zero,[IntPtr]::Zero)
 if(![BrowserFrameTest]::IsIconic($hw)){throw 'Minimize failed'}
 $null=[BrowserFrameTest]::ShowWindow($hw,9)
 & "$PSScriptRoot/window.ps1" -BrowserPid $app.Id -Action capture -OutputFile "$Out/new-tab.png"
 [ordered]@{title='Sisu';explicitFontAssigned=$hasFont;captionAligned=$true;dragHitTest=$true;maximize=$true;restore=$true;minimize=$true}|ConvertTo-Json|Set-Content "$Out/frame.json"
}finally{
 if(!$app.HasExited){$null=$app.CloseMainWindow();$null=$app.WaitForExit(10000)}
}
