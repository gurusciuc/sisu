param([Parameter(Mandatory)][int]$BrowserPid,
    [ValidateSet('navigate','button','close','geometry','address','type','suggestions','addressKey')][string]$Action,
    [string]$Value)
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class BrowserControl {
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowEx(IntPtr parent,IntPtr after,string cls,string text);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern IntPtr SendMessage(IntPtr h,int m,IntPtr w,string text);
 [DllImport("user32.dll",EntryPoint="SendMessageW")] public static extern IntPtr Send(IntPtr h,int m,IntPtr w,IntPtr l);
 [DllImport("user32.dll",EntryPoint="SendMessageW",CharSet=CharSet.Unicode)] public static extern IntPtr Read(IntPtr h,int m,IntPtr w,StringBuilder value);
 [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h,int m,IntPtr w,IntPtr l);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr h,StringBuilder name,int n);
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h,out uint pid);
 delegate bool EnumProc(IntPtr h,IntPtr l);
 [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc fn,IntPtr l);
 public static IntPtr Frame(int pid){
   IntPtr frame=IntPtr.Zero;
   EnumWindows((h,l)=>{uint owner;GetWindowThreadProcessId(h,out owner);if(owner!=pid)return true;
     var c=new StringBuilder(100);GetClassName(h,c,c.Capacity);if(c.ToString()!="DesktopBrowserFrame")return true;frame=h;return false;},IntPtr.Zero);return frame;
 }
}
'@
$p=Get-Process -Id $BrowserPid
$h=[BrowserControl]::Frame($BrowserPid)
if($h -eq [IntPtr]::Zero){$h=$p.MainWindowHandle}
if ($p.ProcessName -ne 'desktop_browser' -or !$h.ToInt64()) {throw 'Expected fixture browser'}
$class=[Text.StringBuilder]::new(128)
$null=[BrowserControl]::GetClassName($h,$class,128)
if($Action -eq 'close'){ $null=[BrowserControl]::PostMessage($h,16,[IntPtr]::Zero,[IntPtr]::Zero);return }
if ($class.ToString() -ne 'DesktopBrowserFrame') {throw 'Expected the custom shell'}
if($Action -eq 'type'){
    if($Value.Length -gt 8192){throw 'Input is too long'}
    $null=[BrowserControl]::PostMessage($h,273,[IntPtr]14,[IntPtr]::Zero)
    Start-Sleep -Milliseconds 100
    $edit=[BrowserControl]::FindWindowEx($h,[IntPtr]::Zero,'Edit',$null)
    $null=[BrowserControl]::SendMessage($edit,12,[IntPtr]::Zero,$Value)
}elseif($Action -eq 'suggestions'){
    $list=[BrowserControl]::FindWindowEx($h,[IntPtr]::Zero,'ListBox','Address suggestions')
    if($list -eq [IntPtr]::Zero){throw 'Address suggestions control absent'}
    $n=[BrowserControl]::Send($list,395,[IntPtr]::Zero,[IntPtr]::Zero).ToInt32()
    $rows=@(for($i=0;$i -lt $n;$i++){
        $buf=[Text.StringBuilder]::new(8192)
        $null=[BrowserControl]::Read($list,393,[IntPtr]$i,$buf)
        $buf.ToString()
    })
    ConvertTo-Json -InputObject $rows -Compress
}elseif($Action -eq 'addressKey'){
    $key=switch($Value){'down'{40};'up'{38};'enter'{13};'escape'{27};default{throw 'Expected an address key'}}
    $edit=[BrowserControl]::FindWindowEx($h,[IntPtr]::Zero,'Edit',$null)
    $null=[BrowserControl]::Send($edit,256,[IntPtr]$key,[IntPtr]::Zero)
}elseif($Action -eq 'address'){
    $edit=[BrowserControl]::FindWindowEx($h,[IntPtr]::Zero,'Edit',$null)
    $buf=[Text.StringBuilder]::new(8192)
    $null=[BrowserControl]::Read($edit,13,[IntPtr]8192,$buf)
    $buf.ToString()
} elseif ($Action -eq 'navigate') {
    if($Value -notin '#settings','browser://settings','browser://settings/' -and $Value -notmatch '^http://localhost:[0-9]+/[A-Za-z0-9/?=_&-]*$') {throw 'Expected a local test destination'}
    $edit=[BrowserControl]::FindWindowEx($h,[IntPtr]::Zero,'Edit',$null)
    if (!$edit.ToInt64()){throw 'Address field absent'}
    $null=[BrowserControl]::SendMessage($edit,12,[IntPtr]::Zero,$Value)
    $null=[BrowserControl]::Send($edit,256,[IntPtr]13,[IntPtr]::Zero)
} elseif ($Action -eq 'button') {
    $b=[BrowserControl]::FindWindowEx($h,[IntPtr]::Zero,'Button',$Value)
    if (!$b.ToInt64()){throw "Button absent: $Value"}
    $null=[BrowserControl]::Send($b,245,[IntPtr]::Zero,[IntPtr]::Zero)
} else { $h.ToInt64() }
