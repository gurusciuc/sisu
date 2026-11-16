param([Parameter(Mandatory)][string]$Stage,[Parameter(Mandatory)][string]$Destination)
$ErrorActionPreference='Stop'
$stagePath=(Resolve-Path -LiteralPath $Stage).Path
foreach($required in 'desktop_browser.exe','desktop_browser.dll','libcef.dll','LICENSE','NOTICE','THIRD_PARTY.md','CREDITS.html','notices/rust-dependencies.txt'){
  if(!(Test-Path -LiteralPath (Join-Path $stagePath $required))){throw "Incomplete staging directory ($required)"}
}
$manifest=Get-Content "$PSScriptRoot/../Cargo.toml" -Raw
$version=[regex]::Match($manifest,'(?m)^version = "([0-9.]+)"').Groups[1].Value
if(!$version){throw 'Missing package version'}
$null=New-Item -ItemType Directory -Force $Destination
$dir=(Resolve-Path -LiteralPath $Destination).Path
if($dir -eq $stagePath -or $dir.StartsWith($stagePath+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){throw 'Choose a directory outside the runtime tree'}
$name="desktop-browser-$version-preview-windows-x64.zip"
$file=Join-Path $dir $name
if(Test-Path -LiteralPath $file){throw "Archive already exists at $file"}
& tar.exe -a -cf $file -C $stagePath .
if($LASTEXITCODE){throw 'Archive failed'}
$hash=(Get-FileHash -Algorithm SHA256 -LiteralPath $file).Hash.ToLowerInvariant()
[IO.File]::WriteAllText("$file.sha256","$hash  $name`n")
Get-Item -LiteralPath $file,"$file.sha256"
