param([Parameter(Mandatory)][string]$Exe,[Parameter(Mandatory)][string]$Out)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public class SettingsTest {
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowEx(IntPtr h,IntPtr after,string cls,string name);
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll")] public static extern bool IsWindowEnabled(IntPtr h);
 [DllImport("user32.dll")] public static extern IntPtr GetDlgItem(IntPtr h,int id);
 [DllImport("user32.dll")] public static extern int GetDlgCtrlID(IntPtr h);
 [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h,int msg,IntPtr w,IntPtr l);
 [DllImport("user32.dll",EntryPoint="SendMessageW")] public static extern IntPtr Send(IntPtr h,int msg,IntPtr w,IntPtr l);
 [DllImport("user32.dll",EntryPoint="SendMessageW",CharSet=CharSet.Unicode)] public static extern IntPtr Read(IntPtr h,int msg,IntPtr max,StringBuilder buf);
 [DllImport("user32.dll",EntryPoint="SendMessageW",CharSet=CharSet.Unicode)] public static extern IntPtr Write(IntPtr h,int msg,IntPtr w,string text);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h,out uint process);
 public delegate bool EnumProc(IntPtr h,IntPtr data);
 [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc fn,IntPtr data);
 [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr h,EnumProc fn,IntPtr data);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h,StringBuilder text,int n);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr h,StringBuilder text,int n);
 public static IntPtr OwnedDialog(int process,string title){
    IntPtr found=IntPtr.Zero;
    EnumWindows((h,p)=>{uint owner;GetWindowThreadProcessId(h,out owner);if(owner!=process||!IsWindowVisible(h))return true;
      var b=new StringBuilder(256);GetWindowText(h,b,b.Capacity);if(b.ToString()!=title)return true;found=h;return false;},IntPtr.Zero);return found;
 }
 public static IntPtr FileEdit(IntPtr dialog){
    var combo=GetDlgItem(dialog,1148);IntPtr found=IntPtr.Zero;
    if(combo==IntPtr.Zero)EnumChildWindows(dialog,(h,p)=>{if(GetDlgCtrlID(h)!=1148)return true;combo=h;return false;},IntPtr.Zero);
    if(combo==IntPtr.Zero)return found;
    EnumChildWindows(combo,(h,p)=>{var b=new StringBuilder(80);GetClassName(h,b,b.Capacity);
      if(b.ToString()!="Edit")return true;found=h;return false;},IntPtr.Zero);return found;
 }
}
'@
$env:BROWSER_TEST_HOME=Join-Path ([IO.Path]::GetTempPath()) ('desktop-settings-'+[guid]::NewGuid())
$Out=[IO.Path]::GetFullPath($Out)
$null=New-Item -ItemType Directory -Force $Out
$app=$null
function Children($parent,$cls,$name) {
    $after=[IntPtr]::Zero
    while(($after=[SettingsTest]::FindWindowEx($parent,$after,$cls,$name)) -ne [IntPtr]::Zero){$after}
}
function ReadAddress {
    $ed=[SettingsTest]::GetDlgItem($hw,7)
    $s=[Text.StringBuilder]::new(8192)
    $null=[SettingsTest]::Read($ed,13,[IntPtr]8192,$s)
    $s.ToString()
}
function GetPanel {
    foreach($v in Children $hw 'Static' $null){foreach($page in Children $v 'Static' $null){
        foreach($panel in Children $page 'BrowserSettingsPage' 'Settings'){
            if([SettingsTest]::IsWindowVisible($panel)){return $panel}
        }
    }}
}
function WaitFor($predicate,$label) {
    for($i=0;$i -lt 60;$i++){if((& $predicate)){return};Start-Sleep -Milliseconds 100}
    throw "Timed out waiting for $label"
}
function Launch([string[]]$flags) {
    $script:app=Start-Process -FilePath $Exe -ArgumentList ($flags + @('--url=about:blank')) -PassThru
    WaitFor {$app.Refresh();$app.MainWindowHandle -ne [IntPtr]::Zero} 'browser window'
    $script:hw=$app.MainWindowHandle
}
function CloseOwnWindow {
    if($app -and !$app.HasExited){$null=$app.CloseMainWindow();if(!$app.WaitForExit(10000)){throw 'Test window did not close'};if($app.ExitCode -ne 0){throw "Browser exit $($app.ExitCode)"}}
}
function OpenSettings($value) {
    & "$PSScriptRoot/control.ps1" -BrowserPid $app.Id -Action navigate -Value $value
    WaitFor {(ReadAddress) -eq 'browser://settings' -and (GetPanel)} 'the Settings tab and its native controls'
}
function ClickControl($id) {
    $button=[SettingsTest]::GetDlgItem((GetPanel),$id)
    if($button -eq [IntPtr]::Zero){throw "Missing preference $id"}
    $null=[SettingsTest]::Send($button,245,[IntPtr]::Zero,[IntPtr]::Zero)
}
try {
    Launch @()
    OpenSettings '#settings'
    $initial=@(Children $hw 'Button' 'Settings').Count
    if($initial -ne 1){throw 'Expected one Settings tab'}
    & "$PSScriptRoot/window.ps1" -BrowserPid $app.Id -Action capture -OutputFile "$Out/settings-tab.png"
    $null=[SettingsTest]::PostMessage($hw,273,[IntPtr]18,[IntPtr]::Zero)
    OpenSettings 'browser://settings/'
    if(@(Children $hw 'Button' 'Settings').Count -ne 1){throw 'Duplicate Settings tab'}
    ClickControl 201
    ClickControl 203
    $prefs=Get-Content "$env:BROWSER_TEST_HOME/browser-test/everyday/layout.json" -Raw | ConvertFrom-Json
    if($prefs.placement -ne 'left' -or !$prefs.google){throw 'Preferences were not applied'}
    $export=Join-Path $env:BROWSER_TEST_HOME 'bookmarks.html'
    [IO.File]::WriteAllText($export,'<!DOCTYPE NETSCAPE-Bookmark-file-1><DL><DT><H3>Reading</H3><DL><DT><A HREF="https://example.org/article">Article</A></DL></DL>')
    $pick=[SettingsTest]::GetDlgItem((GetPanel),207)
    $null=[SettingsTest]::PostMessage($pick,245,[IntPtr]::Zero,[IntPtr]::Zero)
    WaitFor {[SettingsTest]::OwnedDialog($app.Id,'Import browser data') -ne [IntPtr]::Zero} 'import format chooser'
    $chooser=[SettingsTest]::OwnedDialog($app.Id,'Import browser data')
    $null=[SettingsTest]::PostMessage([SettingsTest]::GetDlgItem($chooser,207),245,[IntPtr]::Zero,[IntPtr]::Zero)
    $pickerTitle='Import a bookmarks HTML export from Brave, Chrome, or Firefox'
    WaitFor {[SettingsTest]::OwnedDialog($app.Id,$pickerTitle) -ne [IntPtr]::Zero} 'bookmark file picker'
    $picker=[SettingsTest]::OwnedDialog($app.Id,$pickerTitle)
    WaitFor {[SettingsTest]::FileEdit($picker) -ne [IntPtr]::Zero} 'file name control'
    $null=[SettingsTest]::Write([SettingsTest]::FileEdit($picker),12,[IntPtr]::Zero,$export)
    $null=[SettingsTest]::PostMessage([SettingsTest]::GetDlgItem($picker,1),245,[IntPtr]::Zero,[IntPtr]::Zero)
    WaitFor {[SettingsTest]::OwnedDialog($app.Id,'Bookmark import') -ne [IntPtr]::Zero} 'import result'
    $resultDialog=[SettingsTest]::OwnedDialog($app.Id,'Bookmark import')
    $null=[SettingsTest]::PostMessage($resultDialog,16,[IntPtr]::Zero,[IntPtr]::Zero)
    WaitFor {[SettingsTest]::IsWindowEnabled($hw) -and [SettingsTest]::OwnedDialog($app.Id,'Bookmark import') -eq [IntPtr]::Zero} 'return from bookmark import'
    $book=Get-Content "$env:BROWSER_TEST_HOME/browser-test/everyday/bookmarks.json" -Raw|ConvertFrom-Json
    if($book[0].url -ne 'https://example.org/article' -or $book[0].folders[0] -ne 'Reading'){throw 'Imported bookmark did not match export'}
    & "$PSScriptRoot/control.ps1" -BrowserPid $app.Id -Action button -Value 'Close active tab'
    WaitFor {!(GetPanel) -and (ReadAddress) -eq ''} 'return to the original page'
    CloseOwnWindow
    Launch @()
    OpenSettings 'browser://settings'
    CloseOwnWindow
    Launch @('--private')
    OpenSettings '#settings'
    ClickControl 200
    ClickControl 202
    CloseOwnWindow
    $final=Get-Content "$env:BROWSER_TEST_HOME/browser-test/everyday/layout.json" -Raw | ConvertFrom-Json
    if($final.placement -ne 'left' -or !$final.google){throw 'Private preferences escaped into the ordinary profile'}
    [ordered]@{addressAlias=$true;menuCommand=$true;uniqueTab=$true;placement='left';search='Google';restart=$true;importViaPicker=$true;privateIsolation=$true;closeRestoredOriginalTab=$true} | ConvertTo-Json | Set-Content "$Out/settings.json"
} finally {
    if($app -and !$app.HasExited){
        foreach($title in 'Bookmark import','Import a bookmarks HTML export from Brave, Chrome, or Firefox'){
            $dialog=[SettingsTest]::OwnedDialog($app.Id,$title)
            if($dialog -ne [IntPtr]::Zero){$null=[SettingsTest]::PostMessage($dialog,16,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Milliseconds 180}
        }
    }
    CloseOwnWindow
}
