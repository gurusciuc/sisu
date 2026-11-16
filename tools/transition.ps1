param([Parameter(Mandatory)][int]$BrowserPid,[Parameter(Mandatory)][string]$OutputFile)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
Add-Type -ReferencedAssemblies System.Drawing @'
using System;
using System.Drawing;
using System.Collections.Generic;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Threading;
public class TabTransition {
  [DllImport("user32.dll")] static extern bool PrintWindow(IntPtr h,IntPtr dc,uint flags);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h,out RECT r);
  [DllImport("user32.dll")] static extern IntPtr SetThreadDpiAwarenessContext(IntPtr ctx);
  [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern IntPtr FindWindowEx(IntPtr h,IntPtr after,string cls,string name);
  [DllImport("user32.dll")] static extern bool PostMessage(IntPtr h,int msg,IntPtr w,IntPtr l);
  struct RECT {public int left,top,right,bottom;}
  public class Sample { public double ms; public int black,oldPage,newPage,white,points; }
  public static List<Sample> Measure(IntPtr h) {
    SetThreadDpiAwarenessContext((IntPtr)(-4));
    RECT r;GetWindowRect(h,out r);
    var plus=FindWindowEx(h,IntPtr.Zero,"Button","New tab");
    if(plus==IntPtr.Zero)throw new Exception("New tab control unavailable");
    var rows=new List<Sample>();var watch=Stopwatch.StartNew();
    using(var bitmap=new Bitmap(r.right-r.left,r.bottom-r.top))using(var g=Graphics.FromImage(bitmap)){
      for(int i=0;i<65;i++){
        if(i==3)PostMessage(plus,245,IntPtr.Zero,IntPtr.Zero);
        if(i==35)PostMessage(FindWindowEx(h,IntPtr.Zero,"Button","Close active tab"),245,IntPtr.Zero,IntPtr.Zero);
        var sample=new Sample{ms=watch.Elapsed.TotalMilliseconds};
        var dc=g.GetHdc();bool ok;try{ok=PrintWindow(h,dc,2);}finally{g.ReleaseHdc(dc);}
        if(!ok)throw new Exception("PrintWindow failed");
        for(int yi=0;yi<6;yi++)for(int xi=0;xi<9;xi++){
          var c=bitmap.GetPixel(32+(bitmap.Width-64)*xi/8,155+(bitmap.Height-191)*yi/5);
          if(c.R<=5&&c.G<=5&&c.B<=5)sample.black++;
          if(c.R==247&&c.G==247&&c.B==241)sample.oldPage++;
          if(c.R==16&&c.G==17&&c.B==19)sample.newPage++;
          if(c.R>=250&&c.G>=250&&c.B>=250)sample.white++;
          sample.points++;
        }
        rows.Add(sample);Thread.Sleep(5);
      }
    }
    return rows;
  }
}
'@
$app=Get-Process -Id $BrowserPid
if($app.ProcessName -ne 'desktop_browser'){throw 'Expected fixture browser'}
$rows=@([TabTransition]::Measure($app.MainWindowHandle))
$result=[ordered]@{method='PrintWindow(PW_RENDERFULLCONTENT), 54 viewport points per sample';createBeforeSample=3;closeBeforeSample=35;samples=$rows;allBlack=@($rows|Where-Object black -ge 51).Count;allWhite=@($rows|Where-Object white -ge 51).Count}
$result|ConvertTo-Json -Depth 4|Set-Content $OutputFile
if($rows[0].oldPage -lt 20 -or $rows[-1].oldPage -lt 20 -or @($rows|Where-Object newPage -ge 20).Count -eq 0){throw 'Create and close endpoints were not captured'}
if($result.allBlack -or $result.allWhite){throw 'A near-solid black or white surface was captured'}
