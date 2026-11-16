param([Parameter(Mandatory)][int]$BrowserPid,[ValidateSet('inspect','capture','navigate','keys','context','choose','click')][string]$Action='inspect',[string]$Value,[string]$OutputFile)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName UIAutomationClient,UIAutomationTypes,System.Drawing,System.Windows.Forms
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class TestWindow {
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr handle);
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h,IntPtr pid);
 [DllImport("user32.dll")] static extern bool AttachThreadInput(uint from,uint to,bool attach);
 public static bool FocusOwned(IntPtr h){
   uint current=GetCurrentThreadId(),foreground=GetWindowThreadProcessId(GetForegroundWindow(),IntPtr.Zero);
   bool attached=current!=foreground && AttachThreadInput(current,foreground,true);
   try{return SetForegroundWindow(h);}finally{if(attached)AttachThreadInput(current,foreground,false);}
 }
 [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr handle, IntPtr dc, uint flags);
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr handle, out RECT rect);
 [DllImport("user32.dll")] public static extern bool GetCursorPos(out POINT point);
 [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
 [DllImport("user32.dll")] public static extern void mouse_event(uint flags,uint x,uint y,uint data,UIntPtr extra);
 [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
 public struct RECT {public int l,t,r,b;}
 public struct POINT {public int x,y;}
}
'@
[void][TestWindow]::SetThreadDpiAwarenessContext([IntPtr](-4))
$p=Get-Process -Id $BrowserPid
$hw=$p.MainWindowHandle
if (!$hw.ToInt64() -or $p.ProcessName -ne 'desktop_browser') { throw 'Fixture browser window required' }
$root=[System.Windows.Automation.AutomationElement]::FromHandle($hw)
$scope=[System.Windows.Automation.TreeScope]::Descendants
$all=[System.Windows.Automation.Condition]::TrueCondition
function Focus-Browser {
    $null=[TestWindow]::SetForegroundWindow($hw)
    Start-Sleep -Milliseconds 150
    if ([TestWindow]::GetForegroundWindow() -ne $hw) { (New-Object -ComObject WScript.Shell).AppActivate($BrowserPid) | Out-Null; Start-Sleep -Milliseconds 200 }
    if ([TestWindow]::GetForegroundWindow() -ne $hw) { $null=[TestWindow]::FocusOwned($hw);Start-Sleep -Milliseconds 100 }
    if ([TestWindow]::GetForegroundWindow() -ne $hw) { throw 'Could not focus the fixture window' }
}
function Find-Item($name) {
    $condition=[System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::NameProperty,$name)
    $item=$root.FindFirst($scope,$condition)
    if (!$item) {throw "No control named '$name'"}
    return $item
}
switch ($Action) {
 'inspect' {
    $root.FindAll($scope,$all) | ForEach-Object {
        $c=$_.Current
        if ($c.Name) { [pscustomobject]@{name=$c.Name;role=$c.ControlType.ProgrammaticName;bounds=$c.BoundingRectangle.ToString()} }
    } | ConvertTo-Json -Depth 2
 }
 'capture' {
    $rect=New-Object TestWindow+RECT
    [void][TestWindow]::GetWindowRect($hw,[ref]$rect)
    $image=New-Object Drawing.Bitmap ($rect.r-$rect.l),($rect.b-$rect.t)
    $graphics=[Drawing.Graphics]::FromImage($image)
    $dc=$graphics.GetHdc()
    try { if (![TestWindow]::PrintWindow($hw,$dc,2)) {throw 'Window capture failed'} }
    finally {$graphics.ReleaseHdc($dc);$graphics.Dispose()}
    $image.Save($OutputFile,[Drawing.Imaging.ImageFormat]::Png);$image.Dispose()
 }
 'keys' {Focus-Browser;[Windows.Forms.SendKeys]::SendWait($Value)}
 'navigate' {
    if ($Value -notmatch '^http://localhost:[0-9]+/[A-Za-z0-9/?=_-]*$') {throw 'Expected a loopback fixture URL'}
    Focus-Browser
    [Windows.Forms.SendKeys]::SendWait('^l')
    [Windows.Forms.SendKeys]::SendWait($Value)
    [Windows.Forms.SendKeys]::SendWait('{ENTER}')
 }
 'choose' {
    $item=Find-Item $Value
    $item.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 }
 {$_ -in 'click','context'} {
    Focus-Browser
    $item=Find-Item $Value
    $box=$item.Current.BoundingRectangle
    if ($box.IsEmpty) {throw 'Control not visible'}
    $pos=New-Object TestWindow+POINT
    [void][TestWindow]::GetCursorPos([ref]$pos)
    try {
        [void][TestWindow]::SetCursorPos([int]($box.X+$box.Width/2),[int]($box.Y+$box.Height/2))
        if ($Action -eq 'context') {$down=8;$up=16} else {$down=2;$up=4}
        [TestWindow]::mouse_event($down,0,0,0,[UIntPtr]::Zero); [TestWindow]::mouse_event($up,0,0,0,[UIntPtr]::Zero)
    } finally { [void][TestWindow]::SetCursorPos($pos.x,$pos.y) }
 }
}
